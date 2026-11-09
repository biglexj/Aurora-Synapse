//! Adaptador MCP para DaVinci Flow.
//!
//! DaVinci Flow publica sus operaciones como servidor JSON-RPC 2.0 sobre stdio
//! (`python -m davinci_flow.mcp_server`). Este adaptador localiza el intérprete
//! Python correspondiente (entorno virtual del repositorio o ejecutable instalado)
//! y despacha las llamadas de herramientas MCP garantizando aislamiento, tiempo
//! límite y auditoría.

use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use super::routes;

/// Tiempo de espera predeterminado para operaciones normales.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Tiempo de espera extendido para descargas de medios (`fetch_media`).
const FETCH_MEDIA_TIMEOUT: Duration = Duration::from_secs(150);

/// Consulta el estado vivo de DaVinci Flow y su conexión con Resolve.
pub async fn status() -> Result<Value, String> {
    call_tool("davinciflow_status", &json!({})).await
}

/// Invoca una herramienta de DaVinci Flow mediante su servidor MCP por stdio.
pub async fn call_tool(name: &str, arguments: &Value) -> Result<Value, String> {
    let python = routes::davinci_python_exe();
    let repo_dir = routes::davinci_repo_dir();

    if !python.is_file() && python.to_string_lossy() != "python" {
        return Err(format!(
            "No se encontró el entorno Python de DaVinci Flow en {}",
            python.display()
        ));
    }

    let mut cmd = tokio::process::Command::new(&python);
    cmd.arg("-m")
        .arg("davinci_flow.mcp_server")
        .current_dir(&repo_dir)
        .env("PYTHONPATH", repo_dir.join("src"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("No se pudo iniciar el proceso de DaVinci Flow: {e}"))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "No se pudo abrir stdin de DaVinci Flow".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "No se pudo abrir stdout de DaVinci Flow".to_string())?;

    let init_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": { "protocolVersion": "2025-06-18" }
    });
    let initialized_notif = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    let tool_call_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": name,
            "arguments": arguments
        }
    });

    let payload = format!(
        "{}\n{}\n{}\n",
        init_req,
        initialized_notif,
        tool_call_req
    );

    stdin
        .write_all(payload.as_bytes())
        .await
        .map_err(|e| format!("Error escribiendo a DaVinci Flow: {e}"))?;
    stdin
        .flush()
        .await
        .map_err(|e| format!("Error en flush a DaVinci Flow: {e}"))?;
    drop(stdin);

    let mut reader = BufReader::new(stdout).lines();
    let timeout_duration = if name == "davinciflow_fetch_media" {
        FETCH_MEDIA_TIMEOUT
    } else {
        DEFAULT_TIMEOUT
    };

    let read_future = async {
        let mut target_response: Option<Value> = None;
        while let Ok(Some(line)) = reader.next_line().await {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
                if parsed.get("id").and_then(Value::as_u64) == Some(2) {
                    target_response = Some(parsed);
                    break;
                }
            }
        }
        target_response
    };

    let response = tokio::time::timeout(timeout_duration, read_future)
        .await
        .map_err(|_| format!("DaVinci Flow superó el tiempo límite para {name}"))?
        .ok_or_else(|| "DaVinci Flow terminó sin responder a la solicitud".to_string())?;

    let _ = child.kill().await;

    if let Some(err) = response.get("error") {
        let msg = err
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("error desconocido");
        return Err(format!("DaVinci Flow devolvió error: {msg}"));
    }

    let result = response
        .get("result")
        .ok_or_else(|| "Respuesta de DaVinci Flow sin campo 'result'".to_string())?;

    if result.get("isError").and_then(Value::as_bool) == Some(true) {
        let msg = result
            .get("content")
            .and_then(Value::as_array)
            .and_then(|arr| arr.first())
            .and_then(|c| c.get("text"))
            .and_then(Value::as_str)
            .unwrap_or("Error en herramienta de DaVinci Flow");
        return Err(msg.to_string());
    }

    if let Some(structured) = result.get("structuredContent") {
        Ok(structured.clone())
    } else if let Some(content) = result.get("content").and_then(Value::as_array) {
        if let Some(first) = content.first() {
            if let Some(text) = first.get("text").and_then(Value::as_str) {
                if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                    return Ok(parsed);
                }
                return Ok(json!({ "text": text }));
            }
        }
        Ok(json!(content))
    } else {
        Ok(result.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn davinci_status_succeeds_when_env_present() {
        let python = routes::davinci_python_exe();
        if !python.is_file() {
            return;
        }
        let res = status().await;
        assert!(res.is_ok(), "davinciflow_status falló: {:?}", res.err());
        let val = res.unwrap();
        assert!(val.get("proveedor").is_some());
    }
}
