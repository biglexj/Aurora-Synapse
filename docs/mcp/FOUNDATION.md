# Base común MCP — objetivo 1.1.0

Alcance autorizado el 2026-10-26: reutilizar el puente existente, consolidar descubrimiento y sesiones, y probar la conexión de agentes con el MCP de Aurora. Descargas, nuevas integraciones e interacción teléfono ↔ PC se aplazan. La versión del código sigue siendo 1.0.1; este frente no la cambia ni publica builds.

Revalidado el 2026-10-28 sobre la revisión Git `58a1afa` (árbol de trabajo con cambios locales; el módulo MCP está sin seguimiento). Los resultados anteriores son antecedentes, no evidencia de esta pasada. El detalle de sesiones y conexión está en [SESIONES.md](SESIONES.md).

> **Antecedente.** Hubo una ejecución anterior de este frente el 2026-10-27 que dejó la documentación y el guion de sondas, pero **sus cambios de código Rust no están en el árbol**. Esta pasada cierra ese hueco: implementa el contrato de sesión en el código, lo revalida y deja el árbol y la documentación coherentes entre sí.

## Contraste de implementación

| Base | Antes | Entrega actual |
| --- | --- | --- |
| Transporte | HTTP loopback y stdio implementados | Se conservan; probados con cliente JSON-RPC y con un cliente de agente real en ambos transportes |
| Herramientas | 20 herramientas y validación de argumentos | Se conserva el catálogo; no se añaden adaptadores |
| Descubrimiento | `tools/list` y `apps_list`; recursos vacíos | `aurora-synapse://capabilities` derivado del registro ejecutable, idéntico por HTTP y stdio; disponibilidad actual solo con `apps_list` |
| Estado | Comando nativo del panel implementado, contrato documental obsoleto | Contrato actualizado; panel distingue servidor, app y agente, e informa de la última actividad de la sesión |
| Sesiones | Renovaban al resolver la sesión, incluso si la petición se rechazaba | Solo una petición **aceptada** renueva; `initialize` recupera tras un 404 y retira la sesión anterior |
| Autenticación MCP | Bearer opcional por entorno y rechazo de orígenes web ajenos | Se conserva y se verifica, también para el recurso de capacidades |
| Agente real | Solo sonda JSON-RPC | Cliente de agente real conectado: las 20 herramientas quedaron **registradas de forma nativa** y se ejecutaron lecturas de capacidades y de catálogo |
| Binario instalado | Sin caducidad efectiva | El binario **en ejecución** (1.0.1 del 26-10 21:30) no caduca sesiones; requiere reinicio con la compilación actual, pendiente de autorización |

## Sesiones, renovación y reconexión

- Ventana de inactividad: **300 s** (`SESSION_IDLE_TIMEOUT`).
- Renueva una **petición MCP aceptada** en `/mcp`, incluidas las notificaciones con sesión. No renuevan: consultar el panel, `GET /mcp/status`, un 404 de sesión, JSON inválido ni una versión JSON-RPC incorrecta.
- Una sesión vencida responde **HTTP 404** con `{"error":"session_not_found"}`, `detail` que pide `initialize` y `idle_timeout_seconds: 300`.
- `initialize` **siempre** puede abrir una sesión nueva, aunque el cliente reenvíe la cabecera caducada. La sesión anterior se retira en el mismo acto, sin dejar fantasmas.
- `DELETE /mcp` cierra la sesión; cerrar una no afecta a las demás. El panel solo nombra clientes con sesión viva.
- Alcance del indicador: «conectado» significa **petición HTTP aceptada dentro de la ventana**, no un socket permanente. Un cliente **stdio** no aparece en el panel porque corre en otro proceso. Un cliente HTTP **sin `initialize`** puede operar de forma permisiva, pero no crea sesión y tampoco se muestra. Ambas limitaciones son deliberadas y están cubiertas en el contrato del panel.
- Consumidor tras inactividad: el primer `tools/call` después de 300 s recibe 404. El puente de Ely/Aurora limpia la sesión y rehace el handshake en la llamada siguiente, sin reintentar la operación fallida (correcto para mutaciones). Un runtime con refresco periódico de 30 s no llega a inactivarse; un cliente esporádico debe reinicializar antes de actuar si quiere evitar ese primer error.

Detalle completo: [SESIONES.md](SESIONES.md).

## Contrato de capacidades

1. `initialize` anuncia `tools` y `resources`.
2. `resources/list` publica `aurora-synapse://capabilities`.
3. `resources/read` devuelve JSON con versión de esquema, transportes, sesión, apps/adaptadores, herramientas, esquemas, anotaciones y significado de resultados.
4. `availability: not_probed` es deliberado: leer el recurso no consulta apps ni acredita instalación. `apps_list` consulta disponibilidad en vivo.
5. `sent` es envío; `accepted` es aceptación; `completed` exige resultado comprobado. Una herramienta listada no acredita el recorrido completo.

El recurso no incluye tokens, credenciales, configuración privada del teléfono ni resultados de contenido. La fuente de herramientas es `ToolRegistry`; no se mantiene una segunda lista de esquemas.

## Conexión de clientes y configuración reproducible

HTTP local (solo loopback):

```powershell
$env:AURORA_SYNAPSE_MCP_ADDR = '127.0.0.1:49296'
# Opcional: $env:AURORA_SYNAPSE_MCP_KEY = '<clave local; no registrar su valor>'
& '<ruta>\aurora-synapse.exe' --mcp-http
```

stdio:

```powershell
& '<ruta>\aurora-synapse.exe' --mcp-stdio
```

Cliente de agente real (Claude Code), sin editar la configuración global:

```jsonc
// mcp.json
{ "mcpServers": { "aurora-synapse": {
  "command": "D:/ruta/al/aurora-synapse.exe",
  "args": ["--mcp-stdio"] } } }
```

```powershell
claude -p "<petición de solo lectura>" `
  --mcp-config <ruta>/mcp.json --strict-mcp-config `
  --permission-mode dontAsk `
  --allowedTools "mcp__aurora-synapse__apps_list" "mcp__aurora-synapse__classify_intent"
```

Si el puente exige clave, el cliente HTTP puede leerla de una variable de entorno (`bearer_token_env_var`); no se escribe en el repositorio. La clave de API no es la sesión: la sesión no es una credencial.

Consumidor Aurora/Ely en un runtime **local**:

```text
AURORA_SYNAPSE_MCP_URL=http://127.0.0.1:49296/mcp
AURORA_SYNAPSE_MCP_KEY=<opcional; no registrar>
AURORA_SYNAPSE_TIMEOUT_MS=150000
```

No configurar un runtime **remoto** con `127.0.0.1`: el loopback del servidor remoto no es este PC. El canal remoto autenticado sigue pendiente y no se sustituye por una URL loopback ni abriendo el MCP a la LAN.

## Verificación ejecutada (2026-10-28)

| Comprobación | Comando | Resultado |
| --- | --- | --- |
| Pruebas Rust | `cargo test --manifest-path src-tauri/Cargo.toml --lib` | **62 aprobadas**, 0 fallidas (48 del módulo MCP, 14 del frente paralelo en `peer.rs`) |
| Compilación | `cargo build --manifest-path src-tauri/Cargo.toml --bin aurora-synapse` | Correcta; el binario se regeneró con el contrato de sesión nuevo |
| Sonda rápida | `bun scripts/mcp/verify-foundation.mjs` | `ok`: HTTP + stdio, 20 herramientas, recurso idéntico, 401 sin clave y con clave incorrecta, 403 origen ajeno, cierre, recuperación tras 404, aislamiento, `classify_intent` |
| Caducidad real | `bun scripts/mcp/verify-foundation.mjs --slow` | `ok` con **315 s** de espera real: `idle_expired`, `panel_polling_did_not_renew`, `renewed_session_alive`, `recovery_initialize_after_404`, `max_sessions_observed: 2` |
| Panel | `bun scripts/mcp/verify-mcp-panel.mjs` | **28 comprobaciones** correctas |
| Agente real | Claude Code → binario por stdio | Las 20 herramientas registradas de forma **nativa**; `ListMcpResourcesTool`, `ReadMcpResourceTool` y `mcp__aurora-synapse__classify_intent` ejecutadas; el agente leyó `session.idle_timeout_seconds = 300` del recurso |
| Aurora: cuatro superficies | `bun scripts/mcp/verify-aurora-services.mjs` | Conector público 200 con 8 herramientas; `mcp.status` con `host_bridge=disconnected`, `invocation_audit=unavailable` y `synapse_bridge.configured=false`. Con el backend local levantado en loopback: MCP directo correcto y **`synapse_bridge` `connected: true` con 20 herramientas** |

No se ejecutaron descargas, escalados, transferencias ni acciones mutantes.

## Aurora: cuatro superficies distintas

Estado observado el 2026-10-28 con [verify-aurora-services.mjs](../../scripts/mcp/verify-aurora-services.mjs). La sonda **no envía credenciales a hosts públicos** salvo que se pase `--auth-publica`.

| Superficie | Acceso comprobado | Estado observado | Qué acredita |
| --- | --- | --- | --- |
| **Aurora público** | `https://api.biglexj.com/api/v1/mcp` | 200; `serverInfo: aurora-mcp`; protocolo `2025-06-18`; **8 herramientas**; `catalog_tools: 39` | Lectura pública del servicio. **No** acredita control del PC, Hub ni Synapse |
| **Proxy público del sitio** | `https://www.biglexj.com/api/v1/mcp` | 200; mismas 8 herramientas | El proxy del sitio sirve el mismo servicio MCP |
| **MCP directo (runtime local)** | `http://127.0.0.1:5000/api/v1/mcp` | **Correcto** con el backend real en marcha: `serverInfo aurora-mcp 1.0.0`; 17 herramientas remotas; 59 en el control plane, 20 de ellas `synapse.*` | Contrato del MCP directo de Aurora, verificado |
| **Aurora Hub** | `ws://127.0.0.1:5000/api/v1/mcp/hosts/ws` (vía backend) | Sin proceso Hub; `host_bridge: disconnected`; `invocation_audit: unavailable`; `hosts/status` → `{"connected":false,"ready":false}` | No hay puente Host conectado; no se tocó su configuración |
| **Aurora → Synapse** | `GET /api/v1/mcp/synapse/status` en el runtime local | **`configured: true`, `connected: true`, `tools: 20`, `last_error: null`**; log «Catálogo del puente Aurora Synapse actualizado — count=20» | **Lectura real del puente en este PC**, no una lectura pública |

Detalle de la parte local: [evidencia del backend real](evidencia/aurora-backend-real-2026-10-28.md).

### Causa concreta del estado

- **Aurora público no puede llegar a este PC.** Su `mcp.status` informa literalmente `synapse_bridge: {configured: false, reason: "AURORA_SYNAPSE_MCP_URL no está definido"}`; además, aunque se definiera, un backend remoto no alcanza `127.0.0.1`. Falta un canal autenticado hasta el PC y el cambio de configuración del despliegue; no se hizo ni se simuló.
- **El MCP directo local sí funciona** cuando el backend de Aurora está en marcha: se arrancó su binario ya compilado con `APP_HOST=127.0.0.1` —solo loopback, sin exponer su MCP a la LAN— y se detuvo al terminar. Con él se obtuvo la lectura real del puente Synapse.
- **El fallo `-32603` anterior no se reprodujo.** El catálogo directo de Aurora publica `mcp.status` (con punto), no `mcp_status`; una llamada al nombre equivocado produce un error genérico de ese tipo. El catálogo público observed hoy confirma el nombre con punto. No se atribuye una causa única sin más evidencia.
- **Aurora Hub no está conectado.** No hay proceso Hub; el backend lo reporta desconectado. No se inició ni se modificó.

## Binario en ejecución: advertencia operativa

La app instalada en ejecución es `C:\Users\biglexj\AppData\Local\Programs\Aurora Synapse\aurora-synapse.exe`, versión **1.0.1**, fechada el 2026-10-26 21:30 — anterior al contrato de sesión actual. Esa build **no aplica la caducidad de 300 s**: el contador de sesiones de `/mcp/status` incluye sesiones inactivas retenidas y no puede interpretarse como agentes vivos.

Consecuencia: hasta reiniciar con la compilación actual, el panel de la app instalada no es evidencia fiable de agentes conectados. No se reinició ni se instaló nada; la actualización corresponde a la preparación de 1.1.0 y a una decisión de Biglex. Las pruebas de esta entrega usaron el binario recién compilado en un puerto loopback temporal.

## Bloqueos y pendientes

1. **Aurora público → Synapse:** bloqueado por configuración del despliegue (`AURORA_SYNAPSE_MCP_URL` ausente) y por la imposibilidad de alcanzar el loopback del PC. Requiere un canal autenticado y un cambio en el runtime remoto; no autorizado en este frente.
2. **Aurora Hub:** desconectado y sin proceso; validar cuando el Host esté en marcha y emparejado. No es requisito del MCP local.
3. **Backend de Aurora local:** se levantó solo para el diagnóstico, con `APP_HOST=127.0.0.1`, y se detuvo al terminar. Su MCP nunca quedó expuesto a la LAN. El canal remoto autenticado sigue pendiente.
4. **Binario instalado:** reiniciar/actualizar para aplicar la caducidad y el contrato de sesión nuevo. Sin autorización en esta pasada.
5. **Consumidores esporádicos:** tras 300 s de inactividad, el primer `tools/call` recibe 404; el consumidor debe reinicializar o reintentar solo lecturas. El refresco de 30 s evita el caso en los runtimes que lo ejecutan.
6. **Conversación de Ely Intelligence:** no verificada. No es requisito de este frente, pero sigue abierta en el proceso global.

## Referencias

- [Sesiones y conexión de agentes](SESIONES.md) y [evidencia del agente real](evidencia/agente-real-claude-code-2026-10-28.md).
- [Contrato del panel](UI_CONTRACT.md).
- [Proceso SYN-110-MCP](../../process/active/2026-10-27_sesiones-y-conexion-mcp/PLAN.md), [tareas](../../process/active/2026-10-27_sesiones-y-conexion-mcp/TASKS.md), [validación](../../process/active/2026-10-27_sesiones-y-conexion-mcp/VALIDATION.md) y [cierre](../../process/active/2026-10-27_sesiones-y-conexion-mcp/APPROVAL.md).
- [Proceso global SYN-110](../../process/active/2026-10-26_puente-universal-mcp-e-interconectividad/PLAN.md).
