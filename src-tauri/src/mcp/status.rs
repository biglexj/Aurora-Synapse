//! Estado real del puente MCP para la interfaz.
//!
//! El panel de la cabecera distingue tres conceptos que no son
//! intercambiables (ver `docs/mcp/UI_CONTRACT.md`):
//!
//! 1. **Servidor MCP activo**: el puente escucha y responde. Es una propiedad
//!    del proceso local, y aquí sale del runtime publicado al abrir el puerto.
//! 2. **Aplicación disponible**: cada app satélite se comprueba AHORA MISMO
//!    contra su superficie real. Que el puente esté encendido no implica que
//!    la app esté disponible.
//! 3. **Agente conectado**: hay una sesión MCP viva. Sin sesiones, la
//!    respuesta es `agent_connected: Some(false)`; sin puente, `None`
//!    ("sin evidencia"), nunca "desconectado".

use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

use crate::mcp::{protocol, routes};

/// Tiempo máximo de una sonda HTTP contra una aplicación satélite.
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Puerto por defecto de Gallery-DL GUI (respaldo si el catálogo no lo dice).
const GALLERY_DL_FALLBACK_PORT: u16 = 18274;

/// Identificador canónico de la aplicación a la que pertenece una herramienta.
const AURORA_SYNAPSE: &str = "aurora_synapse";
const SUPER_GALLERY: &str = "super_gallery";
const LUNA_FETCH: &str = "luna_fetch";
const PRISMA: &str = "prisma";
const PRISMA_UPSCALER: &str = "prisma_upscaler";
const DAVINCI_FLOW: &str = "davinci_flow";

/// Estado completo del puente MCP y de lo que tiene alrededor.
#[derive(Debug, Serialize)]
pub struct McpServerStatus {
    /// El puente está escuchando y responde.
    pub running: bool,
    /// "http" | "stdio" | "none".
    pub transport: String,
    /// Dirección completa del endpoint MCP.
    pub endpoint: Option<String>,
    /// Host efectivo del puente (siempre loopback).
    pub host: Option<String>,
    /// Puerto realmente escucha.
    pub port: Option<u16>,
    /// Versión del protocolo MCP negociada.
    pub protocol_version: Option<String>,
    /// El puente exige clave de API para aceptar conexiones.
    pub api_key_required: bool,
    /// Número de herramientas publicadas.
    pub tool_count: usize,
    /// Herramientas publicadas, con su aplicación de destino.
    pub tools: Vec<McpToolInfo>,
    /// Aplicaciones y su disponibilidad real.
    pub apps: Vec<McpAppInfo>,
    /// Clientes de agente con sesión viva.
    pub agents: Vec<McpAgentInfo>,
    /// `None` significa "sin evidencia", no "desconectado".
    pub agent_connected: Option<bool>,
    /// Marca de tiempo ISO 8601 del arranque.
    pub started_at: Option<String>,
    /// Último error conocido del puente.
    pub error: Option<String>,
}

/// Herramienta publicada por el puente.
#[derive(Debug, Serialize)]
pub struct McpToolInfo {
    pub name: String,
    pub description: Option<String>,
    /// Aplicación a la que pertenece la herramienta.
    pub app_id: String,
    pub read_only: bool,
    pub destructive: bool,
}

/// Aplicación satélite vista desde el puente MCP.
#[derive(Debug, Serialize)]
pub struct McpAppInfo {
    pub id: String,
    pub name: String,
    /// El puente publica herramientas MCP para esta aplicación.
    pub supported: bool,
    /// La aplicación responde ahora mismo.
    pub reachable: bool,
    /// "available" | "offline". Siempre explícito: la ausencia de dato la
    /// interpretaría la interfaz como "desconocido".
    pub availability: Option<String>,
    /// Detalle legible del por qué.
    pub detail: Option<String>,
}

/// Cliente de agente con una sesión MCP viva.
#[derive(Debug, Serialize)]
pub struct McpAgentInfo {
    pub client: String,
    /// Hay una sesión viva de este cliente.
    ///
    /// «Viva» significa con actividad dentro de la ventana de inactividad: no
    /// es un socket permanente. Use `idle_seconds` para saber cuánto lleva
    /// sin hablar.
    pub connected: bool,
    pub connected_at: Option<String>,
    /// Marca de tiempo de la última petición aceptada de la sesión.
    pub last_activity_at: Option<String>,
    /// Segundos desde la última petición aceptada.
    pub idle_seconds: Option<u64>,
    pub protocol_version: Option<String>,
}

/// Resultado de una sonda contra una aplicación.
struct Probe {
    reachable: bool,
    detail: String,
}

impl Probe {
    fn offline(detail: impl Into<String>) -> Self {
        Self {
            reachable: false,
            detail: detail.into(),
        }
    }

    fn online(detail: impl Into<String>) -> Self {
        Self {
            reachable: true,
            detail: detail.into(),
        }
    }
}

/// Estado del puente MCP tal y como lo consume la interfaz.
pub async fn status() -> McpServerStatus {
    let Some(runtime) = super::runtime() else {
        // Sin puente en marcha no hay evidencia de nada: ni de servidor, ni de
        // aplicaciones, ni de agentes.
        return McpServerStatus {
            running: false,
            transport: "none".to_string(),
            endpoint: None,
            host: None,
            port: None,
            protocol_version: None,
            api_key_required: false,
            tool_count: 0,
            tools: Vec::new(),
            apps: Vec::new(),
            agents: Vec::new(),
            agent_connected: None,
            started_at: None,
            error: None,
        };
    };

    let tools = collect_tools(&runtime.state.tools);
    let tool_count = tools.len();

    let (gallery, luna, prisma, upscaler, gallery_dl, davinci) = tokio::join!(
        crate::mcp::gallery::status(),
        runtime.state.tools.luna_status(),
        runtime.state.tools.prisma_status(),
        runtime.state.tools.upscaler_status(),
        probe_gallery_dl(),
        crate::mcp::davinci::status(),
    );

    let apps = vec![
        super_gallery_app(gallery),
        luna_app(luna),
        prisma_app(prisma),
        upscaler_app(upscaler),
        gallery_dl_app(gallery_dl),
        davinci_app(davinci),
    ];

    let agents: Vec<McpAgentInfo> = runtime
        .state
        .sessions
        .list()
        .into_iter()
        .map(|meta| {
            let idle_seconds = meta.idle_seconds();
            McpAgentInfo {
                client: meta.client,
                connected: true,
                connected_at: Some(meta.created_at_utc),
                idle_seconds: Some(idle_seconds),
                last_activity_at: Some(meta.last_activity_utc),
                protocol_version: Some(meta.protocol_version),
            }
        })
        .collect();

    McpServerStatus {
        running: true,
        transport: "http".to_string(),
        endpoint: Some(format!(
            "http://127.0.0.1:{}{}",
            runtime.port,
            crate::mcp::server::MCP_PATH
        )),
        host: Some("127.0.0.1".to_string()),
        port: Some(runtime.port),
        protocol_version: Some(protocol::PROTOCOL_VERSION.to_string()),
        api_key_required: runtime.state.api_key.is_some(),
        tool_count,
        tools,
        apps,
        agent_connected: Some(!agents.is_empty()),
        agents,
        started_at: Some(runtime.started_at),
        error: None,
    }
}

/// Traduce el catálogo MCP de herramientas al contrato de la interfaz.
fn collect_tools(registry: &crate::mcp::tools::ToolRegistry) -> Vec<McpToolInfo> {
    registry
        .list()
        .iter()
        .map(|tool| {
            let name = tool["name"].as_str().unwrap_or_default().to_string();
            McpToolInfo {
                app_id: app_id_for_tool(&name).to_string(),
                name,
                description: tool["description"].as_str().map(str::to_string),
                read_only: tool["annotations"]["readOnlyHint"].as_bool().unwrap_or(false),
                destructive: tool["annotations"]["destructiveHint"]
                    .as_bool()
                    .unwrap_or(false),
            }
        })
        .collect()
}

/// Aplicación canónica de una herramienta, deducida del catálogo real.
///
/// La fuente de verdad es [`routes::BRIDGE_TARGETS`]: no se inventa el
/// prefijo, se recorre la lista de herramientas que el puente publica.
fn app_id_for_tool(name: &str) -> &'static str {
    for (target, _, _, tools) in routes::BRIDGE_TARGETS {
        if tools.contains(&name) {
            return canonical_app_id(target);
        }
    }
    AURORA_SYNAPSE
}

/// Traduce el identificador de un target del puente al identificador canónico
/// que usa la interfaz.
fn canonical_app_id(target: &str) -> &'static str {
    match target {
        "supergallery" => SUPER_GALLERY,
        "luna" => LUNA_FETCH,
        "prisma" => PRISMA,
        "prisma-upscaler" => PRISMA_UPSCALER,
        "davinciflow" => DAVINCI_FLOW,
        _ => AURORA_SYNAPSE,
    }
}

/// Super Galería: estado real del teléfono configurado.
fn super_gallery_app(status: Result<Value, String>) -> McpAppInfo {
    let probe = match status {
        Ok(status) => {
            let reachable = status["reachable"].as_bool().unwrap_or(false);
            let paired = status["paired"].as_bool().unwrap_or(false);
            let configured = status["configured"].as_bool().unwrap_or(false);
            if !configured {
                Probe::offline("Sin teléfono configurado en Synapse")
            } else if reachable && paired {
                Probe::online("Teléfono autorizado y respondiendo")
            } else if reachable {
                Probe::online("Teléfono respondiendo, sin autorización de agentes")
            } else {
                Probe::offline("El teléfono configurado no responde")
            }
        }
        Err(error) => Probe::offline(error),
    };

    app_info(SUPER_GALLERY, "Super Galería", true, probe)
}

/// Luna Fetch: servidor Synapse local en `127.0.0.1:49288`.
fn luna_app(status: Result<Value, String>) -> McpAppInfo {
    let probe = match status {
        Ok(_) => Probe::online(format!(
            "HTTP {} respondiendo",
            routes::LUNA_LAN_PORT
        )),
        Err(error) => Probe::offline(format!("HTTP {}: {error}", routes::LUNA_LAN_PORT)),
    };

    app_info(LUNA_FETCH, "Luna Fetch", true, probe)
}

/// Prisma: servidor Synapse local en `127.0.0.1:49290`.
fn prisma_app(status: Result<Value, String>) -> McpAppInfo {
    let probe = match status {
        Ok(_) => Probe::online(format!(
            "HTTP {} respondiendo",
            routes::PRISMA_HTTP_PORT
        )),
        Err(error) => Probe::offline(format!(
            "HTTP {}: {error}",
            routes::PRISMA_HTTP_PORT
        )),
    };

    app_info(PRISMA, "Prisma", true, probe)
}

/// Prisma Upscaler: microservicio HTTP en `127.0.0.1:8085` o CLI local.
fn upscaler_app(status: Result<Value, String>) -> McpAppInfo {
    let probe = match status {
        Ok(status) => {
            let reachable = status["server_reachable"].as_bool().unwrap_or(false);
            let cli = status["cli_executable"].as_bool().unwrap_or(false);
            if reachable {
                Probe::online(format!(
                    "HTTP {} respondiendo",
                    routes::UPSCALER_HTTP_PORT
                ))
            } else if cli {
                Probe::offline(format!(
                    "HTTP {} no responde; CLI local instalada",
                    routes::UPSCALER_HTTP_PORT
                ))
            } else {
                Probe::offline(format!(
                    "HTTP {} no responde y no hay CLI instalada",
                    routes::UPSCALER_HTTP_PORT
                ))
            }
        }
        Err(error) => Probe::offline(format!(
            "HTTP {}: {error}",
            routes::UPSCALER_HTTP_PORT
        )),
    };

    app_info(PRISMA_UPSCALER, "Prisma Upscaler", true, probe)
}

/// Gallery-DL GUI: sonda directa a su endpoint `/status`.
///
/// El puente **no** publica herramientas MCP para Gallery-DL todavía, así que
/// `supported` es `false`; la sonda solo informa de si la app responde.
fn gallery_dl_app(probe: Probe) -> McpAppInfo {
    app_info("gallery_dl", "Gallery-DL GUI", false, probe)
}

/// DaVinci Flow: motor Python local y conexión con DaVinci Resolve.
fn davinci_app(status: Result<Value, String>) -> McpAppInfo {
    let probe = match status {
        Ok(status) => {
            let resolve_connected = status
                .get("resolve")
                .and_then(|r| r.get("connected"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if resolve_connected {
                Probe::online("Motor DaVinci Flow y conexión con Resolve activos")
            } else if let Some(err) = status
                .get("resolve")
                .and_then(|r| r.get("error"))
                .and_then(Value::as_str)
            {
                Probe::online(format!("Motor activo; Resolve: {err}"))
            } else {
                Probe::online("Motor DaVinci Flow respondiendo")
            }
        }
        Err(error) => Probe::offline(format!("DaVinci Flow: {error}")),
    };

    app_info(DAVINCI_FLOW, "DaVinci Flow", true, probe)
}

/// Construye la entrada de una aplicación con su disponibilidad explícita.
fn app_info(id: &str, name: &str, supported: bool, probe: Probe) -> McpAppInfo {
    McpAppInfo {
        id: id.to_string(),
        name: name.to_string(),
        supported,
        reachable: probe.reachable,
        availability: Some(if probe.reachable { "available" } else { "offline" }.to_string()),
        detail: Some(probe.detail),
    }
}

/// Sonda el estado de Gallery-DL GUI.
///
/// El puerto sale del catálogo real de la aplicación (no de una constante
/// escrita a mano); 18274 es solo el respaldo.
async fn probe_gallery_dl() -> Probe {
    let port = gallery_dl_port();
    let url = format!("http://127.0.0.1:{port}/status");

    let client = match reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(PROBE_TIMEOUT)
        .build()
    {
        Ok(client) => client,
        Err(error) => return Probe::offline(format!("HTTP {port}: {error}")),
    };

    match client.get(&url).send().await {
        Ok(response) => {
            let code = response.status().as_u16();
            if response.status().is_success() {
                Probe::online(format!("HTTP {port} respondiendo (HTTP {code})"))
            } else {
                Probe::offline(format!("HTTP {port} respondió HTTP {code}"))
            }
        }
        Err(error) => Probe::offline(format!("HTTP {port} no responde: {error}")),
    }
}

/// Puerto real de Gallery-DL GUI según el catálogo de la aplicación.
fn gallery_dl_port() -> u16 {
    crate::app_catalog()
        .iter()
        .find(|app| app.id == "gallerydl")
        .map(|app| app.port)
        .filter(|port| *port != 0)
        .unwrap_or(GALLERY_DL_FALLBACK_PORT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_mapeo_de_herramientas_usa_el_catalogo_real() {
        assert_eq!(app_id_for_tool("gallery_status"), SUPER_GALLERY);
        assert_eq!(app_id_for_tool("gallery_download"), SUPER_GALLERY);
        assert_eq!(app_id_for_tool("luna_status"), LUNA_FETCH);
        assert_eq!(app_id_for_tool("prisma_status"), PRISMA);
        assert_eq!(app_id_for_tool("upscaler_status"), PRISMA_UPSCALER);
        assert_eq!(app_id_for_tool("davinciflow_status"), DAVINCI_FLOW);
        assert_eq!(app_id_for_tool("davinciflow_fetch_media"), DAVINCI_FLOW);
        assert_eq!(app_id_for_tool("apps_list"), AURORA_SYNAPSE);
        assert_eq!(app_id_for_tool("classify_intent"), AURORA_SYNAPSE);
        assert_eq!(app_id_for_tool("dispatch"), AURORA_SYNAPSE);
    }

    #[test]
    fn toda_herramienta_publicada_tiene_aplicacion_canonica() {
        let registry = crate::mcp::tools::ToolRegistry::new();
        for tool in collect_tools(&registry) {
            assert!(!tool.name.is_empty());
            assert!(
                [
                    AURORA_SYNAPSE,
                    SUPER_GALLERY,
                    LUNA_FETCH,
                    PRISMA,
                    PRISMA_UPSCALER,
                    DAVINCI_FLOW,
                ]
                .contains(&tool.app_id.as_str()),
                "{}",
                tool.name
            );
        }
    }

    #[test]
    fn el_catalogo_publica_todas_las_herramientas_del_contrato() {
        let registry = crate::mcp::tools::ToolRegistry::new();
        let herramientas = collect_tools(&registry);
        assert_eq!(herramientas.len(), 27);
        assert_eq!(herramientas.iter().filter(|t| t.app_id == SUPER_GALLERY).count(), 5);
        assert_eq!(herramientas.iter().filter(|t| t.app_id == DAVINCI_FLOW).count(), 7);
    }

    #[test]
    fn la_disponibilidad_es_siempre_explicita() {
        let info = app_info(
            "luna_fetch",
            "Luna Fetch",
            true,
            Probe::offline("HTTP 49288: no se pudo conectar"),
        );
        assert_eq!(info.availability.as_deref(), Some("offline"));
        assert!(!info.reachable);
        assert!(info.detail.unwrap().contains("49288"));
    }

    #[tokio::test]
    async fn sin_puente_la_interfaz_no_affirma_nada() {
        if super::super::runtime().is_some() {
            // Otro test de este proceso ya arrancó el puente: no aplica.
            return;
        }
        let status = status().await;
        assert!(!status.running);
        assert_eq!(status.transport, "none");
        assert_eq!(status.agent_connected, None);
        assert!(status.tools.is_empty());
        assert!(status.apps.is_empty());
        assert_eq!(status.tool_count, 0);
    }

    #[test]
    fn el_puerto_de_gallery_dl_sale_del_catalogo() {
        assert_eq!(gallery_dl_port(), 18274);
    }
}
