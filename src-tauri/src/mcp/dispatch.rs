//! Capa de despacho del puente MCP: construye el envoltorio del Aurora Synapse
//! Protocol y lo entrega a la app destino por el primer transporte viable.
//!
//! Orden de intento (de más directo a más-tolerant):
//!
//! 1. **Named pipe** — entrega a la instancia viva de la app, sin abrir un proceso nuevo.
//! 2. **HTTP local** — endpoint loopback declarado por la app.
//! 3. **URI / CLI** — despertar la app vía esquema registrado o subcomando headless.
//!
//! Cada orden queda registrada en el log de auditoría local, tal y como exige el
//! contrato de la puerta de entrada para agentes (nivel 3 y 4 del protocolo).

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use serde_json::{json, Value};

/// Origen declarado en el envoltorio cuando la orden entra por el puente MCP.
pub const MCP_SOURCE_APP: &str = "aurora_synapse";

/// Versión del protocolo Synapse declarada en el envoltorio.
pub const SYNAPSE_VERSION: &str = "1.0";

/// Apertor de esquemas URI. En la app se inyecta el plugin `opener` de Tauri;
/// fuera de ella se usa el registro del sistema operativo.
pub type UriOpener = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

/// Transporte con el que se entregó una orden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    Pipe,
    Http,
    Uri,
    Cli,
    Tcp,
    Udp,
    Stdio,
}

impl Transport {
    pub fn as_str(self) -> &'static str {
        match self {
            Transport::Pipe => "named_pipe",
            Transport::Http => "http",
            Transport::Uri => "uri",
            Transport::Cli => "cli",
            Transport::Tcp => "tcp",
            Transport::Udp => "udp",
            Transport::Stdio => "stdio",
        }
    }
}

/// Resultado de un despacho exitoso.
#[derive(Debug, Clone)]
pub struct DispatchOutcome {
    pub transport: Transport,
    pub detail: String,
}

/// Construye el envoltorio estándar del Aurora Synapse Protocol.
///
/// Contrato: `Docs/features/aurora-synapse/payload-contracts.md`.
pub fn build_envelope(target_app: &str, action: &str, payload: Value) -> Value {
    json!({
        "synapse_version": SYNAPSE_VERSION,
        "source_app": MCP_SOURCE_APP,
        "target_app": target_app,
        "action": action,
        "timestamp_utc": crate::mcp::protocol::iso8601_utc(SystemTime::now()),
        "idempotency_key": idempotency_key(),
        "payload": payload,
    })
}

/// Clave de idempotencia única por orden (`syn-<nanos>-<contador>`).
pub fn idempotency_key() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let nanos = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!(
        "syn-{:x}-{:x}",
        nanos,
        COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

/// Escribe el envoltorio en un named pipe de Windows.
///
/// Se usa la API de archivos estándar de Rust: si no hay ninguna instancia
/// escuchando, `open` falla de inmediato, lo que sirve además de sonda de
/// disponibilidad. La escritura se ejecuta en un hilo aparte con tiempo máximo
/// para no dejar el despacho bloqueado si el lector no consume.
#[cfg(windows)]
pub fn send_via_pipe(pipe: &str, envelope: &Value) -> Result<String, String> {
    use std::io::Write;

    let pipe = pipe.to_string();
    let body = envelope.to_string();
    let pipe_for_thread = pipe.clone();

    let (sender, receiver) = std::sync::mpsc::channel::<Result<String, String>>();
    std::thread::spawn(move || {
        let result = (|| -> Result<String, String> {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .open(&pipe_for_thread)
                .map_err(|error| {
                    format!("no hay instancia escuchando en {pipe_for_thread}: {error}")
                })?;
            file.write_all(body.as_bytes())
                .map_err(|error| format!("fallo al escribir en {pipe_for_thread}: {error}"))?;
            file.flush().ok();
            Ok(format!("orden entregada por {pipe_for_thread}"))
        })();
        let _ = sender.send(result);
    });

    match receiver.recv_timeout(Duration::from_millis(2500)) {
        Ok(result) => result,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(format!(
            "{pipe} aceptó la conexión pero no confirmó la lectura en 2,5 s"
        )),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Err(format!(
            "el hilo de escritura en {pipe} terminó de forma inesperada"
        )),
    }
}

/// En Linux y macOS no hay named pipes: el despacho por pipe no aplica.
#[cfg(not(windows))]
pub fn send_via_pipe(_pipe: &str, _envelope: &Value) -> Result<String, String> {
    Err("el transporte por named pipe solo está disponible en Windows".to_string())
}

/// Entrega el envoltorio por HTTP local (`127.0.0.1`).
pub async fn send_via_http(
    port: u16,
    path: &str,
    envelope: &Value,
    timeout_ms: u64,
) -> Result<String, String> {
    let url = format!("http://127.0.0.1:{port}{path}");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .map_err(|error| format!("no se pudo crear el cliente HTTP: {error}"))?;

    let response = client
        .post(&url)
        .json(envelope)
        .send()
        .await
        .map_err(|error| format!("no se pudo conectar con {url}: {error}"))?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if status.is_success() {
        Ok(format!("orden entregada a {url} (HTTP {status})"))
    } else {
        Err(format!(
            "{url} respondió HTTP {status}: {}",
            truncate(&body, 200)
        ))
    }
}

/// Abre un esquema URI mediante el opener inyectado.
pub fn open_uri(opener: &UriOpener, uri: &str) -> Result<String, String> {
    let cleaned = uri.replace('"', "");
    opener(&cleaned).map(|()| format!("orden entregada vía {cleaned}"))
}

/// Registro de auditoría local de las órdenes ejecutadas por agentes.
///
/// Contrato: "Toda orden ejecutada por un agente a través de Synapse queda
/// registrada en el log local con marca de tiempo, agente emisor, acción y
/// parámetros ejecutados."
pub fn audit(entry: &AuditEntry) {
    let line = serde_json::to_string(entry).unwrap_or_else(|_| "{}".to_string());
    let path = audit_log_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        use std::io::Write;
        let _ = writeln!(file, "{line}");
    }
}

/// Una entrada del registro de auditoría.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AuditEntry {
    pub timestamp_utc: String,
    pub source_app: String,
    pub target_app: String,
    pub action: String,
    pub transport: String,
    pub status: String,
    pub detail: String,
    pub payload_preview: String,
}

/// Ruta del log de auditoría.
///
/// Windows: `%LOCALAPPDATA%\biglexj\aurora-synapse\audit.log`.
/// Linux y macOS: `$XDG_STATE_HOME/biglexj/aurora-synapse/audit.log`.
pub fn audit_log_path() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("XDG_STATE_HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    base.join("biglexj")
        .join("aurora-synapse")
        .join("audit.log")
}

/// Recorta un texto para el registro sin volcar datos masivos.
pub fn truncate(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    let head: String = value.chars().take(limit).collect();
    format!("{head}…")
}

/// Resumen corto y seguro de un payload para el log de auditoría.
pub fn payload_preview(payload: &Value) -> String {
    let text = payload.to_string();
    truncate(&text, 400)
}

/// Normaliza y valida una ruta de archivo recibida por un agente.
///
/// Contrato de seguridad del protocolo: "Todo parámetro de ruta de archivo
/// recibido por URI debe pasar por validación de seguridad (PathGuard)".
pub fn sanitize_path(raw: &str) -> Result<PathBuf, String> {
    let trimmed = raw.trim().trim_matches('"');
    if trimmed.is_empty() {
        return Err("la ruta está vacía".to_string());
    }
    if trimmed.contains('\0') {
        return Err("la ruta contiene caracteres no válidos".to_string());
    }
    let path = PathBuf::from(trimmed);
    if path
        .components()
        .any(|component| component.as_os_str() == "..")
    {
        return Err("la ruta no puede contener segmentos '..'".to_string());
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_envoltorio_cumple_el_contrato() {
        let envelope = build_envelope("luna", "enqueue_download", json!({"url": "https://x.test"}));
        assert_eq!(envelope["synapse_version"], "1.0");
        assert_eq!(envelope["source_app"], MCP_SOURCE_APP);
        assert_eq!(envelope["target_app"], "luna");
        assert_eq!(envelope["action"], "enqueue_download");
        assert!(envelope["idempotency_key"]
            .as_str()
            .unwrap()
            .starts_with("syn-"));
        assert!(envelope["timestamp_utc"].as_str().unwrap().ends_with('Z'));
    }

    #[test]
    fn las_claves_de_idempotencia_no_se_repiten() {
        assert_ne!(idempotency_key(), idempotency_key());
    }

    #[test]
    fn sanitize_path_rechaza_recorridos() {
        assert!(sanitize_path("C:\\Users\\biglexj\\Music").is_ok());
        assert!(sanitize_path("../../etc/passwd").is_err());
        assert!(sanitize_path("  ").is_err());
    }

    #[test]
    fn truncate_recorta_sin_romper_utf8() {
        assert_eq!(truncate("hola", 10), "hola");
        assert_eq!(truncate("áéíóú", 3), "áéí…");
    }
}
