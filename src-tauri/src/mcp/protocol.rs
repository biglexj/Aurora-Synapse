//! Tipos JSON-RPC 2.0, manejo de sesión y utilidades de tiempo del servidor MCP.
//!
//! El contrato de transporte lo fija el cliente `SynapseBridge` del backend de
//! Ely Intelligence (`Aurora---Blog/backend/src/features/mcp/services/synapse_bridge.rs`):
//! Streamable HTTP con `Content-Type: application/json`, protocolo `2025-06-18`
//! y correlación por cabecera `Mcp-Session-Id`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Versión del protocolo MCP negociada en el handshake.
pub const PROTOCOL_VERSION: &str = "2025-06-18";

/// Una sesión HTTP acredita actividad reciente, no una conexión permanente.
pub const SESSION_IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);

pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const INTERNAL_ERROR: i32 = -32603;

/// Petición JSON-RPC entrante.
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[serde(default)]
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

/// Respuesta JSON-RPC saliente.
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn ok(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn err(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

/// Metadatos de una sesión MCP viva.
#[derive(Debug, Clone)]
pub struct SessionMeta {
    pub created_at_utc: String,
    pub client: String,
    pub protocol_version: String,
    /// Marca de tiempo de la última petición **aceptada** de esta sesión.
    ///
    /// Es lo que acredita el indicador "conectado": actividad reciente, no un
    /// socket permanente.
    pub last_activity_utc: String,
    last_activity: Instant,
}

impl SessionMeta {
    /// Segundos transcurridos desde la última petición aceptada.
    pub fn idle_seconds(&self) -> u64 {
        self.last_activity.elapsed().as_secs()
    }

    /// Renueva la actividad: la petición se ha aceptado y cuenta como señal.
    fn renew(&mut self) {
        self.last_activity = Instant::now();
        self.last_activity_utc = iso8601_utc(SystemTime::now());
    }
}

/// Registro de sesiones MCP activas.
///
/// El servidor es permisivo: una petición sin `Mcp-Session-Id` se acepta igual,
/// porque el cliente degrada sin sesión si el handshake no puede completarse.
#[derive(Default)]
pub struct SessionStore {
    sessions: Mutex<HashMap<String, SessionMeta>>,
    counter: AtomicU64,
}

impl SessionStore {
    pub fn create(&self, client: &str, protocol_version: &str) -> String {
        let id = format!(
            "syn-mcp-{:x}-{:x}",
            unix_nanos(),
            self.counter.fetch_add(1, Ordering::Relaxed)
        );
        let now = SystemTime::now();
        let meta = SessionMeta {
            created_at_utc: iso8601_utc(now),
            client: client.to_string(),
            protocol_version: protocol_version.to_string(),
            last_activity_utc: iso8601_utc(now),
            last_activity: Instant::now(),
        };
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.retain(|_, meta| meta.last_activity.elapsed() < SESSION_IDLE_TIMEOUT);
            sessions.insert(id.clone(), meta);
        }
        id
    }

    /// Consulta una sesión **sin** renovar su actividad.
    ///
    /// El servidor usa esta lectura para decidir si responde 404: renovar en
    /// la comprobación mantendría viva una sesión con peticiones que luego se
    /// rechazan.
    pub fn get(&self, id: &str) -> Option<SessionMeta> {
        let mut sessions = self.sessions.lock().ok()?;
        sessions.retain(|_, meta| meta.last_activity.elapsed() < SESSION_IDLE_TIMEOUT);
        sessions.get(id).cloned()
    }

    /// Indica si la sesión existe, sin tocar su actividad.
    pub fn exists(&self, id: &str) -> bool {
        self.get(id).is_some()
    }

    /// Renueva la actividad de una sesión viva y devuelve sus metadatos.
    pub fn renew(&self, id: &str) -> Option<SessionMeta> {
        let mut sessions = self.sessions.lock().ok()?;
        sessions.retain(|_, meta| meta.last_activity.elapsed() < SESSION_IDLE_TIMEOUT);
        let meta = sessions.get_mut(id)?;
        meta.renew();
        Some(meta.clone())
    }

    /// Renueva la actividad de una sesión viva sin devolver sus metadatos.
    pub fn touch(&self, id: &str) -> bool {
        self.renew(id).is_some()
    }

    /// Metadatos de todas las sesiones vivas.
    ///
    /// La interfaz los usa para saber si hay algún cliente de agente conectado:
    /// sin esta lista solo se puede afirmar el número, no quién. Consultar esta
    /// lista **no** renueva ninguna sesión.
    pub fn list(&self) -> Vec<SessionMeta> {
        self.sessions
            .lock()
            .map(|mut sessions| {
                sessions.retain(|_, meta| meta.last_activity.elapsed() < SESSION_IDLE_TIMEOUT);
                sessions.values().cloned().collect()
            })
            .unwrap_or_default()
    }

    pub fn remove(&self, id: &str) -> bool {
        self.sessions
            .lock()
            .ok()
            .map(|mut sessions| sessions.remove(id).is_some())
            .unwrap_or(false)
    }

    /// Deja la sesión como si llevara `duration` sin actividad.
    ///
    /// Existe solo en las pruebas del crate: permite comprobar la caducidad y
    /// la renovación sin esperar los cinco minutos reales.
    #[cfg(test)]
    pub fn backdate(&self, id: &str, duration: Duration) -> bool {
        let Ok(mut sessions) = self.sessions.lock() else {
            return false;
        };
        match sessions.get_mut(id) {
            Some(meta) => {
                meta.last_activity = Instant::now() - duration;
                true
            }
            None => false,
        }
    }

    pub fn len(&self) -> usize {
        self.list().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Marca de tiempo actual en ISO-8601 UTC (`2026-10-21T23:30:00Z`).
pub fn iso8601_utc(now: SystemTime) -> String {
    let secs = now
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let millis = now
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_millis())
        .unwrap_or(0);
    format!("{}T{:02}:{:02}:{:02}.{:03}Z", civil_utc(secs), (secs / 3600) % 24, (secs / 60) % 60, secs % 60, millis)
}

fn unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

/// Convierte segundos desde epoch a `(año, mes, día)` en UTC.
/// Algoritmo de días civiles de Howard Hinnant (sin dependencias).
fn civil_utc(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_utc_convierte_epoch() {
        assert_eq!(civil_utc(0), "1970-01-01");
        assert_eq!(civil_utc(1_767_225_600), "2026-01-01");
    }

    #[test]
    fn iso8601_tiene_forma_completa() {
        let stamp = iso8601_utc(SystemTime::now());
        assert!(stamp.ends_with('Z'), "{stamp}");
        assert_eq!(stamp.len(), 24, "{stamp}");
    }

    #[test]
    fn la_sesion_se_registra_y_se_elimina() {
        let store = SessionStore::default();
        let id = store.create("cliente", PROTOCOL_VERSION);
        assert!(store.get(&id).is_some());
        assert_eq!(store.len(), 1);
        assert!(store.remove(&id));
        assert!(store.is_empty());
    }

    #[test]
    fn una_sesion_inactiva_caduca_y_no_se_reanima() {
        let store = SessionStore::default();
        let id = store.create("cliente", PROTOCOL_VERSION);
        store.sessions.lock().unwrap().get_mut(&id).unwrap().last_activity =
            Instant::now() - SESSION_IDLE_TIMEOUT - Duration::from_secs(1);
        assert!(store.get(&id).is_none());
        assert!(store.list().is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn consultar_el_panel_no_renueva_la_sesion_pero_una_peticion_si() {
        let store = SessionStore::default();
        let id = store.create("cliente", PROTOCOL_VERSION);
        let previous = Instant::now() - Duration::from_secs(60);
        store.sessions.lock().unwrap().get_mut(&id).unwrap().last_activity = previous;
        // Consultar el panel (listar) no renueva: sigue inactivo.
        assert_eq!(store.list()[0].last_activity, previous);
        // Una petición aceptada sí renueva.
        assert!(store.renew(&id).unwrap().last_activity > previous);
    }

    #[test]
    fn una_peticion_aceptada_registra_su_marca_de_tiempo() {
        let store = SessionStore::default();
        let id = store.create("cliente", PROTOCOL_VERSION);
        let creada = store.get(&id).unwrap().last_activity_utc;
        // Consultar no renueva: la marca sigue siendo la de creación.
        assert_eq!(store.get(&id).unwrap().last_activity_utc, creada);
        assert!(store.exists(&id));
        // Renovar mueve el reloj interno de inactividad.
        store.sessions.lock().unwrap().get_mut(&id).unwrap().last_activity =
            Instant::now() - Duration::from_secs(30);
        assert_eq!(store.get(&id).unwrap().idle_seconds(), 30);
        assert!(store.touch(&id));
        assert!(store.renew(&id).is_some());
    }

    #[test]
    fn una_consulta_sin_renovar_no_reanima_una_sesion_caducada() {
        let store = SessionStore::default();
        let id = store.create("cliente", PROTOCOL_VERSION);
        store.sessions.lock().unwrap().get_mut(&id).unwrap().last_activity =
            Instant::now() - SESSION_IDLE_TIMEOUT - Duration::from_secs(1);
        assert!(!store.exists(&id));
        assert!(!store.touch(&id));
        assert!(store.renew(&id).is_none());
    }

    #[test]
    fn la_inactividad_se_puede_medir_en_segundos() {
        let store = SessionStore::default();
        let id = store.create("cliente", PROTOCOL_VERSION);
        store.sessions.lock().unwrap().get_mut(&id).unwrap().last_activity =
            Instant::now() - Duration::from_secs(42);
        assert_eq!(store.get(&id).unwrap().idle_seconds(), 42);
    }

    #[test]
    fn list_expone_los_clientes_de_las_sesiones_vivas() {
        let store = SessionStore::default();
        assert!(store.list().is_empty());
        store.create("Ely Intelligence", PROTOCOL_VERSION);
        store.create("agente-terminal", PROTOCOL_VERSION);

        let listado = store.list();
        assert_eq!(listado.len(), 2);
        let mut clientes: Vec<&str> = listado.iter().map(|meta| meta.client.as_str()).collect();
        clientes.sort_unstable();
        assert_eq!(clientes, vec!["Ely Intelligence", "agente-terminal"]);
        assert!(listado
            .iter()
            .all(|meta| meta.protocol_version == PROTOCOL_VERSION));
    }
}
