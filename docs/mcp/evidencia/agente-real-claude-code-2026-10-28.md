# Evidencia: cliente de agente real sobre el MCP de Synapse

Fecha: 2026-10-28. Proceso: [SYN-110-MCP](../../../process/active/2026-10-27_sesiones-y-conexion-mcp/PLAN.md).

Resumen redactado de una sesión real de agente. No se copia la transcripción
cruda: se anotan las llamadas nativas registradas y los resultados observados.

## Configuración

Cliente: Claude Code (`claude -p`), cliente MCP nativo por stdio.

```jsonc
{
  "mcpServers": {
    "aurora-synapse": {
      "command": "D:/Proyectos/biglexj/Aurora-Synapse/src-tauri/target/debug/aurora-synapse.exe",
      "args": ["--mcp-stdio"]
    }
  }
}
```

Invocación:

```powershell
claude -p "<petición de solo lectura>" `
  --mcp-config <ruta>/mcp.json --strict-mcp-config `
  --permission-mode dontAsk `
  --allowedTools "mcp__aurora-synapse__apps_list" `
                "mcp__aurora-synapse__classify_intent" `
                "mcp__aurora-synapse__ListMcpResourcesTool" `
                "mcp__aurora-synapse__ReadMcpResourceTool" `
  --output-format stream-json --verbose
```

El binario es el de `src-tauri/target/debug/`, compilado en esta pasada. No
lleva credencial: el transporte stdio usa el canal de proceso local.

## Registro nativo de herramientas

Las 20 herramientas del catálogo quedaron registradas con el prefijo nativo
`mcp__aurora-synapse__`:

```
apps_list          classify_intent     dispatch
gallery_devices    gallery_download    gallery_list
gallery_send       gallery_status
luna_analyze       luna_download       luna_focus
luna_open_folder   luna_status
prisma_control     prisma_play         prisma_playback
prisma_status
upscaler_models    upscaler_scale      upscaler_status
```

## Llamadas nativas ejecutadas

| # | Llamada nativa | Argumentos | Resultado observado |
| --- | --- | --- | --- |
| 1 | `ListMcpResourcesTool` | — | El servidor expone un recurso: `aurora-synapse://capabilities` (`synapse_capabilities`, `application/json`) |
| 2 | `ReadMcpResourceTool` | uri `aurora-synapse://capabilities` | Contenido con `apps`, `generic_tools`, `limits`, **`session`**, `tool_count: 20`, `protocol_version: 2025-06-18` |
| 3 | `mcp__aurora-synapse__classify_intent` | `input: "https://www.youtube.com/watch?v=synapse_real_client"` | `detected_type: stream_video`; `recommended_app_id: luna` |

## Comprobación que hizo el propio agente

El agente leyó del recurso que `session.idle_timeout_seconds = 300` y
contrastó el recuento del catálogo: 5 (supergallery) + 5 (luna) + 4 (prisma) +
3 (upscaler) + 3 (genéricas) = 20, coherente con `tool_count: 20`.

## Alcance de esta evidencia

- Es **registro nativo** en el chat del agente: las herramientas se invocaron
  por su nombre nativo, no por una sonda externa que simule la conexión.
- Todas las llamadas fueron de **solo lectura**. No se descargó, escaló,
  transfirió ni abrió ninguna app, ni se tocaron el teléfono ni Android.
- `classify_intent` y `apps_list` son lecturas; `apps_list` consulta la
  disponibilidad real de las apps en ese instante.
- Esta evidencia no es una aprobación humana ni una conversación de Ely
  Intelligence, que sigue sin verificarse.
