//! Catálogo de herramientas MCP del puente de Aurora Synapse.
//!
//! Cada herramienta es una orden concreta contra una superficie que **existe
//! hoy** en el código de la app destino. No se publica ninguna herramienta para
//! WinTTS, Ely-Tesia, LyraFlow ni MouziFlow porque esas apps no aceptan
//! invocaciones externas; publicarlas sería anunciar capacidades rotas.
//!
//! Convenciones:
//!
//! - Las herramientas de solo lectura llevan `readOnlyHint: true`, de modo que
//!   el cliente MCP no exija confirmación (ver `synapse_bridge.rs` del backend
//!   de Ely Intelligence).
//! - Las operaciones masivas o destructivas llevan `destructiveHint: true`.
//! - Toda orden queda registrada en el log de auditoría local.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};

use crate::mcp::dispatch::{self, AuditEntry, Transport, UriOpener};
use crate::mcp::routes;

/// Tiempo máximo de una llamada HTTP del puente.
const HTTP_TIMEOUT_MS: u64 = 6000;

/// Definición de una herramienta MCP.
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub read_only: bool,
    pub destructive: bool,
    pub schema: Value,
}

impl ToolSpec {
    /// Representación MCP de la herramienta (`tools/list`).
    pub fn to_mcp(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "inputSchema": self.schema,
            "annotations": {
                "readOnlyHint": self.read_only,
                "destructiveHint": self.destructive,
                "idempotentHint": self.read_only,
                "openWorldHint": false,
            },
        })
    }
}

/// Registro vivo de herramientas del puente.
pub struct ToolRegistry {
    specs: Vec<ToolSpec>,
    opener: UriOpener,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            specs: build_catalog(),
            opener: Arc::new(opener_del_sistema),
        }
    }

    /// Registro con un opener inyectado (el plugin `opener` de Tauri).
    pub fn with_opener(opener: UriOpener) -> Self {
        Self {
            specs: build_catalog(),
            opener,
        }
    }

    pub fn len(&self) -> usize {
        self.specs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }

    pub fn exists(&self, name: &str) -> bool {
        self.specs.iter().any(|spec| spec.name == name)
    }

    /// Lista MCP de todas las herramientas.
    pub fn list(&self) -> Vec<Value> {
        self.specs.iter().map(|spec| spec.to_mcp()).collect()
    }

    /// Ejecuta una herramienta y devuelve el resultado en formato MCP.
    pub async fn call(&self, name: &str, arguments: Value) -> Value {
        let validation = self
            .specs
            .iter()
            .find(|spec| spec.name == name)
            .ok_or_else(|| format!("herramienta desconocida: {name}"))
            .and_then(|spec| super::validation::validate(&arguments, &spec.schema));
        if let Err(message) = validation {
            return json!({"content": [{"type": "text", "text": message}], "isError": true});
        }
        let gallery_arguments = name.starts_with("gallery_").then(|| arguments.clone());
        let davinci_arguments = name.starts_with("davinciflow_").then(|| arguments.clone());
        let outcome = match name {
            "apps_list" => self.apps_list().await,
            "classify_intent" => self.classify_intent(arguments),
            "luna_status" => self.luna_status().await,
            "luna_download" => self.luna_download(arguments).await,
            "luna_analyze" => self.luna_analyze(arguments).await,
            "luna_open_folder" => self.luna_open_folder(arguments).await,
            "luna_focus" => self.luna_focus().await,
            "prisma_status" => self.prisma_status().await,
            "prisma_playback" => self.prisma_playback().await,
            "prisma_play" => self.prisma_play(arguments).await,
            "prisma_control" => self.prisma_control(arguments).await,
            "upscaler_status" => self.upscaler_status().await,
            "upscaler_models" => self.upscaler_models().await,
            "upscaler_scale" => self.upscaler_scale(arguments).await,
            "gallery_devices" => Ok(
                json!({"devices": super::devices::list().into_iter().filter(|d| d.target_app == "supergallery").collect::<Vec<_>>()}),
            ),
            "gallery_status" => super::gallery::status().await,
            "gallery_list" => super::gallery::list(&arguments).await,
            "gallery_download" => match (
                required_string(&arguments, "media_id"),
                required_string(&arguments, "output_path"),
            ) {
                (Ok(id), Ok(output)) => super::gallery::download(&id, &output).await,
                _ => Err("gallery_download requiere media_id y output_path".into()),
            },
            "gallery_send" => match required_string(&arguments, "file_path") {
                Ok(path) => super::gallery::send(&path).await,
                Err(error) => Err(error),
            },
            n if n.starts_with("davinciflow_") => super::davinci::call_tool(n, &arguments).await,
            "dispatch" => self.dispatch(arguments).await,
            _ => Err(format!("herramienta desconocida: {name}")),
        };

        if let Some(arguments) = gallery_arguments.as_ref() {
            let transport = if name == "gallery_devices" {
                Transport::Udp
            } else {
                Transport::Http
            };
            match &outcome {
                Ok(payload) => record(
                    "supergallery",
                    name,
                    transport,
                    if payload["completed"] == true {
                        "completed"
                    } else {
                        "ok"
                    },
                    "operación de galería",
                    &arguments,
                ),
                Err(error) => record("supergallery", name, transport, "error", error, &arguments),
            }
        }

        if let Some(arguments) = davinci_arguments.as_ref() {
            let (status, detail) = match &outcome {
                Ok(payload) => (
                    if payload.get("éxito") == Some(&json!(true))
                        || payload.get("insertado") == Some(&json!(true))
                        || payload.get("escrito") == Some(&json!(true))
                    {
                        "completed"
                    } else {
                        "ok"
                    },
                    "operación de DaVinci Flow",
                ),
                Err(error) => ("error", error.as_str()),
            };
            record(
                "davinciflow",
                name,
                Transport::Stdio,
                status,
                detail,
                arguments,
            );
        }

        match outcome {
            Ok(payload) => json!({
                "content": [{ "type": "text", "text": payload.to_string() }],
                "isError": false,
                "structuredContent": payload,
            }),
            Err(message) => json!({
                "content": [{ "type": "text", "text": message }],
                "isError": true,
            }),
        }
    }

    // ---------------------------------------------------------------- Luna

    pub async fn luna_status(&self) -> Result<Value, String> {
        let ping = http_get_json(routes::LUNA_LAN_PORT, routes::LUNA_PING).await?;
        let status = http_get_json(routes::LUNA_LAN_PORT, routes::LUNA_STATUS).await?;
        Ok(json!({
            "app": "Luna Fetch",
            "reachable": true,
            "ping": ping,
            "status": status,
            "dispatch_port": routes::LUNA_DISPATCH_PORT,
        }))
    }

    async fn luna_download(&self, arguments: Value) -> Result<Value, String> {
        let url = required_url(&arguments)?;
        let media_type =
            optional_string(&arguments, "media_type").unwrap_or_else(|| "video".to_string());
        let source_device = device_name();

        let mut body = json!({
            "url": url,
            "media_type": media_type,
            "source_device": source_device,
        });
        if let Some(quality) = optional_string(&arguments, "quality") {
            body["quality"] = json!(quality);
        }
        if let Some(auto_play) = arguments.get("auto_play").and_then(Value::as_bool) {
            body["auto_play"] = json!(auto_play);
        }

        let response =
            http_post_json(routes::LUNA_LAN_PORT, routes::LUNA_PUSH_DOWNLOAD, body).await?;
        record(
            "luna",
            "enqueue_download",
            Transport::Http,
            "accepted",
            "solicitud de descarga recibida",
            &arguments,
        );

        Ok(json!({
            "app": "Luna Fetch",
            "action": "enqueue_download",
            "delivery": "accepted",
            "completed": false,
            "url": url,
            "media_type": media_type,
            "source_device": source_device,
            "response": response,
        }))
    }

    async fn luna_analyze(&self, arguments: Value) -> Result<Value, String> {
        let url = required_url(&arguments)?;
        let envelope = dispatch::build_envelope(
            "luna",
            "analyze_url",
            json!({ "url": url, "silent": arguments.get("silent").and_then(Value::as_bool).unwrap_or(true) }),
        );
        let line = envelope.to_string();
        luna_dispatch(&line).await?;
        record(
            "luna",
            "analyze_url",
            Transport::Tcp,
            "sent",
            "análisis solicitado",
            &arguments,
        );
        Ok(json!({
            "app": "Luna Fetch",
            "action": "analyze_url",
            "url": url,
            "delivery": "sent",
            "note": "El despachador de Luna no confirma con ACK: se registró la orden, no su resultado.",
        }))
    }

    async fn luna_open_folder(&self, arguments: Value) -> Result<Value, String> {
        let raw = required_string(&arguments, "path")?;
        let path = dispatch::sanitize_path(&raw)?;
        if !path.is_absolute() || !path.is_dir() {
            return Err("path debe ser una carpeta existente con ruta absoluta".into());
        }
        let envelope = dispatch::build_envelope(
            "luna",
            "open_folder",
            json!({ "path": path.to_string_lossy() }),
        );
        luna_dispatch(&envelope.to_string()).await?;
        record(
            "luna",
            "open_folder",
            Transport::Tcp,
            "sent",
            "apertura de carpeta solicitada",
            &arguments,
        );
        Ok(json!({
            "app": "Luna Fetch",
            "action": "open_folder",
            "path": path.to_string_lossy(),
            "delivery": "sent",
        }))
    }

    async fn luna_focus(&self) -> Result<Value, String> {
        let envelope = dispatch::build_envelope("luna", "focus", json!({}));
        luna_dispatch(&envelope.to_string()).await?;
        record(
            "luna",
            "focus",
            Transport::Tcp,
            "sent",
            "foco solicitado",
            &json!({}),
        );
        Ok(json!({
            "app": "Luna Fetch",
            "action": "focus",
            "delivery": "sent",
        }))
    }

    // -------------------------------------------------------------- Prisma

    pub async fn prisma_status(&self) -> Result<Value, String> {
        let status = http_get_json(routes::PRISMA_HTTP_PORT, routes::PRISMA_STATUS).await?;
        Ok(json!({
            "app": "Prisma",
            "reachable": true,
            "status": status,
        }))
    }

    async fn prisma_playback(&self) -> Result<Value, String> {
        let playback = http_get_json(routes::PRISMA_HTTP_PORT, routes::PRISMA_PLAYBACK).await?;
        Ok(json!({
            "app": "Prisma",
            "reachable": true,
            "playback": playback,
        }))
    }

    async fn prisma_play(&self, arguments: Value) -> Result<Value, String> {
        let raw = required_string(&arguments, "file_path")?;
        let path = dispatch::sanitize_path(&raw)?;
        if !path.is_absolute() || !path.is_file() {
            return Err("file_path debe ser un archivo existente con ruta absoluta".into());
        }
        let autoplay = arguments
            .get("autoplay")
            .and_then(Value::as_bool)
            .unwrap_or(true);

        // Prisma registra el esquema `prisma://` en Windows, así que la vía
        // natural es el deep link con la ruta codificada.
        let uri = format!(
            "prisma://open?path={}&autoplay={}",
            urlencoding::encode(&path.to_string_lossy()),
            autoplay
        );

        let mut attempts: Vec<String> = Vec::new();
        match dispatch::open_uri(&self.opener, &uri) {
            Ok(detail) => {
                record(
                    "prisma",
                    "open_media",
                    Transport::Uri,
                    "sent",
                    &detail,
                    &arguments,
                );
                return Ok(json!({
                    "app": "Prisma",
                    "action": "open_media",
                    "file_path": path.to_string_lossy(),
                    "autoplay": autoplay,
                    "transport": "uri",
                    "delivery": "sent",
                    "completed": false,
                    "uri": uri,
                }));
            }
            Err(error) => attempts.push(format!("uri: {error}")),
        }

        // Respaldo: handoff por HTTP, solo funciona si Prisma ya está corriendo.
        let body = json!({
            "path": path.to_string_lossy(),
            "is_playing": autoplay,
        });
        match http_post_json(routes::PRISMA_HTTP_PORT, routes::PRISMA_HANDOFF, body).await {
            Ok(response) => {
                record(
                    "prisma",
                    "open_media",
                    Transport::Http,
                    "accepted",
                    "handoff recibido",
                    &arguments,
                );
                Ok(json!({
                    "app": "Prisma",
                    "action": "open_media",
                    "file_path": path.to_string_lossy(),
                    "autoplay": autoplay,
                    "transport": "http",
                    "delivery": "accepted",
                    "completed": false,
                    "response": response,
                }))
            }
            Err(error) => {
                attempts.push(format!("handoff: {error}"));
                record(
                    "prisma",
                    "open_media",
                    Transport::Uri,
                    "error",
                    &error,
                    &arguments,
                );
                Err(format!(
                    "No se pudo abrir el archivo en Prisma. Intentos: {}",
                    attempts.join(" | ")
                ))
            }
        }
    }

    async fn prisma_control(&self, arguments: Value) -> Result<Value, String> {
        let command = required_string(&arguments, "command")?;
        if !routes::PRISMA_COMMANDS.contains(&command.as_str()) {
            return Err(format!(
                "comando no permitido: {command}. Admitidos: {}",
                routes::PRISMA_COMMANDS.join(", ")
            ));
        }

        let mut body = json!({ "command": command });
        if let Some(value) = arguments.get("value") {
            body["value"] = value.clone();
        }
        if let Some(text) = optional_string(&arguments, "text") {
            body["text"] = json!(text);
        }

        let response =
            http_post_json(routes::PRISMA_HTTP_PORT, routes::PRISMA_REMOTE, body).await?;
        record(
            "prisma",
            "playback_control",
            Transport::Http,
            "accepted",
            &command,
            &arguments,
        );
        Ok(json!({
            "app": "Prisma",
            "action": "playback_control",
            "delivery": "accepted",
            "completed": false,
            "command": command,
            "response": response,
        }))
    }

    // ------------------------------------------------------------ Upscaler

    pub async fn upscaler_status(&self) -> Result<Value, String> {
        match http_get_json(routes::UPSCALER_HTTP_PORT, routes::UPSCALER_HEALTH).await {
            Ok(health) => Ok(json!({
                "app": "Prisma Upscaler",
                "server_reachable": true,
                "server_port": routes::UPSCALER_HTTP_PORT,
                "health": health,
                "desktop_executable": routes::executable_exists(&routes::upscaler_desktop_exe()),
                "cli_executable": routes::executable_exists(&routes::upscaler_cli_exe()),
            })),
            Err(error) => Ok(json!({
                "app": "Prisma Upscaler",
                "server_reachable": false,
                "server_port": routes::UPSCALER_HTTP_PORT,
                "server_error": error,
                "desktop_executable": routes::executable_exists(&routes::upscaler_desktop_exe()),
                "cli_executable": routes::executable_exists(&routes::upscaler_cli_exe()),
                "note": "Sin microservicio, el puente utiliza la CLI local. La GUI instalada no implica capacidad de escalado por MCP.",
            })),
        }
    }

    async fn upscaler_models(&self) -> Result<Value, String> {
        let models = http_get_json(routes::UPSCALER_HTTP_PORT, routes::UPSCALER_MODELS).await?;
        Ok(json!({
            "app": "Prisma Upscaler",
            "models": models,
        }))
    }

    async fn upscaler_scale(&self, arguments: Value) -> Result<Value, String> {
        let raw = required_string(&arguments, "input_path")?;
        let input = dispatch::sanitize_path(&raw)?;
        if !input.is_absolute() || !input.is_file() {
            return Err(format!(
                "input_path debe ser un archivo existente con ruta absoluta: {}",
                input.display()
            ));
        }
        let scale = arguments.get("scale").and_then(Value::as_u64).unwrap_or(4);
        let engine = optional_string(&arguments, "engine").unwrap_or_else(|| "ncnn".to_string());
        let model = optional_string(&arguments, "model");
        let sharpness = arguments
            .get("sharpness")
            .and_then(Value::as_f64)
            .unwrap_or(0.5);
        super::upscaler::validate_options(scale, &engine, sharpness)?;
        let requested = optional_string(&arguments, "output_path");
        let output = super::upscaler::output_path(&input, requested.as_deref())?;
        // Elegir antes del POST; nunca repetir tras un error de procesamiento.
        let server_available = http_get_json(routes::UPSCALER_HTTP_PORT, routes::UPSCALER_HEALTH)
            .await
            .is_ok();
        let transport = if server_available {
            Transport::Http
        } else {
            Transport::Cli
        };
        let outcome = if server_available {
            upscale_via_server(
                &input,
                scale,
                &engine,
                model.as_deref(),
                Some(sharpness),
                Some(&output.to_string_lossy()),
            )
            .await
        } else {
            super::upscaler::via_cli(&input, &output, scale, &engine, model.as_deref(), sharpness)
                .await
        };
        match outcome {
            Ok(result) => {
                record(
                    "prisma-upscaler",
                    "upscale",
                    transport,
                    "completed",
                    transport.as_str(),
                    &arguments,
                );
                Ok(json!({
                    "app": "Prisma Upscaler", "action": "upscale",
                    "transport": transport.as_str(), "delivery": "completed", "completed": true,
                    "input_path": input, "scale": scale, "engine": engine, "result": result,
                }))
            }
            Err(error) => {
                record(
                    "prisma-upscaler",
                    "upscale",
                    transport,
                    "error",
                    &error,
                    &arguments,
                );
                Err(error)
            }
        }
    }

    // ------------------------------------------------------------ Locales

    async fn apps_list(&self) -> Result<Value, String> {
        let launchpad: Vec<Value> = crate::app_catalog()
            .iter()
            .map(|app| {
                json!({
                    "id": app.id,
                    "name": app.name,
                    "category": app.category,
                    "is_web_app": app.is_web_app,
                    "platforms": app.platforms,
                    "uri_scheme": app.uri_scheme,
                    "port": app.port,
                })
            })
            .collect();

        let (luna, prisma, upscaler, gallery) = tokio::join!(
            self.luna_status(),
            self.prisma_status(),
            self.upscaler_status(),
            super::gallery::status()
        );
        let checked_at = crate::mcp::protocol::iso8601_utc(std::time::SystemTime::now());
        let targets: Vec<Value> = routes::BRIDGE_TARGETS
            .iter()
            .map(|(id, name, surface, tools)| {
                let probe = match *id {
                    "luna" => Some(&luna),
                    "prisma" => Some(&prisma),
                    "prisma-upscaler" => Some(&upscaler),
                    "supergallery" => Some(&gallery),
                    _ => None,
                };
                let mut target = json!({
                    "id": id, "name": name, "surface": surface, "tools": tools,
                    "supported": !tools.is_empty(), "reachable": false,
                    "availability": "unsupported",
                });
                if let Some(probe) = probe {
                    target["checked_at_utc"] = json!(checked_at);
                    match probe {
                        Ok(status) => {
                            let reachable = status
                                .get("reachable")
                                .or_else(|| status.get("server_reachable"))
                                .and_then(Value::as_bool)
                                .unwrap_or(false);
                            target["reachable"] = json!(reachable);
                            target["availability"] =
                                json!(if reachable { "online" } else { "offline" });
                            target["status"] = status.clone();
                            if *id == "prisma-upscaler"
                                && !reachable
                                && status["cli_executable"] == true
                            {
                                target["availability"] = json!("cli_available");
                            }
                        }
                        Err(error) => {
                            target["availability"] = json!("offline");
                            target["error"] = json!(error);
                        }
                    }
                }
                target
            })
            .collect();

        Ok(json!({
            "ecosistema": launchpad,
            "capacidades_del_puente": targets,
        }))
    }

    fn classify_intent(&self, arguments: Value) -> Result<Value, String> {
        let input = required_string(&arguments, "input")?;
        let result = crate::classify_intent_impl(&input);
        Ok(json!({
            "detected_type": result.detected_type,
            "recommended_app_id": result.recommended_app_id,
            "reason": result.reason,
        }))
    }

    // ------------------------------------------------------------ Genérica

    /// Despacho genérico **acotado**: solo acepta combinaciones
    /// (app, acción) que el puente ya sabe traducir. No hay forma de
    /// ejecutar procesos arbitrarios.
    async fn dispatch(&self, arguments: Value) -> Result<Value, String> {
        let target = required_string(&arguments, "target_app")?;
        let action = required_string(&arguments, "action")?;
        let payload = arguments
            .get("payload")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let tool = match (target.as_str(), action.as_str()) {
            ("luna", "enqueue_download") => "luna_download",
            ("luna", "analyze_url") => "luna_analyze",
            ("luna", "open_folder") => "luna_open_folder",
            ("luna", "focus") => "luna_focus",
            ("prisma", "remote") => "prisma_control",
            _ => return Err(format!("Combinación no permitida: {target}/{action}")),
        };
        let spec = self.specs.iter().find(|spec| spec.name == tool).unwrap();
        super::validation::validate(&payload, &spec.schema)?;
        match tool {
            "luna_download" => self.luna_download(payload).await,
            "luna_analyze" => self.luna_analyze(payload).await,
            "luna_open_folder" => self.luna_open_folder(payload).await,
            "luna_focus" => self.luna_focus().await,
            "prisma_control" => self.prisma_control(payload).await,
            _ => unreachable!(),
        }
    }
}

// --------------------------------------------------------------- Transporte

/// Envía una línea JSON al despachador de Luna Fetch (`127.0.0.1:49282`).
///
/// El protocolo es una línea terminada en `\n` y **no hay ACK**: el
/// despachador lee la línea y cierra el socket. Por eso la herramienta
/// informa "enviado", nunca "ejecutado".
async fn luna_dispatch(line: &str) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;

    tokio::time::timeout(Duration::from_millis(HTTP_TIMEOUT_MS), async {
        let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", routes::LUNA_DISPATCH_PORT))
            .await
            .map_err(|error| {
                format!("no se pudo conectar con el despachador de Luna Fetch: {error}")
            })?;
        stream
            .write_all(format!("{line}\n").as_bytes())
            .await
            .map_err(|error| error.to_string())?;
        stream.shutdown().await.map_err(|error| error.to_string())?;
        Ok(())
    })
    .await
    .map_err(|_| "el despachador de Luna superó el tiempo máximo".to_string())?
}

/// GET JSON contra un puerto local.
async fn http_get_json(port: u16, path: &str) -> Result<Value, String> {
    let url = format!("http://127.0.0.1:{port}{path}");
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_millis(HTTP_TIMEOUT_MS))
        .build()
        .map_err(|error| error.to_string())?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|error| format!("no se pudo conectar con {url}: {error}"))?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!(
            "{url} respondió HTTP {status}: {}",
            dispatch::truncate(&body, 200)
        ));
    }
    let result: Value = serde_json::from_str(&body)
        .map_err(|error| format!("{url} devolvió JSON inválido: {error}"))?;
    check_app_response(&result)?;
    Ok(result)
}

/// POST JSON contra un puerto local.
async fn http_post_json(port: u16, path: &str, body: Value) -> Result<Value, String> {
    let url = format!("http://127.0.0.1:{port}{path}");
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_millis(HTTP_TIMEOUT_MS))
        .build()
        .map_err(|error| error.to_string())?;

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|error| format!("no se pudo conectar con {url}: {error}"))?;

    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!(
            "{url} respondió HTTP {status}: {}",
            dispatch::truncate(&text, 200)
        ));
    }
    if text.trim().is_empty() {
        return Ok(json!({ "status": status.as_u16() }));
    }
    let result: Value = serde_json::from_str(&text)
        .map_err(|error| format!("{url} devolvió JSON inválido: {error}"))?;
    check_app_response(&result)?;
    Ok(result)
}

/// Sube una imagen al microservicio del upscaler (`multipart/form-data`).
///
/// El cuerpo se construye a mano para no añadir la característica `multipart`
/// de `reqwest` ni tocar `Cargo.lock`.
async fn upscale_via_server(
    input: &PathBuf,
    scale: u64,
    engine: &str,
    model: Option<&str>,
    sharpness: Option<f64>,
    output: Option<&str>,
) -> Result<Value, String> {
    let bytes = tokio::fs::read(input)
        .await
        .map_err(|error| format!("no se pudo leer {}: {error}", input.display()))?;

    let boundary = format!("----auroraSynapse{}", dispatch::idempotency_key());
    let mut body: Vec<u8> = Vec::with_capacity(bytes.len() + 1024);
    let field = |name: &str, value: &str, body: &mut Vec<u8>| {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
        );
        body.extend_from_slice(value.as_bytes());
        body.extend_from_slice(b"\r\n");
    };

    field("scale", &scale.to_string(), &mut body);
    field("engine", engine, &mut body);
    if let Some(model) = model {
        field("model", model, &mut body);
    }
    if let Some(sharpness) = sharpness {
        field("sharpness", &sharpness.to_string(), &mut body);
    }

    let file_name = "input-image";
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"image\"; filename=\"{file_name}\"\r\n\
             Content-Type: application/octet-stream\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(&bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let url = format!(
        "http://127.0.0.1:{}{}",
        routes::UPSCALER_HTTP_PORT,
        routes::UPSCALER_UPSCALE
    );
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|error| error.to_string())?;

    let response = client
        .post(&url)
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(body)
        .send()
        .await
        .map_err(|error| format!("no se pudo conectar con {url}: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!(
            "{url} respondió HTTP {status}: {}",
            dispatch::truncate(&text, 200)
        ));
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !content_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .eq_ignore_ascii_case("image/png")
    {
        return Err(format!(
            "el upscaler respondió {content_type}, se esperaba image/png"
        ));
    }
    let payload = response.bytes().await.map_err(|error| error.to_string())?;
    let target = output.ok_or("falta la ruta de salida del escalado")?;
    super::upscaler::save_new(std::path::Path::new(target), &payload).await
}

/// Abre un esquema URI usando el registro del sistema operativo.
fn opener_del_sistema(uri: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let mut command = std::process::Command::new("cmd");
        command
            .raw_arg("/C")
            .raw_arg(format!("start \"\" \"{uri}\""));
        command
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("no se pudo abrir {uri}: {error}"))
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("xdg-open")
            .arg(uri)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("no se pudo abrir {uri}: {error}"))
    }
}

// -------------------------------------------------------------- Utilidades

fn required_url(arguments: &Value) -> Result<String, String> {
    let raw = required_string(arguments, "url")?;
    let url = reqwest::Url::parse(&raw).map_err(|_| "URL inválida".to_string())?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err("la URL debe usar http o https y tener un host".into());
    }
    Ok(raw)
}

fn check_app_response(response: &Value) -> Result<(), String> {
    if response.get("success").and_then(Value::as_bool) == Some(false) {
        return Err(format!(
            "la app rechazó la orden: {}",
            response["message"].as_str().unwrap_or("sin detalle")
        ));
    }
    Ok(())
}

fn required_string(arguments: &Value, key: &str) -> Result<String, String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("falta el parámetro '{key}'"))
}

fn optional_string(arguments: &Value, key: &str) -> Option<String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Nombre del dispositivo según el estándar del ecosistema: **hostname**, nunca
/// el nombre de usuario de la sesión.
fn device_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "desconocido".to_string())
}

/// Registra la orden en el log de auditoría local.
fn record(
    target_app: &str,
    action: &str,
    transport: Transport,
    status: &str,
    detail: &str,
    arguments: &Value,
) {
    dispatch::audit(&AuditEntry {
        timestamp_utc: crate::mcp::protocol::iso8601_utc(std::time::SystemTime::now()),
        source_app: dispatch::MCP_SOURCE_APP.to_string(),
        target_app: target_app.to_string(),
        action: action.to_string(),
        transport: transport.as_str().to_string(),
        status: status.to_string(),
        detail: detail.to_string(),
        payload_preview: dispatch::payload_preview(arguments),
    });
}

/// Construye el catálogo completo de herramientas.
fn build_catalog() -> Vec<ToolSpec> {
    let object = |properties: Value, required: Vec<&str>| -> Value {
        json!({
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": false,
        })
    };

    vec![
        ToolSpec {
            name: "gallery_devices", description: "Teléfonos con Super Galería anunciados por los mismos beacons LAN que usa Prisma. Descubrimiento no implica autorización.",
            read_only: true, destructive: false, schema: object(json!({}), vec![]),
        },
        ToolSpec {
            name: "gallery_status", description: "Estado real de Super Galería: conexión, respuesta del teléfono y autorización del catálogo. No revela credenciales.",
            read_only: true, destructive: false, schema: object(json!({}), vec![]),
        },
        ToolSpec {
            name: "gallery_list", description: "Consulta fotos, vídeos y música del teléfono autorizado. Catálogo paginado; excluye bóveda, ocultos, papelera y carpetas excluidas. Usa los IDs devueltos para descargar.",
            read_only: true, destructive: false,
            schema: object(json!({
                "query": {"type": "string"}, "media_type": {"type": "string", "enum": ["all", "image", "video", "audio"]},
                "offset": {"type": "integer", "minimum": 0, "maximum": 10000}, "limit": {"type": "integer", "minimum": 1, "maximum": 100}
            }), vec![]),
        },
        ToolSpec {
            name: "gallery_download", description: "Descarga por ID del catálogo del teléfono a una ruta absoluta nueva en este equipo. Comprueba tamaño y SHA-256; nunca sobrescribe.",
            read_only: false, destructive: false,
            schema: object(json!({"media_id": {"type": "string"}, "output_path": {"type": "string"}}), vec!["media_id", "output_path"]),
        },
        ToolSpec {
            name: "gallery_send", description: "Envía un archivo local a Super Galería mediante su receptor existente, con autorización de agentes. Solo confirma completed cuando el teléfono devuelve tamaño y SHA-256 coincidentes.",
            read_only: false, destructive: false,
            schema: object(json!({"file_path": {"type": "string"}}), vec!["file_path"]),
        },
        ToolSpec {
            name: "apps_list",
            description: "Lista las aplicaciones, sus herramientas soportadas y su disponibilidad \
                          comprobada ahora. cli_available solo indica que la CLI está instalada; \
                          no garantiza que el motor o la GPU estén disponibles.",
            read_only: true,
            destructive: false,
            schema: object(json!({}), vec![]),
        },
        ToolSpec {
            name: "classify_intent",
            description: "Clasifica un texto o enlace (dominio, tipo MIME) y devuelve la app del \
                          ecosistema que mejor encaja, con el motivo de la decisión.",
            read_only: true,
            destructive: false,
            schema: object(
                json!({
                    "input": {
                        "type": "string",
                        "description": "URL, nombre de archivo o texto libre a clasificar."
                    }
                }),
                vec!["input"],
            ),
        },
        ToolSpec {
            name: "luna_status",
            description: "Comprueba si Luna Fetch está vivo y responde en su servidor Synapse \
                          local (ping y estado del nodo).",
            read_only: true,
            destructive: false,
            schema: object(json!({}), vec![]),
        },
        ToolSpec {
            name: "luna_download",
            description: "Pide a Luna Fetch encolar una descarga desde una URL. \
                          Luna confirma la recepción con HTTP 200.",
            read_only: false,
            destructive: false,
            schema: object(
                json!({
                    "url": { "type": "string", "description": "URL a descargar." },
                    "media_type": {
                        "type": "string",
                        "enum": ["audio", "video"],
                        "description": "Tipo de medio; audio para música."
                    },
                    "quality": { "type": "string", "description": "Perfil de calidad opcional." },
                    "auto_play": {
                        "type": "boolean",
                        "description": "Reproducir en Prisma al terminar."
                    }
                }),
                vec!["url"],
            ),
        },
        ToolSpec {
            name: "luna_analyze",
            description: "Pide a Luna Fetch analizar una URL (metadatos, disponibilidad) sin \
                          descargar nada ni abrir la interfaz. Solo consulta: no modifica \
                          archivos ni la cola de descargas. El despachador de Luna no devuelve \
                          acuse de recibo.",
            read_only: true,
            destructive: false,
            schema: object(
                json!({
                    "url": { "type": "string", "description": "URL a analizar." }
                }),
                vec!["url"],
            ),
        },
        ToolSpec {
            name: "luna_open_folder",
            description: "Pide a Luna Fetch abrir una carpeta en su explorador.",
            read_only: false,
            destructive: false,
            schema: object(
                json!({
                    "path": { "type": "string", "description": "Ruta de la carpeta." }
                }),
                vec!["path"],
            ),
        },
        ToolSpec {
            name: "luna_focus",
            description: "Trae la ventana de Luna Fetch al frente, sin cambiar de pestaña.",
            read_only: false,
            destructive: false,
            schema: object(json!({}), vec![]),
        },
        ToolSpec {
            name: "prisma_status",
            description: "Consulta el estado del servidor Synapse de Prisma.",
            read_only: true,
            destructive: false,
            schema: object(json!({}), vec![]),
        },
        ToolSpec {
            name: "prisma_playback",
            description: "Devuelve qué se está reproduciendo en Prisma: título, artista, álbum, \
                          posición, duración y si es vídeo.",
            read_only: true,
            destructive: false,
            schema: object(json!({}), vec![]),
        },
        ToolSpec {
            name: "prisma_play",
            description:
                "Reproduce un archivo (audio, vídeo, imagen) en Prisma mediante su esquema \
                          prisma:// y, si hace falta, entrega el handoff por HTTP.",
            read_only: false,
            destructive: false,
            schema: object(
                json!({
                    "file_path": { "type": "string", "description": "Ruta del archivo a abrir." },
                    "autoplay": {
                        "type": "boolean",
                        "description": "Reproducir de inmediato (por defecto true)."
                    }
                }),
                vec!["file_path"],
            ),
        },
        ToolSpec {
            name: "prisma_control",
            description: "Controla la reproducción de Prisma: play_pause, next, previous, \
                          seek_forward, seek_backward, volume_up, volume_down, toggle_mute, \
                          shuffle, lyrics, queue, search y otros verbos aceptados por Prisma.",
            read_only: false,
            destructive: false,
            schema: object(
                json!({
                    "command": {
                        "type": "string",
                        "enum": routes::PRISMA_COMMANDS,
                        "description": "Verbo de control aceptado por Prisma."
                    },
                    "value": { "type": "number", "description": "Valor opcional del comando." },
                    "text": { "type": "string", "description": "Texto para el comando search." }
                }),
                vec!["command"],
            ),
        },
        ToolSpec {
            name: "upscaler_status",
            description: "Estado del upscaler: microservicio en 127.0.0.1:8085, GPU, motores \
                          disponibles y si el ejecutable de escritorio con motor embebido está \
                          instalado.",
            read_only: true,
            destructive: false,
            schema: object(json!({}), vec![]),
        },
        ToolSpec {
            name: "upscaler_models",
            description: "Lista los modelos de super-resolución disponibles en el microservicio \
                          del upscaler.",
            read_only: true,
            destructive: false,
            schema: object(json!({}), vec![]),
        },
        ToolSpec {
            name: "upscaler_scale",
            description: "Escala una imagen. Usa el microservicio HTTP si está activo; si no, \
                          ejecuta la CLI local y guarda un PNG nuevo sin sobrescribir.",
            read_only: false,
            destructive: false,
            schema: object(
                json!({
                    "input_path": { "type": "string", "description": "Ruta de la imagen de entrada." },
                    "output_path": { "type": "string", "description": "Ruta absoluta de un PNG nuevo. Si se omite se genera un nombre único junto a la entrada. Nunca sobrescribe." },
                    "scale": { "type": "integer", "minimum": 1, "maximum": 4, "description": "Factor de escala." },
                    "engine": { "type": "string", "enum": ["ncnn", "nis", "fsr1", "dlss-nr", "rtx-video"], "description": "Motor de escalado." },
                    "model": { "type": "string", "description": "Modelo de super-resolución." },
                    "sharpness": { "type": "number", "minimum": 0, "maximum": 1, "description": "Nitidez (0..1)." }
                }),
                vec!["input_path"],
            ),
        },
        ToolSpec {
            name: "dispatch",
            description: "Despacho genérico acotado a las combinaciones que el puente ya \
                          traduce: luna/<acción> y prisma/remote. No permite ejecutar comandos \
                          del sistema ni acciones arbitrarias.",
            read_only: false,
            destructive: false,
            schema: object(
                json!({
                    "target_app": {
                        "type": "string",
                        "enum": ["luna", "prisma"],
                        "description": "App destino."
                    },
                    "action": {
                        "type": "string",
                        "description": "Acción admitida por la app destino."
                    },
                    "payload": { "type": "object", "description": "Parámetros de la acción." }
                }),
                vec!["target_app", "action"],
            ),
        },
        ToolSpec {
            name: "davinciflow_status",
            description: "Estado de DaVinci Flow: transporte de Resolve, proveedor de IA activo, \
                          modelo y credenciales disponibles.",
            read_only: true,
            destructive: false,
            schema: object(json!({}), vec![]),
        },
        ToolSpec {
            name: "davinciflow_templates",
            description: "Catálogo de plantillas de títulos y efectos disponible en el Media Pool \
                          de Resolve.",
            read_only: true,
            destructive: false,
            schema: object(json!({}), vec![]),
        },
        ToolSpec {
            name: "davinciflow_capture",
            description: "Captura los subtítulos de la línea de tiempo activa de Resolve con sus \
                          fotogramas.",
            read_only: true,
            destructive: false,
            schema: object(
                json!({
                    "track": { "type": "integer", "description": "Pista de subtítulos a leer." },
                    "first": { "type": "integer", "description": "Primer subtítulo a leer." },
                    "last": { "type": "integer", "description": "Último subtítulo a leer." }
                }),
                vec![],
            ),
        },
        ToolSpec {
            name: "davinciflow_fetch_media",
            description: "Descarga un video o pista de música de una URL (YouTube, Suno, TikTok, \
                          SoundCloud...) y lo importa en el Media Pool de Resolve bajo la carpeta \
                          organizada (Videos o Música).",
            read_only: false,
            destructive: false,
            schema: object(
                json!({
                    "url": { "type": "string", "description": "URL del video o canción a descargar." },
                    "kind": { "type": "string", "enum": ["video", "audio"], "description": "Tipo de medio: video (MP4) o audio (MP3/Música)." },
                    "import_to_resolve": { "type": "boolean", "description": "Si es true, se importa directamente al Media Pool de Resolve en su carpeta correspondiente." },
                    "custom_bin_name": { "type": "string", "description": "Nombre de carpeta opcional en el Media Pool." }
                }),
                vec!["url"],
            ),
        },
        ToolSpec {
            name: "davinciflow_create_fusion_effect",
            description: "Genera un efecto Fusion (Text+ con animación y brillo opcional) y lo \
                          inserta como composición editable.",
            read_only: false,
            destructive: false,
            schema: object(
                json!({
                    "text": { "type": "string", "description": "Texto del efecto." },
                    "role": { "type": "string", "description": "Rol tipográfico: main, context o accent." },
                    "animation_preset": { "type": "string", "description": "Animación: none, pop_bounce, slide_up, fade_smooth, kinetic_pulse o typewriter." },
                    "position_x": { "type": "number", "minimum": 0.0, "maximum": 1.0, "description": "Posición horizontal (0 a 1)." },
                    "position_y": { "type": "number", "minimum": 0.0, "maximum": 1.0, "description": "Posición vertical (0 a 1)." },
                    "glow": { "type": "number", "minimum": 0.0, "maximum": 1.0, "description": "Intensidad del brillo (0 a 1)." },
                    "theme": { "type": "string", "description": "Tema de marca: ely o aurora." },
                    "start_frame": { "type": "number", "description": "Fotograma de inicio." },
                    "duration_frames": { "type": "number", "description": "Duración en fotogramas." },
                    "track_index": { "type": "integer", "description": "Pista de vídeo de destino." }
                }),
                vec!["text"],
            ),
        },
        ToolSpec {
            name: "davinciflow_insert_native_title",
            description: "Inserta un título nativo de Resolve (Text+, Fade On, Headline…) en la \
                          línea de tiempo, editable en el Inspector.",
            read_only: false,
            destructive: false,
            schema: object(
                json!({
                    "template_name": { "type": "string", "description": "Nombre del título nativo, por ejemplo Text+ o Fade On." },
                    "start_frame": { "type": "number", "description": "Fotograma de inicio." },
                    "duration_frames": { "type": "number", "description": "Duración en fotogramas." },
                    "track_index": { "type": "integer", "description": "Pista de vídeo de destino." }
                }),
                vec!["template_name"],
            ),
        },
        ToolSpec {
            name: "davinciflow_write_subtitles",
            description: "Escribe subtítulos en la pista nativa de subtítulos de Resolve y \
                          devuelve el archivo SRT generado.",
            read_only: false,
            destructive: true,
            schema: object(
                json!({
                    "cues": {
                        "type": "array",
                        "description": "Subtítulos a escribir, con texto, start_frame, end_frame y track_index."
                    },
                    "track_index": { "type": "integer", "description": "Pista de subtítulos de destino." },
                    "file_name": { "type": "string", "description": "Nombre base del archivo SRT." }
                }),
                vec!["cues"],
            ),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rechaza_argumentos_invalidos_antes_de_producir_efectos() {
        let registry = ToolRegistry::with_opener(Arc::new(|_| panic!("no debe abrir la app")));
        for (name, arguments) in [
            (
                "prisma_play",
                json!({"file_path": "no-existe.mp3", "autoplay": "false"}),
            ),
            (
                "upscaler_scale",
                json!({"input_path": "no-existe.png", "scale": 8}),
            ),
            (
                "luna_download",
                json!({"url": "https://example.com", "media_type": "texto"}),
            ),
            ("luna_download", json!({"url": "file:///C:/private.txt"})),
            (
                "luna_download",
                json!({"url": "https://example.com", "extra": true}),
            ),
            ("luna_open_folder", json!({"path": "../../no-existe"})),
            (
                "dispatch",
                json!({"target_app": "luna", "action": "enqueue_download", "payload": {}}),
            ),
            (
                "dispatch",
                json!({"target_app": "prisma", "action": "remote", "payload": {"command": "mouse_click"}}),
            ),
        ] {
            assert_eq!(
                registry.call(name, arguments).await["isError"],
                true,
                "{name}"
            );
        }
    }

    #[tokio::test]
    async fn http_200_con_rechazo_de_app_no_es_exito() {
        use tokio::{
            io::{AsyncReadExt, AsyncWriteExt},
            net::TcpListener,
        };
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let worker = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0; 4096];
            let _ = socket.read(&mut request).await.unwrap();
            let body = r#"{"success":false,"message":"cola bloqueada"}"#;
            socket
                .write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
        });
        let result = http_post_json(port, "/reject", json!({})).await;
        assert!(result.unwrap_err().contains("cola bloqueada"));
        worker.await.unwrap();
    }

    #[test]
    fn el_catalogo_no_publica_herramientas_inexistentes() {
        let registry = ToolRegistry::new();
        for spec in &registry.specs {
            assert!(registry.exists(spec.name));
        }
        assert!(!registry.exists("wintts_speak"));
        assert!(!registry.exists("elytesia_practice"));
        assert!(!registry.exists("mouzi_organize"));
        assert!(!registry.exists("lyraflow_dictate"));
    }

    /// Contrato con el consumidor MCP de Ely Intelligence.
    ///
    /// Los nombres son públicos: el backend los publica como `synapse.<nombre>`
    /// y las skills los referencian por nombre. Si cambia uno, hay que
    /// recordinar el contrato antes de cambiarlo aquí.
    #[test]
    fn el_catalogo_publica_exactamente_el_contrato_acordado() {
        let registry = ToolRegistry::new();
        let listado = registry.list();
        let mut publicados: Vec<&str> = listado
            .iter()
            .map(|tool| tool["name"].as_str().unwrap_or_default())
            .collect();
        publicados.sort_unstable();

        let mut esperados = vec![
            "gallery_devices",
            "gallery_status",
            "gallery_list",
            "gallery_download",
            "gallery_send",
            "apps_list",
            "classify_intent",
            "luna_analyze",
            "luna_download",
            "luna_focus",
            "luna_open_folder",
            "luna_status",
            "prisma_control",
            "prisma_play",
            "prisma_playback",
            "prisma_status",
            "dispatch",
            "upscaler_models",
            "upscaler_status",
            "upscaler_scale",
            "davinciflow_status",
            "davinciflow_templates",
            "davinciflow_capture",
            "davinciflow_fetch_media",
            "davinciflow_create_fusion_effect",
            "davinciflow_insert_native_title",
            "davinciflow_write_subtitles",
        ];
        esperados.sort_unstable();

        assert_eq!(publicados, esperados);
    }

    /// Clasificación de solo lectura acordada con el consumidor.
    ///
    /// El cliente decide si pide confirmación a partir de `readOnlyHint`, así
    /// que mover una herramienta entre estos dos grupos cambia la experiencia
    /// del usuario: se avisa antes de tocarla.
    #[test]
    fn la_clasificacion_de_solo_lectura_es_la_acordada() {
        let registry = ToolRegistry::new();
        let listado = registry.list();

        let mut solo_lectura: Vec<&str> = listado
            .iter()
            .filter(|tool| tool["annotations"]["readOnlyHint"] == Value::Bool(true))
            .map(|tool| tool["name"].as_str().unwrap_or_default())
            .collect();
        solo_lectura.sort_unstable();

        let mut esperados = vec![
            "gallery_devices",
            "gallery_status",
            "gallery_list",
            "apps_list",
            "classify_intent",
            "luna_analyze",
            "luna_status",
            "prisma_playback",
            "prisma_status",
            "upscaler_models",
            "upscaler_status",
            "davinciflow_status",
            "davinciflow_templates",
            "davinciflow_capture",
        ];
        esperados.sort_unstable();

        assert_eq!(solo_lectura, esperados);
    }

    #[test]
    fn las_anotaciones_son_camel_case() {
        let registry = ToolRegistry::new();
        let tools = registry.list();
        let luna_download = tools
            .iter()
            .find(|tool| tool["name"] == "luna_download")
            .expect("luna_download está en el catálogo");
        assert_eq!(luna_download["annotations"]["readOnlyHint"], false);
        assert_eq!(luna_download["annotations"]["destructiveHint"], false);

        let luna_status = tools
            .iter()
            .find(|tool| tool["name"] == "luna_status")
            .expect("luna_status está en el catálogo");
        assert_eq!(luna_status["annotations"]["readOnlyHint"], true);
    }

    #[test]
    fn el_esquema_declara_campos_obligatorios() {
        let registry = ToolRegistry::new();
        let tools = registry.list();
        let luna_download = tools
            .iter()
            .find(|tool| tool["name"] == "luna_download")
            .expect("luna_download está en el catálogo");
        assert_eq!(luna_download["inputSchema"]["type"], "object");
        assert!(luna_download["inputSchema"]["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "url"));
    }

    #[tokio::test]
    async fn una_herramienta_inexistente_devuelve_error() {
        let registry = ToolRegistry::new();
        let result = registry.call("no_existe", json!({})).await;
        assert_eq!(result["isError"], true);
    }

    #[tokio::test]
    async fn prisma_control_rechaza_verbos_no_permitidos() {
        let registry = ToolRegistry::new();
        let result = registry
            .call("prisma_control", json!({ "command": "rm -rf" }))
            .await;
        assert_eq!(result["isError"], true);
    }

    #[tokio::test]
    async fn dispatch_rechaza_combinaciones_no_permitidas() {
        let registry = ToolRegistry::new();
        let result = registry
            .call(
                "dispatch",
                json!({ "target_app": "wintts", "action": "speak" }),
            )
            .await;
        assert_eq!(result["isError"], true);
    }
}
