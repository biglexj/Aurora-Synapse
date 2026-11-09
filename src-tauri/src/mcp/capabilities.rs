//! Contrato común de descubrimiento. Deriva del catálogo ejecutable: sin sondas
//! ni efectos, y sin confundir soporte de adaptador con disponibilidad actual.
use serde_json::{json, Value};

use super::{protocol, routes, tools::ToolRegistry};

pub const URI: &str = "aurora-synapse://capabilities";

pub fn descriptor() -> Value {
    json!({
        "uri": URI,
        "name": "synapse_capabilities",
        "description": "Capacidades implementadas del puente, esquemas y límites; consultar apps_list para disponibilidad actual.",
        "mimeType": "application/json"
    })
}

pub fn snapshot(registry: &ToolRegistry) -> Value {
    let tools = registry.list();
    let mut apps: Vec<Value> = routes::BRIDGE_TARGETS.iter().map(|(id, name, surface, names)| {
        json!({
            "id": id, "name": name, "surface": surface,
            "supported": !names.is_empty(),
            "availability": "not_probed",
            "tools": tools.iter().filter(|tool| names.contains(&tool["name"].as_str().unwrap_or_default())).collect::<Vec<_>>()
        })
    }).collect();
    // Gallery-DL dispone de despacho previo, pero todavía no de adaptador MCP.
    apps.push(json!({
        "id": "gallerydl", "name": "Gallery-DL GUI", "supported": false,
        "availability": "not_probed", "tools": [],
        "surface": "Despacho previo; integración MCP dedicada aplazada"
    }));
    let generic: Vec<_> = tools.iter().filter(|tool| {
        !routes::BRIDGE_TARGETS.iter().any(|(_, _, _, names)| {
            names.contains(&tool["name"].as_str().unwrap_or_default())
        })
    }).collect();
    json!({
        "schema_version": 1,
        "server": "aurora-synapse",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol_version": protocol::PROTOCOL_VERSION,
        "transports": ["http_loopback", "stdio"],
        "session_idle_timeout_seconds": protocol::SESSION_IDLE_TIMEOUT.as_secs(),
        "session": {
            "header": "Mcp-Session-Id",
            "idle_timeout_seconds": protocol::SESSION_IDLE_TIMEOUT.as_secs(),
            "renewed_by": "peticiones MCP aceptadas en el endpoint /mcp",
            "not_renewed_by": [
                "consulta del panel de estado de Synapse",
                "GET /mcp/status",
                "peticiones rechazadas (404 de sesión, JSON inválido o versión incorrecta)"
            ],
            "expires_with": "HTTP 404 y cuerpo {\"error\":\"session_not_found\"}",
            "recovery": "Tras un 404, envía una petición initialize; el servidor responde con un Mcp-Session-Id nuevo y retira el anterior.",
            "semantics": "La sesión acredita actividad reciente, no una conexión permanente. El indicador «conectado» del panel significa «petición aceptada dentro de la ventana de inactividad».",
            "authorization": "La sesión NO es una credencial. La autorización del puente es la cabecera Authorization: Bearer, si el proceso se arrancó con AURORA_SYNAPSE_MCP_KEY."
        },
        "availability_tool": "apps_list",
        "tool_count": tools.len(),
        "apps": apps,
        "generic_tools": generic,
        "delivery": {
            "sent": "Envío al transporte; no acredita ejecución",
            "accepted": "La app aceptó la orden; no acredita finalización",
            "completed": "Resultado final comprobado por el adaptador"
        },
        "limits": {
            "remote_execution": "Solo capacidades ya implementadas; no ejecución arbitraria de consola",
            "phone_pc": "Ampliaciones bidireccionales aplazadas",
            "support_is_availability": false,
            "aurora_connection": "El MCP de Aurora es otro servicio; este recurso no acredita su conexión"
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cada_herramienta_aparece_una_vez_y_con_su_esquema_real() {
        let registry = ToolRegistry::new();
        let catalog = snapshot(&registry);
        let mut collected = catalog["generic_tools"].as_array().unwrap().clone();
        for app in catalog["apps"].as_array().unwrap() {
            assert_eq!(app["availability"], "not_probed");
            collected.extend(app["tools"].as_array().unwrap().clone());
        }
        collected.sort_by_key(|tool| tool["name"].as_str().unwrap().to_string());
        let mut expected = registry.list();
        expected.sort_by_key(|tool| tool["name"].as_str().unwrap().to_string());
        assert_eq!(collected, expected);
        let gallery = catalog["apps"].as_array().unwrap().iter().find(|a| a["id"] == "gallerydl").unwrap();
        assert_eq!(gallery["supported"], false);
    }
}
