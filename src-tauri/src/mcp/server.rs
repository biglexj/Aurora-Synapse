//! Transporte del servidor MCP de Aurora Synapse.
//!
//! Dos transportes, un mismo núcleo:
//!
//! - **Streamable HTTP** sobre `127.0.0.1` (loopback, nunca `0.0.0.0`): es el que
//!   consume el cliente `SynapseBridge` de Ely Intelligence mediante
//!   `AURORA_SYNAPSE_MCP_URL`.
//! - **stdio**: para agentes de terminal (Codex CLI, clientes MCP de consola) que
//!   lanzan el binario como subproceso y hablan JSON-RPC línea a línea.
//!
//! El servidor HTTP está escrito sobre `tokio` sin dependencias adicionales: el
//! protocolo que necesita es acotado (POST con cuerpo JSON y respuesta JSON), y
//! evitar crates nuevos mantiene `Cargo.lock` estable para el instalador.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::mcp::dispatch::UriOpener;
use crate::mcp::protocol::{
    self, JsonRpcRequest, JsonRpcResponse, SessionStore, INTERNAL_ERROR, INVALID_PARAMS,
    INVALID_REQUEST, METHOD_NOT_FOUND, PARSE_ERROR,
};
use crate::mcp::tools::ToolRegistry;

/// Dirección por defecto del puente MCP. Solo loopback: el puente concede
/// control sobre todo el ecosistema y no debe quedar expuesto a la red local.
pub const DEFAULT_BIND_ADDR: &str = "127.0.0.1:49296";

/// Ruta del endpoint MCP.
pub const MCP_PATH: &str = "/mcp";

/// Descripción del puente para el agente.
///
/// El consumidor la inyecta en el prompt del modelo, así que tiene que ser
/// accionable: qué apps controla, con qué prefijo y qué límites tiene.
const INSTRUCTIONS: &str = "\
Aurora Synapse es el puente MCP del ecosistema Biglex: cada herramienta despacha \
una orden a una aplicación nativa de esta máquina.

Apps con superficie real:
- Super Galería del teléfono: gallery_devices descubre los beacons existentes; \
gallery_status comprueba conexión y autorización; gallery_list consulta medios; \
gallery_download trae un archivo por ID y gallery_send envía un archivo local. \
Requiere conexión por código temporal en Synapse. No hay acceso a la bóveda ni \
a carpetas excluidas. Las transferencias completadas comprueban tamaño y SHA-256.
- Luna Fetch (descargador): luna_status, luna_download, luna_analyze, \
luna_open_folder, luna_focus.
- Prisma (reproductor): prisma_status, prisma_playback, prisma_play, \
prisma_control (verbos: play_pause, next, previous, seek_forward, volume_up, \
toggle_mute, lyrics, queue, search…; la lista completa va en el esquema de la \
herramienta).
- Prisma Upscaler (imágenes): upscaler_status, upscaler_models, upscaler_scale.
- DaVinci Flow (edición y Resolve): davinciflow_status, davinciflow_templates, \
davinciflow_capture, davinciflow_fetch_media, davinciflow_create_fusion_effect, \
davinciflow_insert_native_title, davinciflow_write_subtitles.
- Genéricas: apps_list, classify_intent, dispatch (luna/<acción> y \
prisma/remote, nada más).

Límites que conviene conocer antes de prometer algo:
- Si la app destino no está corriendo, la herramienta falla con un error \
explícito que lo dice. Eso significa que la app no está en marcha, no que el \
puente esté roto: no afirmes que la orden surtió efecto. Interpreta el error de \
la propia llamada; y si tienes a mano la herramienta de estado de esa app \
(prisma_status, upscaler_status, luna_status), úsala antes de actuar.
- upscaler_scale usa el microservicio local o la CLI instalada y solo confirma \
completed cuando guarda un PNG nuevo. La GUI no es un transporte de escalado. \
NCNN local requiere PRISMA_UPSCALER_BIN y PRISMA_MODELS_DIR configurados.
- apps_list distingue supported, reachable y availability. cli_available solo \
confirma la existencia de la CLI, no la disponibilidad de GPU/modelos.
- delivery=sent confirma envío; accepted confirma recepción por la app; \
completed confirma el resultado guardado del escalado. Para reproducción usa \
prisma_playback después de actuar; para Luna no prometas una descarga terminada.
- Las herramientas de análisis y foco de Luna no devuelven acuse de recibo: \
confirman el envío, no la ejecución.
Apps sin superficie de invocación todavía, por tanto sin herramientas aquí: \
WinTTS, Ely-Tesia, LyraFlow, MouziFlow.";

/// Tamaño máximo de la cabecera HTTP aceptada.
const MAX_HEAD_BYTES: usize = 16 * 1024;

/// Tamaño máximo del cuerpo JSON aceptado.
const MAX_BODY_BYTES: usize = 1024 * 1024;

/// Estado compartido del servidor MCP.
pub struct McpState {
    pub sessions: Arc<SessionStore>,
    pub tools: ToolRegistry,
    pub api_key: Option<String>,
    pub opener: UriOpener,
}

impl McpState {
    /// Credencial opcional leída del entorno.
    ///
    /// `AURORA_SYNAPSE_MCP_KEY` activa la comprobación de `Authorization: Bearer`.
    pub fn api_key_from_env() -> Option<String> {
        std::env::var("AURORA_SYNAPSE_MCP_KEY")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }
}

/// Arranca el servidor MCP sobre Streamable HTTP.
///
/// Devuelve el puerto realmente escucha.
pub async fn serve_http(state: Arc<McpState>, addr: &str) -> std::io::Result<u16> {
    let address = addr.parse::<std::net::SocketAddr>().map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "MCP requiere una dirección IP loopback",
        )
    })?;
    if !address.ip().is_loopback() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "MCP solo puede escuchar en loopback",
        ));
    }
    let listener = TcpListener::bind(addr).await?;
    let port = listener
        .local_addr()
        .map(|address| address.port())
        .unwrap_or(0);

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((socket, _peer)) => {
                    let state = state.clone();
                    tokio::spawn(async move {
                        if let Ok(Err(error)) = tokio::time::timeout(
                            Duration::from_secs(135),
                            handle_connection(state, socket),
                        )
                        .await
                        {
                            tracing_debug(&format!("mcp: conexión terminada con error: {error}"));
                        }
                    });
                }
                Err(error) => {
                    tracing_debug(&format!("mcp: no se pudo aceptar la conexión: {error}"));
                }
            }
        }
    });

    Ok(port)
}

/// Atiende una conexión HTTP completa.
async fn handle_connection(state: Arc<McpState>, mut socket: TcpStream) -> std::io::Result<()> {
    let Ok(Some((head, leftover))) =
        tokio::time::timeout(Duration::from_secs(5), read_head(&mut socket)).await
    else {
        return Ok(());
    };

    let mut lines = head.split("\r\n");
    let request_line = lines.next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or_default().to_string();

    let mut headers: HashMap<String, String> = HashMap::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }

    if let Some(origin) = headers.get("origin") {
        let allowed = reqwest::Url::parse(origin).ok().is_some_and(|url| {
            let host = url.host_str().unwrap_or("");
            (host == "localhost"
                || host
                    .trim_matches(['[', ']'])
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback()))
                && matches!(url.scheme(), "http" | "https" | "tauri")
        });
        if !allowed {
            return write_response::<&str, &str>(
                &mut socket,
                "403 Forbidden",
                "application/json",
                "{\"error\":\"origin_denied\"}",
                &[],
            )
            .await;
        }
    }

    // Los clientes de agente usan HTTP nativo; nunca conceder CORS a sitios externos.
    if method == "OPTIONS" {
        return write_response(
            &mut socket,
            "204 No Content",
            "text/plain",
            "",
            &[
                ("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS"),
                (
                    "Access-Control-Allow-Headers",
                    "Content-Type, Authorization, Mcp-Session-Id, Mcp-Protocol-Version",
                ),
            ],
        )
        .await;
    }

    if path == "/mcp/status" || path == "/status" {
        let body = json!({
            "status": "online",
            "app": "Aurora Synapse",
            "component": "mcp-bridge",
            "protocol": protocol::PROTOCOL_VERSION,
            "transport": "streamable-http",
            "tools": state.tools.len(),
            "sessions": state.sessions.len(),
        })
        .to_string();
        return write_response::<&str, &str>(&mut socket, "200 OK", "application/json", &body, &[])
            .await;
    }

    // Autorización opcional por token.
    if let Some(expected) = &state.api_key {
        let provided = headers
            .get("authorization")
            .and_then(|value| value.strip_prefix("Bearer "))
            .map(|value| value.trim().to_string());
        if provided.as_deref() != Some(expected.as_str()) {
            let body = json!({"error": "unauthorized"}).to_string();
            return write_response(
                &mut socket,
                "401 Unauthorized",
                "application/json",
                &body,
                &[("WWW-Authenticate", "Bearer")],
            )
            .await;
        }
    }

    match (method.as_str(), path.as_str()) {
        ("GET", MCP_PATH) => {
            let body = json!({
                "error": "method_not_allowed",
                "detail": "Este servidor MCP habla Streamable HTTP: usa POST /mcp para las llamadas JSON-RPC."
            })
            .to_string();
            write_response(
                &mut socket,
                "405 Method Not Allowed",
                "application/json",
                &body,
                &[("Allow", "POST, DELETE, OPTIONS")],
            )
            .await
        }
        ("DELETE", MCP_PATH) => {
            if let Some(session) = headers.get("mcp-session-id") {
                state.sessions.remove(session);
            }
            write_response::<&str, &str>(&mut socket, "204 No Content", "text/plain", "", &[]).await
        }
        ("POST", MCP_PATH) => {
            let length: usize = headers
                .get("content-length")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);
            if length > MAX_BODY_BYTES {
                let body = json!({"error": "payload_too_large"}).to_string();
                return write_response::<&str, &str>(
                    &mut socket,
                    "413 Payload Too Large",
                    "application/json",
                    &body,
                    &[],
                )
                .await;
            }

            let mut body = vec![0u8; length];
            if length > 0 {
                // El cuerpo puede haber llegado en el mismo segmento TCP que
                // las cabeceras: se consume primero lo que ya sobró.
                let from_leftover = leftover.len().min(length);
                body[..from_leftover].copy_from_slice(&leftover[..from_leftover]);
                if from_leftover < length
                    && !matches!(
                        tokio::time::timeout(
                            Duration::from_secs(10),
                            socket.read_exact(&mut body[from_leftover..])
                        )
                        .await,
                        Ok(Ok(_))
                    )
                {
                    return Ok(());
                }
            }
            let body = String::from_utf8_lossy(&body).to_string();

            let session_id = headers.get("mcp-session-id").cloned();

            // `initialize` es la vía de recuperación tras un 404: siempre puede
            // abrir una sesión nueva, incluso si el cliente reenvía la cabecera
            // con la sesión caducada. El resto de métodos sí exigen una sesión
            // viva, y esa comprobación NO renueva actividad: una petición que
            // después se rechaza no debe mantener viva la sesión.
            let parsed: Result<JsonRpcRequest, _> =
                serde_json::from_str(body.trim_start_matches('\u{feff}'));
            if let Ok(request) = &parsed {
                if request.method != "initialize" {
                    if let Some(session) = &session_id {
                        if !state.sessions.exists(session) {
                            let detail = json!({
                                "error": "session_not_found",
                                "detail": "La sesión caducó o nunca existió. Envía una petición initialize para abrir una sesión nueva; el servidor responderá con un Mcp-Session-Id nuevo.",
                                "idle_timeout_seconds": protocol::SESSION_IDLE_TIMEOUT.as_secs(),
                            })
                            .to_string();
                            return write_response::<&str, &str>(
                                &mut socket,
                                "404 Not Found",
                                "application/json",
                                &detail,
                                &[],
                            )
                            .await;
                        }
                    }
                }
            }

            let (payload, extra_headers) = match parsed {
                Ok(request) => handle_request(&state, request, session_id).await,
                Err(error) => {
                    let response =
                        JsonRpcResponse::err(Value::Null, PARSE_ERROR, format!("JSON inválido: {error}"));
                    (
                        serde_json::to_value(response).unwrap_or(Value::Null),
                        Vec::new(),
                    )
                }
            };

            // Una notificación no produce respuesta de contenido.
            if payload.is_null() {
                return write_response(
                    &mut socket,
                    "202 Accepted",
                    "text/plain",
                    "",
                    &extra_headers,
                )
                .await;
            }

            let body = payload.to_string();
            write_response(
                &mut socket,
                "200 OK",
                "application/json",
                &body,
                &extra_headers,
            )
            .await
        }
        _ => {
            let body = json!({"error": "not_found"}).to_string();
            write_response::<&str, &str>(
                &mut socket,
                "404 Not Found",
                "application/json",
                &body,
                &[],
            )
            .await
        }
    }
}

/// Ejecuta una petición JSON-RPC y devuelve la respuesta más las cabeceras
/// que deben acompañarla (por ejemplo, el `Mcp-Session-Id` del handshake).
pub async fn handle_rpc(
    state: &Arc<McpState>,
    body: &str,
    session_id: Option<String>,
) -> (Value, Vec<(&'static str, String)>) {
    let parsed: Result<JsonRpcRequest, _> =
        serde_json::from_str(body.trim_start_matches('\u{feff}'));
    match parsed {
        Ok(request) => handle_request(state, request, session_id).await,
        Err(error) => {
            let response =
                JsonRpcResponse::err(Value::Null, PARSE_ERROR, format!("JSON inválido: {error}"));
            (
                serde_json::to_value(response).unwrap_or(Value::Null),
                Vec::new(),
            )
        }
    }
}

/// Atiende una petición JSON-RPC ya interpretada.
///
/// Separar el análisis del cuerpo permite al transporte HTTP comprobar la
/// sesión antes de ejecutar nada sin analizar dos veces el mismo JSON.
pub async fn handle_request(
    state: &Arc<McpState>,
    request: JsonRpcRequest,
    session_id: Option<String>,
) -> (Value, Vec<(&'static str, String)>) {
    let mut extra_headers: Vec<(&'static str, String)> = Vec::new();

    let id = request.id.clone().unwrap_or(Value::Null);

    // Las notificaciones no llevan id y no deben recibir respuesta.
    if request.id.is_none() && request.method.starts_with("notifications/") {
        // Una notificación válida es actividad: renueva la sesión.
        if let Some(session) = &session_id {
            state.sessions.touch(session);
        }
        return (Value::Null, extra_headers);
    }

    if request.jsonrpc != "2.0" {
        // Petición mal formada: no renueva la actividad de la sesión.
        let response = JsonRpcResponse::err(id, INVALID_REQUEST, "solo se admite JSON-RPC 2.0");
        return (
            serde_json::to_value(response).unwrap_or(Value::Null),
            extra_headers,
        );
    }

    // A partir de aquí la petición es válida y cuenta como actividad.
    //
    // `initialize` es la excepción: abre una sesión nueva y retira la que
    // traía la cabecera, para no dejar sesiones fantasma en el panel. El
    // cliente debe usar siempre el `Mcp-Session-Id` del último initialize.
    if request.method != "initialize" {
        if let Some(session) = &session_id {
            state.sessions.touch(session);
        }
    }

    match request.method.as_str() {
        "initialize" => {
            let params = request.params.unwrap_or(Value::Null);
            let client = params
                .get("clientInfo")
                .and_then(|info| info.get("name"))
                .and_then(|name| name.as_str())
                .unwrap_or("desconocido")
                .to_string();
            if let Some(previous) = &session_id {
                state.sessions.remove(previous);
            }
            let session = state.sessions.create(&client, protocol::PROTOCOL_VERSION);
            extra_headers.push(("Mcp-Session-Id", session));

            let result = json!({
                "protocolVersion": protocol::PROTOCOL_VERSION,
                "capabilities": {
                    "tools": { "listChanged": false },
                    "resources": { "listChanged": false, "subscribe": false }
                },
                "serverInfo": {
                    "name": "aurora-synapse",
                    "version": env!("CARGO_PKG_VERSION"),
                    "title": "Aurora Synapse — Puente MCP del ecosistema"
                },
                // Este texto lo inyecta el consumidor en el prompt del agente,
                // así que describe lo que el puente SÍ puede hacer y dónde están
                // sus límites. No prometer apps que no exponen superficie.
                "instructions": INSTRUCTIONS,
            });

            let response = JsonRpcResponse::ok(id, result);
            (
                serde_json::to_value(response).unwrap_or(Value::Null),
                extra_headers,
            )
        }
        "ping" => {
            let response = JsonRpcResponse::ok(id, json!({}));
            (
                serde_json::to_value(response).unwrap_or(Value::Null),
                extra_headers,
            )
        }
        "tools/list" => {
            let result = json!({ "tools": state.tools.list() });
            let response = JsonRpcResponse::ok(id, result);
            (
                serde_json::to_value(response).unwrap_or(Value::Null),
                extra_headers,
            )
        }
        "tools/call" => {
            let params = request.params.unwrap_or(Value::Null);
            let name = params.get("name").and_then(|value| value.as_str());
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));

            let Some(name) = name else {
                let response = JsonRpcResponse::err(
                    id,
                    INVALID_PARAMS,
                    "tools/call requiere el parámetro 'name'",
                );
                return (
                    serde_json::to_value(response).unwrap_or(Value::Null),
                    extra_headers,
                );
            };

            if !state.tools.exists(name) {
                let response = JsonRpcResponse::err(
                    id,
                    INVALID_PARAMS,
                    format!("herramienta desconocida: {name}"),
                );
                return (
                    serde_json::to_value(response).unwrap_or(Value::Null),
                    extra_headers,
                );
            }

            let result = state.tools.call(name, arguments).await;
            let response = JsonRpcResponse::ok(id, result);
            (
                serde_json::to_value(response).unwrap_or(Value::Null),
                extra_headers,
            )
        }
        "resources/list" => {
            let response = JsonRpcResponse::ok(id, json!({ "resources": [super::capabilities::descriptor()] }));
            (
                serde_json::to_value(response).unwrap_or(Value::Null),
                extra_headers,
            )
        }
        "resources/read" => {
            let params = request.params.unwrap_or(Value::Null);
            let response = if params["uri"].as_str() == Some(super::capabilities::URI) {
                JsonRpcResponse::ok(id, json!({ "contents": [{
                    "uri": super::capabilities::URI,
                    "mimeType": "application/json",
                    "text": super::capabilities::snapshot(&state.tools).to_string()
                }] }))
            } else {
                JsonRpcResponse::err(id, INVALID_PARAMS, "Recurso desconocido; consultar resources/list")
            };
            (
                serde_json::to_value(response).unwrap_or(Value::Null),
                extra_headers,
            )
        }
        "prompts/list" => {
            let response = JsonRpcResponse::ok(id, json!({ "prompts": [] }));
            (
                serde_json::to_value(response).unwrap_or(Value::Null),
                extra_headers,
            )
        }
        other => {
            let response = JsonRpcResponse::err(
                id,
                METHOD_NOT_FOUND,
                format!("método no soportado: {other}"),
            );
            (
                serde_json::to_value(response).unwrap_or(Value::Null),
                extra_headers,
            )
        }
    }
}

/// Transporte por stdio: una línea JSON-RPC por entrada, una respuesta por salida.
///
/// Pensado para agentes de terminal que lanzan el binario como subproceso.
pub async fn serve_stdio(state: Arc<McpState>) {
    use tokio::io::AsyncBufReadExt;

    let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let (payload, _headers) = handle_rpc(&state, trimmed, None).await;
        if payload.is_null() {
            continue;
        }
        let mut out = payload.to_string();
        out.push('\n');
        let mut stdout = tokio::io::stdout();
        if stdout.write_all(out.as_bytes()).await.is_err() {
            break;
        }
        let _ = stdout.flush().await;
    }
}

/// Lee la cabecera HTTP hasta `\r\n\r\n`.
///
/// Devuelve la cabecera y los bytes que ya se leyeron de más: el cuerpo de la
/// petición puede venir en el mismo segmento TCP, y descartarlo haría que la
/// lectura posterior del cuerpo se quedara esperando datos que nunca llegan.
async fn read_head(socket: &mut TcpStream) -> Option<(String, Vec<u8>)> {
    let mut buffer = Vec::with_capacity(1024);
    let mut chunk = [0u8; 1024];

    loop {
        match socket.read(&mut chunk).await {
            Ok(0) => {
                if buffer.is_empty() {
                    return None;
                }
                break;
            }
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);
                if let Some(position) = find_subslice(&buffer, b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&buffer[..position]).to_string();
                    let leftover = buffer[position + 4..].to_vec();
                    return Some((head, leftover));
                }
                if buffer.len() > MAX_HEAD_BYTES {
                    return None;
                }
            }
            Err(_) => return None,
        }
    }

    Some((String::from_utf8_lossy(&buffer).to_string(), Vec::new()))
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Escribe una respuesta HTTP completa y cierra la conexión.
async fn write_response<S1, S2>(
    socket: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &str,
    extra_headers: &[(S1, S2)],
) -> std::io::Result<()>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    let mut head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for (name, value) in extra_headers {
        head.push_str(&format!("{}: {}\r\n", name.as_ref(), value.as_ref()));
    }
    head.push_str("\r\n");

    socket.write_all(head.as_bytes()).await?;
    socket.write_all(body.as_bytes()).await?;
    socket.flush().await?;
    let _ = socket.shutdown().await;
    Ok(())
}

/// Registro mínimo: evita arrastrar `tracing` como dependencia del módulo.
fn tracing_debug(message: &str) {
    if cfg!(debug_assertions) {
        eprintln!("[aurora-synapse mcp] {message}");
    }
}

/// Tiempo máximo de espera de una operación de despacho.
pub const DISPATCH_TIMEOUT: Duration = Duration::from_secs(8);

/// Marca de error de cliente MCP.
pub const MCP_CLIENT_ERROR: i32 = INTERNAL_ERROR;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::tools::ToolRegistry;

    fn state() -> Arc<McpState> {
        Arc::new(McpState {
            sessions: Arc::new(SessionStore::default()),
            tools: ToolRegistry::new(),
            api_key: None,
            opener: Arc::new(|_uri| Ok(())),
        })
    }

    #[tokio::test]
    async fn nunca_escucha_mcp_en_toda_la_red() {
        assert!(serve_http(state(), "0.0.0.0:0").await.is_err());
    }

    #[tokio::test]
    async fn rechaza_origenes_web_ajenos_y_sesiones_caducadas() {
        let port = serve_http(state(), "127.0.0.1:0").await.unwrap();
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{port}/mcp");
        let response = client
            .post(&url)
            .header("Origin", "https://example.com")
            .json(&json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), 403);
        let response = client
            .post(&url)
            .header("Mcp-Session-Id", "sesion-inexistente")
            .json(&json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), 404);
    }

    #[tokio::test]
    async fn initialize_devuelve_sesion_y_capacidades() {
        let state = state();
        let (payload, headers) = handle_rpc(
            &state,
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","clientInfo":{"name":"test"}}}"#,
            None,
        )
        .await;

        assert_eq!(payload["result"]["protocolVersion"], "2025-06-18");
        assert!(headers.iter().any(|(name, _)| *name == "Mcp-Session-Id"));
    }

    #[tokio::test]
    async fn tools_list_publica_el_catalogo() {
        let state = state();
        let (payload, _) = handle_rpc(
            &state,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
            None,
        )
        .await;
        assert!(payload["result"]["tools"].as_array().unwrap().len() > 0);
    }

    #[tokio::test]
    async fn recursos_publican_el_mismo_contrato_por_http_y_rpc() {
        let state = state();
        let (listed, _) = handle_rpc(&state,
            r#"{"jsonrpc":"2.0","id":1,"method":"resources/list"}"#, None).await;
        assert_eq!(listed["result"]["resources"][0]["uri"], super::super::capabilities::URI);
        let body = json!({"jsonrpc":"2.0","id":2,"method":"resources/read",
            "params":{"uri":super::super::capabilities::URI}});
        let port = serve_http(state.clone(), "127.0.0.1:0").await.unwrap();
        let read: Value = reqwest::Client::new().post(format!("http://127.0.0.1:{port}/mcp"))
            .json(&body).send().await.unwrap().json().await.unwrap();
        let catalog: Value = serde_json::from_str(read["result"]["contents"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(catalog, super::super::capabilities::snapshot(&state.tools));
        let (unknown, _) = handle_rpc(&state,
            r#"{"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"file:///secret"}}"#, None).await;
        assert_eq!(unknown["error"]["code"], INVALID_PARAMS);
    }

    #[tokio::test]
    async fn capacidades_respetan_la_clave_del_puente() {
        let mut state = state();
        Arc::get_mut(&mut state).unwrap().api_key = Some("test-only-key".into());
        let port = serve_http(state, "127.0.0.1:0").await.unwrap();
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{port}/mcp");
        let body = json!({"jsonrpc":"2.0","id":1,"method":"resources/list"});
        assert_eq!(client.post(&url).json(&body).send().await.unwrap().status().as_u16(), 401);
        assert_eq!(client.post(&url).bearer_auth("test-only-key").json(&body).send().await.unwrap().status().as_u16(), 200);
    }

    #[tokio::test]
    async fn una_herramienta_inexistente_es_error_de_parametros() {
        let state = state();
        let (payload, _) = handle_rpc(
            &state,
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"no_existe","arguments":{}}}"#,
            None,
        )
        .await;
        assert_eq!(payload["error"]["code"], INVALID_PARAMS);
    }

    #[tokio::test]
    async fn las_notificaciones_no_producen_respuesta() {
        let state = state();
        let (payload, _) = handle_rpc(
            &state,
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            None,
        )
        .await;
        assert!(payload.is_null());
    }

    // ── Ciclo de vida de la sesión ─────────────────────────────────────────

    #[tokio::test]
    async fn una_peticion_valida_renueva_y_una_rechazada_no() {
        let state = state();
        let id = state.sessions.create("cliente", protocol::PROTOCOL_VERSION);

        // Petición mal formada: se rechaza y NO cuenta como actividad.
        assert!(state.sessions.backdate(&id, Duration::from_secs(120)));
        let (payload, _) = handle_rpc(
            &state,
            r#"{"jsonrpc":"1.0","id":1,"method":"ping"}"#,
            Some(id.clone()),
        )
        .await;
        assert_eq!(payload["error"]["code"], protocol::INVALID_REQUEST);
        assert!(state.sessions.get(&id).unwrap().idle_seconds() >= 120);

        // Petición válida: renueva la actividad.
        let (payload, _) = handle_rpc(
            &state,
            r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#,
            Some(id.clone()),
        )
        .await;
        assert!(payload["error"].is_null());
        assert!(state.sessions.get(&id).unwrap().idle_seconds() < 5);
    }

    #[tokio::test]
    async fn dos_clientes_no_comparten_sesion() {
        let state = state();
        let (first, _) = handle_rpc(
            &state,
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","clientInfo":{"name":"agente-terminal"}}}"#,
            None,
        )
        .await;
        let (second, _) = handle_rpc(
            &state,
            r#"{"jsonrpc":"2.0","id":2,"method":"initialize","params":{"protocolVersion":"2025-06-18","clientInfo":{"name":"Ely Intelligence"}}}"#,
            None,
        )
        .await;
        assert!(first["result"]["serverInfo"].is_object());
        assert!(second["result"]["serverInfo"].is_object());

        let listado = state.sessions.list();
        let mut clientes: Vec<&str> = listado
            .iter()
            .map(|meta| meta.client.as_str())
            .collect();
        clientes.sort_unstable();
        assert_eq!(clientes, vec!["Ely Intelligence", "agente-terminal"]);
    }

    #[tokio::test]
    async fn reinitialize_no_deja_sesiones_fantasma() {
        let state = state();
        let (_, headers) = handle_rpc(
            &state,
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","clientInfo":{"name":"agente"}}}"#,
            None,
        )
        .await;
        let primera = headers
            .iter()
            .find(|(name, _)| *name == "Mcp-Session-Id")
            .map(|(_, value)| value.clone())
            .unwrap();
        assert_eq!(state.sessions.list().len(), 1);

        // El cliente reenvía su cabecera al reinitialize: la sesión anterior se
        // retira en lugar de quedar viva hasta caducar.
        let (_, headers) = handle_rpc(
            &state,
            r#"{"jsonrpc":"2.0","id":2,"method":"initialize","params":{"protocolVersion":"2025-06-18","clientInfo":{"name":"agente"}}}"#,
            Some(primera.clone()),
        )
        .await;
        let segunda = headers
            .iter()
            .find(|(name, _)| *name == "Mcp-Session-Id")
            .map(|(_, value)| value.clone())
            .unwrap();
        assert_ne!(primera, segunda);
        assert_eq!(state.sessions.list().len(), 1);
        assert!(!state.sessions.exists(&primera));
        assert!(state.sessions.exists(&segunda));
    }

    #[tokio::test]
    async fn un_404_explica_como_reiniciar() {
        let state = state();
        let port = serve_http(state.clone(), "127.0.0.1:0").await.unwrap();
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{port}/mcp");
        let response = client
            .post(&url)
            .header("Mcp-Session-Id", "syn-mcp-inexistente")
            .json(&json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), 404);
        let body: Value = response.json().await.unwrap();
        assert_eq!(body["error"], "session_not_found");
        assert!(body["detail"].as_str().unwrap().contains("initialize"));
    }

    #[tokio::test]
    async fn initialize_reabre_la_sesion_tras_un_404() {
        let state = state();
        let port = serve_http(state.clone(), "127.0.0.1:0").await.unwrap();
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{port}/mcp");

        // El cliente conserva la cabecera de la sesión caducada y reintenta.
        let response = client
            .post(&url)
            .header("Mcp-Session-Id", "syn-mcp-caducada")
            .json(&json!({
                "jsonrpc":"2.0","id":1,"method":"initialize",
                "params":{"protocolVersion":"2025-06-18","clientInfo":{"name":"reconector"}}
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), 200);
        let nueva = response
            .headers()
            .get("mcp-session-id")
            .expect("el initialize debe devolver una sesión nueva")
            .to_str()
            .unwrap()
            .to_string();
        assert_ne!(nueva, "syn-mcp-caducada");

        // Con la sesión nueva el cliente sigue trabajando.
        let response = client
            .post(&url)
            .header("Mcp-Session-Id", &nueva)
            .json(&json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), 200);
        let body: Value = response.json().await.unwrap();
        assert!(body["result"]["tools"].as_array().unwrap().len() > 0);
    }

    /// Las instrucciones se inyectan en el prompt del modelo.
    ///
    /// Si prometen una app que no tiene herramientas, el agente intenta usarla
    /// y falla. Este test ata el texto a la realidad del catálogo.
    #[test]
    fn las_instrucciones_solo_prometen_apps_con_herramientas() {
        let con_herramientas: Vec<&str> = crate::mcp::routes::BRIDGE_TARGETS
            .iter()
            .filter(|(_, _, _, tools)| !tools.is_empty())
            .map(|(_, name, _, _)| *name)
            .collect();

        for nombre in &con_herramientas {
            assert!(
                INSTRUCTIONS.contains(nombre),
                "las instrucciones no mencionan {nombre}, que sí tiene herramientas"
            );
        }

        // La lista final de apps sin superficie debe coincidir exactamente con
        // las que el catálogo declara sin herramientas.
        let marcador = "sin superficie de invocación todavía, por tanto sin herramientas aquí:";
        let cola = INSTRUCTIONS
            .split(marcador)
            .nth(1)
            .expect("las instrucciones declaran la lista de apps sin superficie");
        let declaradas: Vec<&str> = cola
            .trim_end_matches('.')
            .split(", ")
            .map(|nombre| nombre.trim())
            .collect();

        let sin_herramientas: Vec<&str> = crate::mcp::routes::BRIDGE_TARGETS
            .iter()
            .filter(|(_, _, _, tools)| tools.is_empty())
            .map(|(_, name, _, _)| *name)
            .collect();

        assert_eq!(declaradas, sin_herramientas);
    }
}
