// El receptor LAN y la validación del canal emparejado corren en el PC. En
// Android este módulo conserva el contrato compartido y las funciones del QR,
// pero el receptor no se compila en runtime: las funciones de solo escritorio
// no se marcan como error, solo se omiten.
#![cfg_attr(target_os = "android", allow(dead_code))]

//! Vinculación explícita teléfono ↔ PC y receptor LAN autenticado.
//!
//! El QR es efímero y nunca contiene claves MCP. El receptor atiende tres
//! cosas distintas, con reglas separadas: intercambio de invitación, consulta
//! de estado de solo lectura y despacho de una orden hacia una app del PC.
//! Ninguna orden se ejecuta sin la credencial del teléfono; el beacon solo
//! acredita que un dispositivo existe, nunca que esté autorizado.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock,
    },
    time::{Duration, Instant},
};

static LISTENING: AtomicBool = AtomicBool::new(false);
struct Invitation { ticket: String, expires: Instant }
struct Peer { name: String, expires: Instant }
#[derive(Default)]
struct Pairing { invitation: Option<Invitation>, peers: HashMap<String, Peer>, connected: Option<String> }
static STATE: OnceLock<Mutex<Pairing>> = OnceLock::new();
fn state() -> &'static Mutex<Pairing> { STATE.get_or_init(|| Mutex::new(Pairing::default())) }
pub fn listening() { LISTENING.store(true, Ordering::Relaxed); }
fn random() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).map_err(|_| "No se pudo generar una conexión segura")?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
fn digest(token: &str) -> String { format!("{:x}", Sha256::digest(token.as_bytes())) }
pub async fn read_request(socket: &mut tokio::net::TcpStream) -> Result<String, String> {
    use tokio::io::AsyncReadExt;
    tokio::time::timeout(Duration::from_secs(10), async {
        let mut bytes = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let count = socket.read(&mut chunk).await.map_err(|e| e.to_string())?;
            if count == 0 { return Err("Solicitud incompleta".into()); }
            bytes.extend_from_slice(&chunk[..count]);
            if bytes.len() > 65536 { return Err("Solicitud demasiado grande".into()); }
            if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                let headers = std::str::from_utf8(&bytes[..end]).map_err(|_| "Cabeceras inválidas")?;
                let mut length = None;
                for line in headers.lines().skip(1) {
                    if let Some((key,value)) = line.split_once(':') {
                        if key.eq_ignore_ascii_case("transfer-encoding") { return Err("Codificación no admitida".into()); }
                        if key.eq_ignore_ascii_case("content-length") {
                            if length.is_some() { return Err("Tamaño duplicado".into()); }
                            length = Some(value.trim().parse::<usize>().map_err(|_| "Tamaño inválido")?);
                        }
                    }
                }
                let total = (end+4).checked_add(length.unwrap_or(0)).filter(|n| *n <= 65536).ok_or("Solicitud demasiado grande")?;
                if bytes.len() >= total { return String::from_utf8(bytes[..total].to_vec()).map_err(|_| "Solicitud inválida".into()); }
            }
        }
    }).await.map_err(|_| "Solicitud caducada")?
}
pub fn pc_name() -> String { std::env::var("COMPUTERNAME").unwrap_or_else(|_| "PC Synapse".into()) }

pub fn create() -> Result<Value, String> {
    if !LISTENING.load(Ordering::Relaxed) { return Err("El receptor LAN de este Synapse no está activo".into()); }
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    socket.connect("192.168.1.1:80").map_err(|e| e.to_string())?;
    let ip = socket.local_addr().map_err(|e| e.to_string())?.ip();
    if !crate::mcp::devices::local_address(ip) || ip.is_loopback() { return Err("Conecta el PC a una red local antes de generar el QR".into()); }
    let ticket = random()?;
    let address = format!("http://{ip}:49295");
    let payload = format!("aurora-synapse://pair?v=1&address={}&ticket={ticket}", urlencoding::encode(&address));
    let svg = qrcode::QrCode::new(payload.as_bytes()).map_err(|e| e.to_string())?
        .render::<qrcode::render::svg::Color>().min_dimensions(256, 256).build();
    let mut state = state().lock().map_err(|_| "Conexión ocupada")?;
    state.invitation = Some(Invitation { ticket, expires: Instant::now() + Duration::from_secs(90) });
    state.connected = None;
    Ok(json!({"svg": svg, "address": address, "expires_in": 90}))
}
pub fn cancel() { if let Ok(mut state) = state().lock() { state.invitation = None; } }
pub fn status() -> Value {
    let state = state().lock().unwrap_or_else(|e| e.into_inner());
    let connected = state.connected.as_ref().and_then(|key| state.peers.get(key)).filter(|peer| peer.expires > Instant::now());
    json!({"connected": connected.is_some(), "device_name": connected.map(|peer| &peer.name),
        "expires_in": state.invitation.as_ref().map(|i| i.expires.saturating_duration_since(Instant::now()).as_secs()).unwrap_or(0)})
}
/// Segundos que le quedan a la credencial más reciente que llegó autenticada.
///
/// Es un dato informativo para el teléfono: permite avisar antes de reemparejar
/// en lugar de aparecer desconectado sin explicación.
pub fn linked_remaining() -> Option<u64> {
    let state = state().lock().ok()?;
    let key = state.connected.as_ref()?;
    let peer = state.peers.get(key)?;
    let now = Instant::now();
    (peer.expires > now).then(|| peer.expires.saturating_duration_since(now).as_secs())
}
pub fn claim(ticket: &str, name: &str) -> Result<Value, String> {
    if ticket.len() != 64 || name.is_empty() || name.len() > 80 || name.chars().any(char::is_control) { return Err("Solicitud inválida".into()); }
    let mut state = state().lock().map_err(|_| "Conexión ocupada")?;
    let invitation = state.invitation.as_ref().ok_or("Genera un QR nuevo en el PC")?;
    if invitation.expires <= Instant::now() || invitation.ticket != ticket { return Err("El QR caducó o no corresponde a este PC".into()); }
    let token = random()?;
    state.peers.retain(|_, peer| peer.expires > Instant::now());
    if state.peers.len() >= 16 { return Err("Hay demasiados teléfonos vinculados; reinicia Synapse para desvincularlos".into()); }
    state.peers.insert(digest(&token), Peer { name: name.into(), expires: Instant::now() + Duration::from_secs(12 * 3600) });
    state.invitation = None;
    // Se muestra vinculado solo tras una petición autenticada del teléfono.
    state.connected = None;
    Ok(json!({"success": true, "token": token, "pc_name": pc_name(), "expires_in": 12 * 3600}))
}
pub fn authorize(header: Option<&str>) -> bool {
    let Some(token) = header.and_then(|h| h.strip_prefix("Bearer ")) else { return false; };
    if token.len() != 64 { return false; }
    state().lock().map(|mut state| {
        let key = digest(token);
        let valid = state.peers.get(&key).is_some_and(|peer| peer.expires > Instant::now());
        if valid { state.connected = Some(key); }
        valid
    }).unwrap_or(false)
}

// ─────────────────────────── Enrutado y autorización del receptor LAN ───────────────────────────

/// Rutas que atiende el receptor LAN de Synapse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanRoute {
    /// Intercambio de la invitación QR por una credencial de teléfono.
    PairClaim,
    /// Consulta de estado del PC para un teléfono ya vinculado.
    PairedStatus,
    /// Despacho de una orden hacia una app del PC.
    Dispatch,
    /// Estado informativo del PC. No ejecuta ninguna acción.
    Status,
    /// Sondeo CORS. No ejecuta ninguna acción.
    Options,
    /// Ruta desconocida.
    Unknown,
}

/// Ruta canónica del despacho hacia el PC.
///
/// Las rutas heredadas `/synapse` y `/dispatch` ejecutan exactamente lo mismo,
/// pero se marcan como obsoletas: un cliente que ya tiene credencial sigue
/// funcionando por ellas y recibe el aviso de migración en la respuesta.
pub const CANONICAL_DISPATCH_ROUTE: &str = "/paired/dispatch";

/// Rutas heredadas que ejecutaban el despacho sin autenticación.
pub fn is_legacy_dispatch_route(path: &str) -> bool {
    matches!(path, "/synapse" | "/dispatch")
}

/// Petición tal y como llega al receptor LAN.
#[derive(Debug, Clone, Copy)]
pub struct LanRequest<'a> {
    pub method: &'a str,
    pub path: &'a str,
    /// Cabecera `Origin`, presente cuando la petición viene de un navegador.
    pub origin: Option<&'a str>,
    /// Si el par.address está en la red local.
    pub local_peer: bool,
    /// Cabecera `Authorization` con la credencial del teléfono.
    pub authorization: Option<&'a str>,
}

/// Decisión del receptor frente a una petición.
#[derive(Debug, Clone, PartialEq)]
pub enum LanDecision {
    /// Ejecutar la ruta indicada.
    Allow(LanRoute),
    /// Responder sin ejecutar nada.
    Reject { status: u16, body: Value },
}

/// Decide la ruta y la autorización de una petición del receptor LAN.
///
/// Reglas, en orden:
/// 1. `OPTIONS` es siempre un sondeo sin efectos.
/// 2. Solo se atiende a direcciones de la red local.
/// 3. `GET /status` es informativo y no ejecuta acciones.
/// 4. El intercambio de invitación y las rutas del teléfono emparejado no
///    aceptan orígenes web: un navegador no puede suplantar al teléfono.
/// 5. El despacho exige la credencial del teléfono, también en las rutas
///    heredadas. Un cliente sin credencial no ejecuta acciones: recibe un
///    `401` con la ruta canónica y el motivo.
pub fn decide(request: LanRequest) -> LanDecision {
    let LanRequest { method, path, origin, local_peer, authorization } = request;

    if method == "OPTIONS" {
        return LanDecision::Allow(LanRoute::Options);
    }
    if !local_peer {
        return LanDecision::Reject {
            status: 403,
            body: json!({"success": false, "error": "Solo se atiende a direcciones de la red local"}),
        };
    }

    let route = match (method, path) {
        ("POST", "/pair/claim") => LanRoute::PairClaim,
        ("GET", "/paired/status") => LanRoute::PairedStatus,
        ("POST", "/paired/dispatch") | ("POST", "/synapse") | ("POST", "/dispatch") => LanRoute::Dispatch,
        ("GET", "/status") => LanRoute::Status,
        _ => LanRoute::Unknown,
    };

    match route {
        LanRoute::Options | LanRoute::Status => LanDecision::Allow(route),
        LanRoute::Unknown => LanDecision::Reject {
            status: 404,
            body: json!({"success": false, "error": "Ruta desconocida"}),
        },
        LanRoute::PairClaim => {
            if origin.is_some() {
                return LanDecision::Reject {
                    status: 403,
                    body: json!({"success": false, "error": "No se aceptan orígenes web"}),
                };
            }
            LanDecision::Allow(route)
        }
        LanRoute::PairedStatus | LanRoute::Dispatch => {
            if origin.is_some() {
                return LanDecision::Reject {
                    status: 403,
                    body: json!({"success": false, "error": "No se aceptan orígenes web"}),
                };
            }
            if !authorize(authorization) {
                return LanDecision::Reject {
                    status: 401,
                    body: json!({
                        "success": false,
                        "error": "Falta la credencial del teléfono emparejado",
                        "canonical_route": CANONICAL_DISPATCH_ROUTE,
                        "legacy_route": is_legacy_dispatch_route(path),
                        "hint": "Escanea de nuevo el QR del PC para vincular este teléfono",
                    }),
                };
            }
            LanDecision::Allow(route)
        }
    }
}

// ─────────────────────────── Recepción HTTP y respuestas ───────────────────────────

/// Protocolo del canal emparejado. Se publica para que el teléfono y el frente
/// MCP distingan esta versión de las rutas LAN heredadas.
pub const PAIRED_PROTOCOL: &str = "synapse-lan-v1";

/// Futuro que resuelve la orden de una app ya identificada.
///
/// El receptor no conoce las apps: recibe la orden y devuelve el mensaje real
/// de la capa que sí las conoce. Así el mismo receptor sirve en pruebas con un
/// despachador falso, sin ejecutar nada del ecosistema.
pub type DispatchFuture = Pin<Box<dyn Future<Output = Result<String, String>> + Send>>;
pub type Dispatcher = Arc<dyn Fn(&str, &str) -> DispatchFuture + Send + Sync>;

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        _ => "Response",
    }
}

fn split_request(request: &str) -> (&str, &str, &str, &str) {
    let (head, body) = request.split_once("\r\n\r\n").unwrap_or((request, ""));
    let mut parts = head.lines().next().unwrap_or("").split_whitespace();
    (parts.next().unwrap_or(""), parts.next().unwrap_or(""), head, body)
}

fn header<'a>(head: &'a str, name: &str) -> Option<&'a str> {
    head.lines()
        .skip(1)
        .filter_map(|line| line.split_once(':'))
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.trim())
}

fn status_body() -> Value {
    json!({"status": "online", "app": "Aurora Synapse", "version": env!("CARGO_PKG_VERSION"), "protocol": "synapse-v1", "paired_protocol": PAIRED_PROTOCOL})
}

fn paired_status_body() -> Value {
    json!({"success": true, "pc_name": pc_name(), "linked": true, "expires_in": linked_remaining(), "protocol": PAIRED_PROTOCOL})
}

fn dispatch_arguments(body: &str) -> Result<(String, String), String> {
    let payload: Value = serde_json::from_str(body).map_err(|_| "Solicitud inválida".to_string())?;
    // `app`/`url` son los alias que aceptaba el receptor heredado: un cliente
    // existente con credencial sigue funcionando sin cambiar su cuerpo.
    let app_id = payload["app_id"]
        .as_str()
        .or_else(|| payload["app"].as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if app_id.is_empty()
        || app_id.len() > 64
        || !app_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err("Falta app_id o no es un identificador válido".into());
    }
    let content = payload["content"]
        .as_str()
        .or_else(|| payload["url"].as_str())
        .unwrap_or("")
        .to_string();
    if content.len() > 8192 {
        return Err("El contenido supera el máximo del canal emparejado".into());
    }
    Ok((app_id, content))
}

/// Resultado de una orden hacia una app del PC.
///
/// `accepted` significa que el PC recibió la orden y la entregó al sistema; no
/// acredita que la ventana de la app esté visible (`verified: false`). Un fallo
/// de la app se devuelve como resultado de la orden, no como fallo de red.
fn dispatch_body(app_id: &str, legacy: bool, result: Result<String, String>) -> Value {
    match result {
        Ok(message) => json!({
            "success": true, "status": "ok", "message": message, "app_id": app_id,
            "delivery": "accepted", "verified": false,
            "canonical_route": CANONICAL_DISPATCH_ROUTE, "deprecated": legacy,
        }),
        Err(error) => json!({
            "success": false, "status": "error", "message": error, "app_id": app_id,
            "delivery": "rejected", "verified": false,
            "canonical_route": CANONICAL_DISPATCH_ROUTE, "deprecated": legacy,
        }),
    }
}

async fn respond_text(socket: &mut tokio::net::TcpStream, status: u16, content_type: &str, body: &str, cors: bool) {
    use tokio::io::AsyncWriteExt;
    let cors = if cors { "Access-Control-Allow-Origin: *\r\n" } else { "" };
    let response = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\n{cors}Connection: close\r\n\r\n{body}",
        reason(status),
        body.len()
    );
    let _ = socket.write_all(response.as_bytes()).await;
}

async fn respond_json(socket: &mut tokio::net::TcpStream, status: u16, body: Value, cors: bool) {
    respond_text(socket, status, "application/json", &body.to_string(), cors).await;
}

/// Sondeo CORS sin efectos. Conserva el contrato que ya usaban los clientes.
async fn respond_options(socket: &mut tokio::net::TcpStream) {
    use tokio::io::AsyncWriteExt;
    let response = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, Authorization, *\r\nContent-Length: 0\r\n\r\n";
    let _ = socket.write_all(response.as_bytes()).await;
}

/// Atiende el receptor LAN de Synapse.
///
/// Cada conexión se decide con [`decide`] y se ejecuta una sola vez. No hay
/// reintentos ni rutas alternativas: una orden autorizada produce exactamente
/// una llamada al despachador y un rechazo no produce ninguna.
pub async fn serve(listener: tokio::net::TcpListener, dispatch: Dispatcher) {
    while let Ok((mut socket, address)) = listener.accept().await {
        let dispatch = Arc::clone(&dispatch);
        tokio::spawn(async move {
            let Ok(request) = read_request(&mut socket).await else { return; };
            let (method, path, head, body) = split_request(&request);
            let decision = decide(LanRequest {
                method,
                path,
                origin: header(head, "origin"),
                local_peer: crate::mcp::devices::local_address(address.ip()),
                authorization: header(head, "authorization"),
            });
            match decision {
                LanDecision::Reject { status, body } => respond_json(&mut socket, status, body, false).await,
                LanDecision::Allow(LanRoute::Options) => respond_options(&mut socket).await,
                LanDecision::Allow(LanRoute::Status) => respond_json(&mut socket, 200, status_body(), true).await,
                LanDecision::Allow(LanRoute::PairClaim) => {
                    let result = serde_json::from_str::<Value>(body)
                        .map_err(|_| "Solicitud inválida".to_string())
                        .and_then(|payload| {
                            claim(
                                payload["ticket"].as_str().unwrap_or(""),
                                payload["device_name"].as_str().unwrap_or(""),
                            )
                        });
                    match result {
                        Ok(body) => respond_json(&mut socket, 200, body, false).await,
                        Err(error) => respond_json(&mut socket, 400, json!({"success": false, "error": error}), false).await,
                    }
                }
                LanDecision::Allow(LanRoute::PairedStatus) => {
                    respond_json(&mut socket, 200, paired_status_body(), false).await
                }
                LanDecision::Allow(LanRoute::Dispatch) => {
                    let legacy = is_legacy_dispatch_route(path);
                    match dispatch_arguments(body) {
                        Ok((app_id, content)) => {
                            let result = dispatch(&app_id, &content).await;
                            respond_json(&mut socket, 200, dispatch_body(&app_id, legacy, result), true).await;
                        }
                        // La orden está mal formada: es un error del cliente, no
                        // un resultado de la app. No se llamó al despachador.
                        Err(error) => respond_json(
                            &mut socket,
                            400,
                            json!({
                                "success": false, "status": "error", "error": error,
                                "delivery": "rejected", "canonical_route": CANONICAL_DISPATCH_ROUTE,
                            }),
                            true,
                        ).await,
                    }
                }
                LanDecision::Allow(LanRoute::Unknown) => {
                    respond_json(&mut socket, 404, json!({"success": false, "error": "Ruta desconocida"}), false).await
                }
            }
        });
    }
}

#[cfg(target_os = "android")]
fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder().no_proxy().redirect(reqwest::redirect::Policy::none()).timeout(Duration::from_secs(10)).build().map_err(|e| e.to_string())
}
/// Interpreta el QR de invitación del PC.
///
/// Solo lo usa el teléfono; en escritorio se conserva compilado para las
/// pruebas del contrato.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub fn parse_invitation(raw: &str) -> Result<(String, String), String> {
    if raw.len() > 1024 { return Err("QR inválido".into()); }
    let qr = reqwest::Url::parse(raw).map_err(|_| "QR inválido")?;
    if qr.scheme() != "aurora-synapse" || qr.host_str() != Some("pair") || !qr.username().is_empty()
        || qr.password().is_some() || qr.fragment().is_some() || !matches!(qr.path(), "" | "/") { return Err("Este QR no es de conexión con Synapse".into()); }
    let fields: Vec<_> = qr.query_pairs().collect();
    if fields.len() != 3 || fields.iter().filter(|(k,v)| k == "v" && v == "1").count() != 1 { return Err("Versión de QR no compatible".into()); }
    let address = fields.iter().find(|(k,_)| k == "address").ok_or("Falta dirección")?.1.to_string();
    let url = crate::mcp::gallery::validate_url(&address)?;
    if url.port() != Some(49295) { return Err("El QR debe apuntar al receptor local de Synapse".into()); }
    let ticket = fields.iter().find(|(k,_)| k == "ticket").ok_or("Falta invitación")?.1.to_string();
    if ticket.len() != 64 || !ticket.bytes().all(|b| b.is_ascii_hexdigit()) { return Err("Invitación inválida".into()); }
    Ok((url.to_string().trim_end_matches('/').into(), ticket))
}

#[cfg(target_os = "android")]
type Store = std::sync::Arc<dyn Fn(Option<String>) -> Result<String, String> + Send + Sync>;
#[cfg(target_os = "android")]
static STORE: OnceLock<Store> = OnceLock::new();
#[cfg(target_os = "android")]
pub fn set_store(store: Store) { let _ = STORE.set(store); }
#[cfg(target_os = "android")]
fn load() -> Result<Value, String> {
    serde_json::from_str(&STORE.get().ok_or("Almacén Android no disponible")?(None)?).map_err(|_| "Escanea el QR del PC para conectar".into())
}
#[cfg(target_os = "android")]
pub async fn connect(raw: &str) -> Result<Value, String> {
    let (address, ticket) = parse_invitation(raw)?;
    let response = client()?.post(format!("{address}/pair/claim")).json(&json!({"ticket": ticket, "device_name": "Synapse Android"})).send().await.map_err(|_| "No responde el PC; comprueba la red local y el firewall")?;
    if !response.status().is_success() { return Err("El PC rechazó el QR; genera uno nuevo".into()); }
    let response: Value = response.json().await.map_err(|_| "Respuesta del PC inválida")?;
    let token = response["token"].as_str().filter(|s| s.len() == 64).ok_or("El PC no confirmó la conexión")?;
    STORE.get().ok_or("Almacén Android no disponible")?(Some(json!({"address": address, "token": token, "pc_name": response["pc_name"]}).to_string()))?;
    connected_status().await
}
#[cfg(target_os = "android")]
pub async fn connected_status() -> Result<Value, String> {
    let config = load()?;
    let address = config["address"].as_str().ok_or("Dirección inválida")?;
    let url = crate::mcp::gallery::validate_url(address)?;
    let response = client()?.get(url.join("paired/status").unwrap()).bearer_auth(config["token"].as_str().ok_or("Conexión inválida")?).send().await.map_err(|_| "PC sin conexión")?;
    if !response.status().is_success() { return Err("La conexión caducó; escanea un QR nuevo".into()); }
    let response: Value = response.json().await.map_err(|_| "Respuesta del PC inválida")?;
    if response["success"] != true { return Err("El PC no confirmó la conexión".into()); }
    Ok(json!({"connected": true, "pc_name": response["pc_name"], "ip": url.host_str(), "address": address, "expires_in": response["expires_in"]}))
}
#[cfg(target_os = "android")]
pub async fn dispatch(app_id: &str, content: &str, ip: &str) -> Result<Option<String>, String> {
    let Ok(config) = load() else { return Ok(None); };
    let address = config["address"].as_str().ok_or("Dirección inválida")?;
    let url = crate::mcp::gallery::validate_url(address)?;
    // El destino del canal emparejado es el PC vinculado: la dirección manual
    // que aparece en Ajustes describe la ruta heredada, que ya no ejecuta
    // acciones sin credencial. Si el usuario escribió otra IP, se le dice cuál
    // fue el destino real en lugar de enviar en silencio a otro equipo.
    let requested = ip.trim();
    let diverges = !requested.is_empty() && url.host_str() != Some(requested);
    let token = config["token"].as_str().ok_or("Conexión inválida")?;
    let response = client()?.post(url.join("paired/dispatch").unwrap()).bearer_auth(token)
        .json(&json!({"app_id": app_id, "content": content})).send().await.map_err(|_| "No responde el PC vinculado")?;
    if response.status() == 401 {
        return Err("El PC no reconoce esta conexión: escanea de nuevo su QR".into());
    }
    if !response.status().is_success() { return Err("No se pudo enviar al PC vinculado; comprueba la conexión QR".into()); }
    let response: Value = response.json().await.map_err(|_| "Respuesta del PC inválida")?;
    if response["status"] != "ok" { return Err(response["message"].as_str().unwrap_or("El PC rechazó la operación").into()); }
    let mut message = response["message"].as_str().unwrap_or("Recibido en el PC").to_string();
    if diverges {
        message.push_str(&format!(" (destino real: {address})"));
    }
    Ok(Some(message))
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Serializa las pruebas que mutan el estado global de emparejamiento.
    static STATE_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn invitation_is_one_use_and_requires_bearer() {
        let _guard = STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let ticket = random().unwrap();
        state().lock().unwrap().invitation = Some(Invitation { ticket: ticket.clone(), expires: Instant::now() + Duration::from_secs(60) });
        assert!(claim(&"0".repeat(64), "Phone").is_err());
        let accepted = claim(&ticket, "Phone").unwrap();
        assert!(claim(&ticket, "Phone").is_err());
        assert!(!authorize(None));
        assert!(authorize(Some(&format!("Bearer {}", accepted["token"].as_str().unwrap()))));
        state().lock().unwrap().invitation = Some(Invitation { ticket: ticket.clone(), expires: Instant::now() - Duration::from_secs(1) });
        assert!(claim(&ticket, "Phone").is_err());
    }
    #[test]
    fn qr_only_accepts_our_protocol_and_private_receiver() {
        let ticket = "a".repeat(64);
        let raw = format!("aurora-synapse://pair?v=1&address=http%3A%2F%2F192.168.1.4%3A49295&ticket={ticket}");
        assert!(parse_invitation(&raw).is_ok());
        assert!(parse_invitation(&raw.replace("192.168.1.4", "8.8.8.8")).is_err());
        assert!(parse_invitation(&raw.replace("49295", "49296")).is_err());
        assert!(parse_invitation(&(raw.clone()+"&v=1")).is_err());
        assert!(parse_invitation(&raw.replace("aurora-synapse", "https")).is_err());
    }
    #[tokio::test]
    async fn reads_fragmented_body_and_rejects_ambiguous_lengths() {
        use tokio::io::AsyncWriteExt;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let writer = tokio::spawn(async move {
            let mut client = tokio::net::TcpStream::connect(("127.0.0.1", port)).await.unwrap();
            client.write_all(b"POST /pair/claim HTTP/1.1\r\nContent-Length: 2\r\n\r\n{").await.unwrap();
            tokio::task::yield_now().await;
            client.write_all(b"}").await.unwrap();
        });
        let (mut socket, _) = listener.accept().await.unwrap();
        assert!(read_request(&mut socket).await.unwrap().ends_with("{}"));
        writer.await.unwrap();
        let mut client = tokio::net::TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        client.write_all(b"POST /pair/claim HTTP/1.1\r\nContent-Length: 0\r\nContent-Length: 2\r\n\r\n{}").await.unwrap();
        let (mut socket, _) = listener.accept().await.unwrap();
        assert!(read_request(&mut socket).await.is_err());
    }

    fn request<'a>(method: &'a str, path: &'a str) -> LanRequest<'a> {
        LanRequest { method, path, origin: None, local_peer: true, authorization: None }
    }

    #[test]
    fn el_receptor_solo_atiende_a_la_red_local() {
        let decision = decide(LanRequest { local_peer: false, ..request("GET", "/status") });
        assert_eq!(decision, LanDecision::Reject { status: 403, body: serde_json::json!({"success": false, "error": "Solo se atiende a direcciones de la red local"}) });
    }

    #[test]
    fn el_sondeo_y_el_estado_no_exigen_credencial() {
        assert_eq!(decide(request("OPTIONS", "/paired/dispatch")), LanDecision::Allow(LanRoute::Options));
        assert_eq!(decide(request("GET", "/status")), LanDecision::Allow(LanRoute::Status));
    }

    #[test]
    fn las_rutas_del_telefono_no_aceptan_origen_web() {
        let decision = decide(LanRequest { origin: Some("http://ejemplo.test"), ..request("POST", "/pair/claim") });
        assert!(matches!(decision, LanDecision::Reject { status: 403, .. }));
        let decision = decide(LanRequest { origin: Some("http://ejemplo.test"), ..request("GET", "/paired/status") });
        assert!(matches!(decision, LanDecision::Reject { status: 403, .. }));
    }

    #[test]
    fn el_despacho_exige_credencial_tambien_en_las_rutas_heredadas() {
        for path in ["/paired/dispatch", "/synapse", "/dispatch"] {
            let decision = decide(request("POST", path));
            match decision {
                LanDecision::Reject { status, body } => {
                    assert_eq!(status, 401);
                    assert_eq!(body["success"], false);
                    assert_eq!(body["canonical_route"], CANONICAL_DISPATCH_ROUTE);
                    assert_eq!(body["legacy_route"], is_legacy_dispatch_route(path));
                }
                other => panic!("{path} no debe ejecutar sin credencial: {other:?}"),
            }
        }
    }

    #[test]
    fn con_credencial_valida_el_despacho_por_cualquier_ruta() {
        let _guard = STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let ticket = random().unwrap();
        state().lock().unwrap().invitation = Some(Invitation { ticket: ticket.clone(), expires: Instant::now() + Duration::from_secs(60) });
        let accepted = claim(&ticket, "Phone").unwrap();
        let token = format!("Bearer {}", accepted["token"].as_str().unwrap());
        for path in ["/paired/dispatch", "/synapse", "/dispatch"] {
            let decision = decide(LanRequest { authorization: Some(&token), ..request("POST", path) });
            assert_eq!(decision, LanDecision::Allow(LanRoute::Dispatch), "{path} con credencial válida");
        }
        let expired = format!("Bearer {}", "a".repeat(64));
        let decision = decide(LanRequest { authorization: Some(&expired), ..request("POST", "/paired/dispatch") });
        assert!(matches!(decision, LanDecision::Reject { status: 401, .. }));
    }

    #[test]
    fn una_ruta_desconocida_no_ejecuta_nada() {
        let decision = decide(request("POST", "/apagar"));
        assert!(matches!(decision, LanDecision::Reject { status: 404, .. }));
    }

    // ── Receptor HTTP real: comprueba la transición de las rutas LAN ──

    fn pairing_token() -> String {
        let _guard = STATE_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        let ticket = random().unwrap();
        state().lock().unwrap().invitation =
            Some(Invitation { ticket: ticket.clone(), expires: Instant::now() + Duration::from_secs(60) });
        let accepted = claim(&ticket, "Phone").unwrap();
        accepted["token"].as_str().unwrap().to_string()
    }

    fn recording(calls: Arc<Mutex<Vec<(String, String)>>>, error: Option<&'static str>) -> Dispatcher {
        Arc::new(move |app_id: &str, content: &str| {
            let calls = Arc::clone(&calls);
            let app_id = app_id.to_string();
            let content = content.to_string();
            let future: DispatchFuture = Box::pin(async move {
                calls.lock().unwrap_or_else(|lock| lock.into_inner()).push((app_id.clone(), content));
                match error {
                    Some(error) => Err(error.to_string()),
                    None => Ok(format!("{app_id} recibió la orden")),
                }
            });
            future
        })
    }

    async fn receiver(error: Option<&'static str>) -> (std::net::SocketAddr, Arc<Mutex<Vec<(String, String)>>>, tokio::task::JoinHandle<()>) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(serve(listener, recording(Arc::clone(&calls), error)));
        (address, calls, task)
    }

    async fn http(address: std::net::SocketAddr, raw: &str) -> (u16, String) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut socket = tokio::net::TcpStream::connect(address).await.unwrap();
        socket.write_all(raw.as_bytes()).await.unwrap();
        let mut data = Vec::new();
        socket.read_to_end(&mut data).await.unwrap();
        let text = String::from_utf8(data).unwrap();
        let status = text.lines().next().and_then(|line| line.split_whitespace().nth(1)).unwrap().parse().unwrap();
        let body = text.split_once("\r\n\r\n").map(|(_, body)| body.to_string()).unwrap_or_default();
        (status, body)
    }

    fn order(path: &str, authorization: Option<&str>, body: &str) -> String {
        let auth = authorization.map(|token| format!("Authorization: Bearer {token}\r\n")).unwrap_or_default();
        format!("POST {path} HTTP/1.1\r\nHost: receptor\r\nContent-Type: application/json\r\n{auth}Content-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
    }

    #[tokio::test]
    async fn el_receptor_http_exige_credencial_y_ejecuta_una_sola_vez() {
        let token = pairing_token();
        let (address, calls, task) = receiver(None).await;
        let body = r#"{"app_id":"prisma","content":"https://ejemplo.test/video"}"#;

        // Las rutas heredadas ya no ejecutan nada sin la credencial del teléfono.
        for path in ["/paired/dispatch", "/synapse", "/dispatch"] {
            let (status, response) = http(address, &order(path, None, body)).await;
            assert_eq!(status, 401, "{path} no debe ejecutar sin credencial");
            let response: Value = serde_json::from_str(&response).unwrap();
            assert_eq!(response["canonical_route"], CANONICAL_DISPATCH_ROUTE);
            assert_eq!(response["legacy_route"], is_legacy_dispatch_route(path));
        }
        assert!(calls.lock().unwrap_or_else(|lock| lock.into_inner()).is_empty(), "ninguna orden puede ejecutarse sin credencial");

        // Con credencial: una llamada por orden y resultado real de la app.
        for (path, deprecated) in [("/paired/dispatch", false), ("/synapse", true), ("/dispatch", true)] {
            let (status, response) = http(address, &order(path, Some(&token), body)).await;
            assert_eq!(status, 200, "{path}");
            let response: Value = serde_json::from_str(&response).unwrap();
            assert_eq!(response["status"], "ok");
            assert_eq!(response["app_id"], "prisma");
            assert_eq!(response["delivery"], "accepted");
            assert_eq!(response["verified"], false);
            assert_eq!(response["deprecated"], deprecated, "{path}");
        }
        let recorded = calls.lock().unwrap_or_else(|lock| lock.into_inner()).clone();
        assert_eq!(recorded.len(), 3, "cada petición autorizada produce exactamente una orden");
        assert!(recorded.iter().all(|(app, content)| app == "prisma" && content == "https://ejemplo.test/video"));

        // Un cliente heredado que ya tiene credencial conserva sus alias
        // `app`/`url`; no se le obliga a cambiar el cuerpo para seguir usando
        // la ruta antigua.
        let legacy_body = r#"{"app":"luna","url":"https://ejemplo.test/audio"}"#;
        let (status, response) = http(address, &order("/synapse", Some(&token), legacy_body)).await;
        assert_eq!(status, 200);
        let response: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response["app_id"], "luna");
        assert_eq!(response["deprecated"], true);
        let recorded = calls.lock().unwrap_or_else(|lock| lock.into_inner()).clone();
        assert_eq!(recorded.last(), Some(&("luna".to_string(), "https://ejemplo.test/audio".to_string())));
        task.abort();
    }

    #[tokio::test]
    async fn el_receptor_http_completa_el_intercambio_y_el_estado() {
        let guard = STATE_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        let ticket = random().unwrap();
        state().lock().unwrap().invitation =
            Some(Invitation { ticket: ticket.clone(), expires: Instant::now() + Duration::from_secs(60) });
        let (address, calls, task) = receiver(None).await;

        let claim_body = format!(r#"{{"ticket":"{ticket}","device_name":"Motorola"}}"#);
        let (status, response) = http(address, &order("/pair/claim", None, &claim_body)).await;
        assert_eq!(status, 200);
        let response: Value = serde_json::from_str(&response).unwrap();
        let token = response["token"].as_str().unwrap().to_string();

        // El mismo ticket no se puede canjear dos veces.
        let (status, _) = http(address, &order("/pair/claim", None, &claim_body)).await;
        assert_eq!(status, 400);
        drop(guard);

        let (status, response) = http(address, &format!("GET /paired/status HTTP/1.1\r\nHost: receptor\r\nAuthorization: Bearer {token}\r\nConnection: close\r\n\r\n")).await;
        assert_eq!(status, 200);
        let response: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response["success"], true);
        assert!(response["pc_name"].as_str().is_some_and(|name| !name.is_empty()));
        assert!(response["expires_in"].as_u64().is_some_and(|seconds| seconds > 0));
        assert_eq!(response["protocol"], PAIRED_PROTOCOL);

        let (status, response) = http(address, "GET /status HTTP/1.1\r\nHost: receptor\r\nConnection: close\r\n\r\n").await;
        assert_eq!(status, 200);
        let response: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response["status"], "online");
        assert_eq!(response["version"], env!("CARGO_PKG_VERSION"));

        let (status, _) = http(address, "OPTIONS /paired/dispatch HTTP/1.1\r\nHost: receptor\r\nConnection: close\r\n\r\n").await;
        assert_eq!(status, 204);
        let (status, _) = http(address, "GET /desconocida HTTP/1.1\r\nHost: receptor\r\nConnection: close\r\n\r\n").await;
        assert_eq!(status, 404);
        assert!(calls.lock().unwrap_or_else(|lock| lock.into_inner()).is_empty());
        task.abort();
    }

    #[tokio::test]
    async fn el_receptor_http_rechaza_origenes_web_y_ordenes_invalidas() {
        let token = pairing_token();
        let (address, calls, task) = receiver(None).await;
        let body = r#"{"app_id":"prisma","content":""}"#;
        let request = format!(
            "POST /paired/dispatch HTTP/1.1\r\nHost: receptor\r\nOrigin: http://malicioso.test\r\nAuthorization: Bearer {token}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let (status, _) = http(address, &request).await;
        assert_eq!(status, 403, "un navegador no puede suplantar al teléfono");

        let invalid = r#"{"content":"sin app"}"#;
        let (status, response) = http(address, &order("/paired/dispatch", Some(&token), invalid)).await;
        assert_eq!(status, 400);
        let response: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response["delivery"], "rejected");
        assert!(calls.lock().unwrap_or_else(|lock| lock.into_inner()).is_empty());
        task.abort();
    }

    #[tokio::test]
    async fn un_rechazo_de_la_app_se_propaga_sin_segundo_intento() {
        let token = pairing_token();
        let (address, calls, task) = receiver(Some("Luna Fetch no está instalada en este equipo")).await;
        let body = r#"{"app_id":"luna","content":""}"#;
        let (status, response) = http(address, &order("/paired/dispatch", Some(&token), body)).await;
        assert_eq!(status, 200, "el rechazo de la app es un resultado, no un fallo de transporte");
        let response: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response["status"], "error");
        assert_eq!(response["delivery"], "rejected");
        assert_eq!(response["message"], "Luna Fetch no está instalada en este equipo");
        assert_eq!(calls.lock().unwrap_or_else(|lock| lock.into_inner()).len(), 1, "un rechazo no se reintenta");
        task.abort();
    }

    #[tokio::test]
    async fn una_credencial_vencida_no_ejecuta() {
        let guard = STATE_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        let token = "b".repeat(64);
        state().lock().unwrap().peers.insert(
            digest(&token),
            Peer { name: "Phone".into(), expires: Instant::now() - Duration::from_secs(1) },
        );
        drop(guard);
        let (address, calls, task) = receiver(None).await;
        let body = r#"{"app_id":"prisma","content":""}"#;
        let (status, _) = http(address, &order("/paired/dispatch", Some(&token), body)).await;
        assert_eq!(status, 401);
        assert!(calls.lock().unwrap_or_else(|lock| lock.into_inner()).is_empty());
        task.abort();
    }
}
