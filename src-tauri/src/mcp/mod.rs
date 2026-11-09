//! Puente MCP de Aurora Synapse.
//!
//! Convierte a Aurora Synapse en el **puente MCP del ecosistema Biglex**: un
//! servidor MCP (Streamable HTTP sobre loopback, con variante stdio) que
//! publica herramientas para operar las demás aplicaciones del ecosistema.
//!
//! El puente no reimplementa el protocolo de cada app: habla el dialecto real
//! que cada una expone hoy (ver [`routes`]). Cuando una app no expone ninguna
//! superficie externa, no se publica herramienta para ella en lugar de
//! prometer algo que no funciona.
//!
//! Consumidores:
//!
//! - **Ely Intelligence** (`Aurora---Blog`), mediante `SynapseBridge`, que
//!   consume `AURORA_SYNAPSE_MCP_URL`.
//! - **Agentes de terminal**, lanzando el binario con `--mcp-stdio`.

pub mod dispatch;
pub mod capabilities;
pub mod protocol;
pub mod routes;
pub mod server;
pub mod status;
pub mod tools;
pub mod devices;
pub mod gallery;
pub(crate) mod gallery_credentials;
pub mod upscaler;
pub mod davinci;
mod validation;

use std::sync::{Arc, OnceLock, RwLock};

use dispatch::UriOpener;
use server::McpState;
use tools::ToolRegistry;

/// Runtime vivo del puente MCP.
///
/// Se publica al abrir el puerto para que la interfaz pueda informar del
/// estado real (puerto efectivo, herramientas, sesiones) sin adivinarlo.
#[derive(Clone)]
pub struct BridgeRuntime {
    /// Puerto realmente escucha (puede diferir del por defecto si está ocupado).
    pub port: u16,
    /// Estado compartido del servidor MCP.
    pub state: Arc<McpState>,
    /// Marca de tiempo ISO-8601 UTC del arranque.
    pub started_at: String,
}

/// Runtime publicado del puente (`None` si el puente no está en marcha).
static BRIDGE: OnceLock<RwLock<Option<BridgeRuntime>>> = OnceLock::new();

/// Publica el runtime del puente ya escuchando.
pub fn publish_runtime(runtime: BridgeRuntime) {
    let slot = BRIDGE.get_or_init(|| RwLock::new(None));
    if let Ok(mut guard) = slot.write() {
        *guard = Some(runtime);
    }
}

/// Copia del runtime publicado, o `None` si el puente no está en marcha.
pub fn runtime() -> Option<BridgeRuntime> {
    BRIDGE
        .get()
        .and_then(|slot| slot.read().ok().and_then(|guard| (*guard).clone()))
}

/// Construye el estado compartido del servidor MCP.
pub fn build_state(opener: UriOpener) -> Arc<McpState> {
    devices::start();
    Arc::new(McpState {
        sessions: Arc::new(protocol::SessionStore::default()),
        tools: ToolRegistry::with_opener(opener.clone()),
        api_key: McpState::api_key_from_env(),
        opener,
    })
}

/// Arranca el puente MCP sobre Streamable HTTP en loopback.
///
/// Devuelve el puerto realmente escucha. Si el puerto por defecto está ocupado
/// se prueba el siguiente disponible, para que el proceso siga siendo usable.
pub async fn start(opener: UriOpener) -> std::io::Result<u16> {
    let state = build_state(opener);
    let preferred = server::DEFAULT_BIND_ADDR
        .rsplit_once(':')
        .map(|(_, port)| port.parse::<u16>().unwrap_or(49296))
        .unwrap_or(49296);

    for offset in 0..10u16 {
        let candidate = preferred + offset;
        let addr = format!("127.0.0.1:{candidate}");
        match server::serve_http(state.clone(), &addr).await {
            Ok(port) => {
                log(&format!(
                    "puente MCP escuchando en http://{addr}{} ({} herramientas)",
                    server::MCP_PATH,
                    state.tools.len()
                ));
                publish_runtime(BridgeRuntime {
                    port,
                    state: state.clone(),
                    started_at: protocol::iso8601_utc(std::time::SystemTime::now()),
                });
                return Ok(port);
            }
            Err(error) => {
                log(&format!("no se pudo abrir {addr}: {error}"));
            }
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::AddrInUse,
        "no se pudo abrir ningún puerto local para el puente MCP",
    ))
}

/// Arranca el puente MCP en un puerto exacto, sin alternativas.
///
/// Se usa en el modo headless: si el puerto ya está ocupado significa que
/// hay otro puente vivo, y arrancar un segundo crearía dos puentes distintos
/// para el mismo ecosistema.
pub async fn start_fixed(opener: UriOpener, addr: &str) -> std::io::Result<u16> {
    let state = build_state(opener);
    let port = server::serve_http(state.clone(), addr).await?;
    log(&format!(
        "puente MCP escuchando en http://{addr}{} ({} herramientas)",
        server::MCP_PATH,
        state.tools.len()
    ));
    publish_runtime(BridgeRuntime {
        port,
        state: state.clone(),
        started_at: protocol::iso8601_utc(std::time::SystemTime::now()),
    });
    Ok(port)
}

/// Ejecuta el puente MCP sobre stdio (una línea JSON-RPC por entrada).
pub async fn serve_stdio(opener: UriOpener) {
    let state = build_state(opener);
    server::serve_stdio(state).await;
}

/// Mensaje de arranque en modo depuración.
fn log(message: &str) {
    if cfg!(debug_assertions) {
        eprintln!("[aurora-synapse mcp] {message}");
    }
}
