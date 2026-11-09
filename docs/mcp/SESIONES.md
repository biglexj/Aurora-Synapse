# Sesiones y conexión MCP de agentes

Fecha: 2026-10-28. Proceso: [SYN-110-MCP](../../process/active/2026-10-27_sesiones-y-conexion-mcp/PLAN.md).
Alcance: consolidating y conexión real de agentes con Synapse y Aurora, partiendo de la base común ya existente.

Este documento sustituye a la parte de sesiones y conexión de
[FOUNDATION.md](FOUNDATION.md), que queda como antecedente del proceso global.

## 1. Qué se consolidó

| Base previa | Estado actual |
| --- | --- |
| Caducidad de 5 min ya implementada | Se conserva y se documenta el ciclo exacto: qué renueva, qué no y cómo se recupera |
| Renovación implícita al resolver la sesión | Ahora solo una petición **aceptada** renueva; una petición rechazada no mantiene viva la sesión |
| `initialize` creaba una sesión nueva y abandonaba la anterior | `initialize` retira la sesión anterior: no quedan sesiones fantasma en el panel |
| 404 sin explicación | El 404 incluye `error`, `detail` e `idle_timeout_seconds`, y `initialize` reabre la sesión incluso si el cliente reenvía la cabecera caducada |
| «Conectado» sin matiz | El panel informa de la última petición aceptada (`idle_seconds`, `last_activity_at`) |
| Recurso de capacidades sin contrato de sesión | El recurso publica el contrato de sesión, su recuperación y su relación con la autorización |
| Sonda JSON-RPC como prueba de agente | Cliente de agente **real** conectado y ejecutado (sección 6) |

No se añadieron herramientas, adaptadores ni descargas. El catálogo sigue siendo
el mismo: 20 herramientas.

Los cambios de comportamiento de las filas anteriores son de la ejecución del
**2026-10-28** y tocan el módulo MCP (`src-tauri/src/mcp/`), la capa de
interfaz del panel (`src/features/mcp/`) y las sondas de
`scripts/mcp/`. La prueba Rust pasó de 43 a 51 casos. Los cambios de la
ejecución anterior (2026-10-27) fueron solo de sondas y documentación.

## 2. Ciclo de vida de la sesión

El puente MCP de Synapse usa Streamable HTTP sobre loopback. La sesión se
transporta en la cabecera `Mcp-Session-Id`.

### 2.1 Renovación

- **Renueva**: cualquier petición MCP válida y aceptada en `POST /mcp`
  (incluidas las notificaciones `notifications/*`).
- **No renueva**: la consulta del panel de estado, `GET /mcp/status`, las
  peticiones rechazadas (404 de sesión, JSON inválido, `jsonrpc` distinto de
  `2.0`) y cualquier lectura que no llegue al puente.

### 2.2 Caducidad

La sesión caduca tras **300 segundos** sin una petición aceptada. La caducidad
se aplica de forma perezosa: se purga en la siguiente interacción con el
registro de sesiones. Una sesión caducada no se reanima; hay que inicializar
de nuevo.

### 2.3 Recuperación tras un 404

`POST /mcp` con una cabecera `Mcp-Session-Id` que no existe devuelve:

```json
{
  "error": "session_not_found",
  "detail": "La sesión caducó o nunca existió. Envía una petición initialize para abrir una sesión nueva; el servidor responderá con un Mcp-Session-Id nuevo.",
  "idle_timeout_seconds": 300
}
```

`initialize` es la única vía de recuperación: **siempre** abre una sesión
nueva, incluso si la petición reenvía la cabecera caducada. El cliente debe
usar a partir de ahí el `Mcp-Session-Id` del último `initialize` recibido; el
anterior se retira para no dejar sesiones fantasma.

### 2.4 Cierre

`DELETE /mcp` con la cabecera `Mcp-Session-Id` cierra esa sesión y responde
`204`. Es idempotente: cerrar una sesión inexistente no es un error.

### 2.5 Qué NO es la sesión

La sesión **no es una credencial**. Es un contador de actividad. La
autorización del puente es la cabecera `Authorization: Bearer`, y solo existe
si el proceso se arrancó con `AURORA_SYNAPSE_MCP_KEY`. Sin esa variable, el
puente acepta cualquier proceso local: la seguridad del loopback es la única
frontera.

## 3. Semántica del panel

El panel de la cabecera sigue separando tres conceptos, y ahora dice **por
qué** cada uno es lo que es:

1. **Servidor MCP activo**: el proceso local escucha y responde.
2. **Aplicación disponible**: cada app satélite se comprueba en el momento
   contra su superficie real.
3. **Agente conectado**: hay al menos una sesión MCP viva.

«Conectado» significa *petición aceptada dentro de la ventana de
inactividad*, no un socket permanente. Por eso el panel añade:

| Campo | Significado |
| --- | --- |
| `agents[].last_activity_at` | Marca ISO-8601 de la última petición aceptada |
| `agents[].idle_seconds` | Segundos desde esa petición |
| `agents[].connected_at` | Cuándo se abrió la sesión (handshake) |

Un cliente que desaparece sigue figurando hasta cinco minutos. Es el precio de
no afirmar «desconectado» sin evidencia: el panel prefiere una señal tardía y
correcta a una afirmación inmediata y falsa.

## 4. Catálogo y recurso de capacidades

El recurso `aurora-synapse://capabilities` se deriva del registro ejecutable
(`ToolRegistry`); no existe una segunda lista de esquemas. `availability:
not_probed` es deliberado: leer el recurso no consulta apps ni prueba que
estén instaladas. Para disponibilidad en vivo está `apps_list`.

El bloque `session` del recurso publica el contrato de la sección 2, incluido
que la sesión no es una credencial.

HTTP y stdio publican **el mismo** catálogo y el mismo recurso: la
verificación compara el JSON del recurso y el array de herramientas obtenidos
por los dos transportes y exige igualdad exacta.

## 5. Transporte, autenticación y orígenes

| Aspecto | Comportamiento |
| --- | --- |
| HTTP | `http://127.0.0.1:49296/mcp`; si el puerto está ocupado se prueba el siguiente y el panel informa del real |
| Bind | Solo loopback. `serve_http` rechaza cualquier dirección que no sea loopback |
| Clave | `AURORA_SYNAPSE_MCP_KEY` activa `Authorization: Bearer`. Ausente o incorrecta → `401` |
| Orígenes | Un `Origin` ajeno a loopback → `403`. Sin cabecera `Origin` se acepta (clientes nativos) |
| Sesión | Opcional: el servidor acepta peticiones sin `Mcp-Session-Id` para tolerar clientes que degradan |
| stdio | Canal de proceso local; no usa cabeceras HTTP |

El puente **no** se expone a la LAN. Un backend remoto de Aurora no puede
alcanzar el loopback de este PC: el canal remoto autenticado es una
dependencia externa, no algo que se resuelva abriendo el puerto.

## 6. Cliente de agente real

### 6.1 Configuración reproducible (Claude Code)

```jsonc
// mcp.json — configuración del cliente, sin secretos
{
  "mcpServers": {
    "aurora-synapse": {
      "command": "D:/ruta/al/aurora-synapse/target/debug/aurora-synapse.exe",
      "args": ["--mcp-stdio"]
    }
  }
}
```

```powershell
claude -p "lista las herramientas del servidor MCP aurora-synapse" `
  --mcp-config <ruta>/mcp.json --strict-mcp-config `
  --permission-mode dontAsk `
  --allowedTools "mcp__aurora-synapse__apps_list" "mcp__aurora-synapse__classify_intent"
```

La ruta del ejecutable debe ser **absoluta y real**: el cliente la lanza como
subproceso. Un cliente que soporte MCP por HTTP usaría en su lugar
`http://127.0.0.1:<puerto real>/mcp` y, si el puente exige clave, la cabecera
`Authorization: Bearer`.

### 6.2 Qué se comprobó

Sesión real de agente contra el binario real por stdio:

- Las 20 herramientas del catálogo quedaron **registradas como herramientas
  nativas** del cliente (nombres `mcp__aurora-synapse__*`), no como resultado
  de una sonda externa.
- Se ejecutaron llamadas de **solo lectura**: `apps_list` y `classify_intent`
  con `input=https://www.youtube.com/watch?v=synapse_real_client`.
- `classify_intent` devolvió `recommended_app_id = luna` y
  `detected_type = stream_video`.
- `apps_list` informó la disponibilidad real de las apps (por ejemplo Luna
  Fetch `offline` / `reachable: false` en `127.0.0.1:49288`).

Esto **sí** es registro nativo en el chat del agente. La sonda JSON-RPC de
[verify-foundation.mjs](../../scripts/mcp/verify-foundation.mjs) es una
comprobación distinta y complementaria: verifica el contrato del servidor, no
la integración del cliente.

### 6.3 Registro nativo dentro de esta interfaz de chat (2026-10-28)

La prueba anterior usaba Codex como cliente externo. Esta ejecución conectó
**Claude Code** —el propio agente que ejecuta el frente— al puente por stdio,
con lo que las herramientas y los recursos aparecen **en el chat** como
herramientas MCP nativas, no como texto ni como salida de una sonda.

| Paso | Herramienta nativa | Resultado observado |
| --- | --- | --- |
| Listar recursos | `ListMcpResourcesTool` | Un recurso: `aurora-synapse://capabilities` |
| Leer capacidades | `ReadMcpResourceTool` | `tool_count` 20, `session.idle_timeout_seconds` 300, protocolo `2025-06-18` |
| Llamar de solo lectura | `mcp__aurora-synapse__classify_intent` | `detected_type: stream_video`, `recommended_app_id: luna` |

El agente leyó el contrato de sesión **desde el propio recurso**: el
descubrimiento por recurso y la llamada por herramienta funcionan con un
cliente de agente real. Queda registrado el nombre nativo de la herramienta
(`mcp__aurora-synapse__classify_intent`), no una simulación.

Es reproducible:

```powershell
bun scripts/mcp/verify-agent-client.mjs
```

```json
{
  "result": "ok",
  "cliente": "claude (agente real, transporte stdio)",
  "herramientas_nativas_usadas": [
    "ListMcpResourcesTool",
    "ReadMcpResourceTool",
    "mcp__aurora-synapse__classify_intent"
  ],
  "recurso_leido": "aurora-synapse://capabilities",
  "evidencia": "EVIDENCIA: (a) 20 herramientas en el catálogo; (b) session.idle_timeout_seconds = 300; (c) recommended_app_id = luna"
}
```

La configuración que usa el guion es la de la sección 6.1; también se entrega
como archivo listo para copiar en
[mcp-client.ejemplo.json](mcp-client.ejemplo.json), con la variante HTTP
sobre loopback y sin ningún secreto.

## 7. Diagnóstico por separado de los servicios de Aurora

Cuatro servicios distintos, que no deben confundirse. La sonda reproducible es
[verify-aurora-services.mjs](../../scripts/mcp/verify-aurora-services.mjs).

| Servicio | Qué es | Endpoint | Resultado de esta ejecución |
| --- | --- | --- | --- |
| **MCP directo de Aurora** | Backend Rust de Aurora, el que habla MCP de verdad (SSE + Streamable HTTP) | `http://127.0.0.1:5000/api/v1/mcp` | Correcto con el backend local en marcha (sección 7.4) |
| **Conector público Aurora MCP** | El mismo backend publicado en internet | `https://api.biglexj.com/api/v1/mcp` | Alcanzable; ver detalle en la sección 7.1 |
| **Aurora Hub** | App de escritorio Tauri. **Cliente** Host, no servidor: no expone puerto MCP propio | WS `ws://127.0.0.1:5000/api/v1/mcp/hosts/ws` | Sin proceso en marcha; su estado solo se ve desde el backend (`host_bridge`) |
| **Aurora → Synapse** | Cliente `synapse_bridge` del backend, que consume el MCP de Synapse | `AURORA_SYNAPSE_MCP_URL` en el runtime del backend | **Conectado y verificado** (sección 7.2) |

### 7.1 Conector público

La sonda lo alcanza **sin credencial** por defecto: enviar el secreto del
backend a un host público es una decisión explícita del operador
(`--auth-publica`). Con `--auth-publica` se admite `x-mcp-key` y
`Authorization: Bearer`.

La superficie pública declarada en el repositorio de Aurora es
`https://api.biglexj.com/api/v1/mcp`, con OAuth opcional y redirect
restringido al conector de ChatGPT. El repositorio **no** contiene la
configuración de despliegue del backend, así que que el endpoint esté
publicado no se deduce del código: se deduce de la respuesta observada.

Observado el 2026-10-28, sin credencial:

| Comprobación | Resultado |
| --- | --- |
| `initialize` | `200`, `serverInfo: {name: "aurora-mcp", version: "1.0.0"}`, protocolo `2025-06-18` |
| `tools/list` | `200`; catálogo remoto de herramientas de Aurora |
| `aurora_site_context` | `200`; devuelve el contexto de la plataforma |
| `mcp_status` por llamada directa sin credencial | `{"error":"invalid_token"}`: el endpoint directo exige credencial |

La sonda `verify-aurora-services.mjs` sí lee el estado del runtime publicado:
`host_bridge: disconnected`, `invocation_audit: unavailable`,
`synapse_bridge: {configured: false, reason: "AURORA_SYNAPSE_MCP_URL no está
definido"}`, `catalog_tools: 39`, 8 herramientas públicas.

**Límite:** ese estado es del runtime **publicado**. No acredita control del
PC: el runtime publicado no tiene `AURORA_SYNAPSE_MCP_URL` y, aunque lo
tuviera, una URL loopback no alcanza este equipo.

### 7.2 Aurora → Synapse

El backend de Aurora habla con Synapse como cualquier otro cliente MCP:
Streamable HTTP contra la URL literal de `AURORA_SYNAPSE_MCP_URL`, con
`MCP-Protocol-Version: 2025-06-18` y `Authorization: Bearer` si existe
`AURORA_SYNAPSE_MCP_KEY`.

- `configured` = la variable está definida en el runtime del backend.
- `connected` = solo tras un `tools/list` **exitoso**.

El estado se consulta en `GET /api/v1/mcp/synapse/status`
(`configured`, `connected`, `url`, `tools`, `last_error`).

En `backend/.env` la variable apunta a `http://127.0.0.1:49296/mcp`, que es
la dirección real del puente de Synapse. Eso es una **configuración local**:
solo funciona si el backend y Synapse están en la misma máquina. No se
presenta como control remoto del PC, y no se ha modificado ningún `.env`.

**Verificación del 2026-10-28, con el runtime de Aurora real en marcha:** el
enlace se comprobó con una lectura del puente en este PC, no con una lectura
pública.

| Comprobación | Resultado observado |
| --- | --- |
| `GET /api/v1/mcp/synapse/status` | `{"configured":true,"connected":true,"last_error":null,"tools":20,"url":"http://127.0.0.1:49296/mcp"}` |
| `mcp.status` → `synapse_bridge` | `configured: true`, `connected: true`, `tools: 20`, `last_error: null` |
| Log del backend | «🌉 [MCP] Catálogo del puente Aurora Synapse actualizado — count=20» |
| Catálogo del control plane | 59 herramientas, de las cuales **20 con prefijo `synapse.`** |

El runtime de Aurora, en este mismo equipo, refrescó el catálogo del puente
Synapse y lo publica como `synapse.*`. Es control de lectura verificado.

**Alcance:** la URL configurada es loopback. Un runtime de Aurora remoto no
puede alcanzar este puente por esa URL; para eso hace falta un canal
autenticado específico, que sigue pendiente (sección 9).

### 7.3 Aurora Hub

Aurora Hub es un **cliente** del backend (tickets + WebSocket Host), no un
servidor MCP. Por eso `host_bridge` aparece en la herramienta `mcp.status` del
backend y no en Synapse:

- `host_bridge: "ready"` → el handshake `hello` del host llegó.
- `host_bridge: "connected"` → hay un host conectado del mismo principal.
- `host_bridge: "disconnected"` → no hay host.

`invocation_audit` es la constante `"unavailable"`: no existe registro de
auditoría de invocaciones MCP implementado en Aurora.

Comprobado el 2026-10-28 con el backend real en marcha:

| Comprobación | Resultado observado |
| --- | --- |
| `GET /api/v1/mcp/hosts/status` | `{"connected":false,"ready":false,"connectionId":null,"hostId":null,"capabilities":[]}` |
| `mcp.status` → `host_bridge` | `"disconnected"` |
| Proceso Aurora Hub | No en ejecución |

La causa es concreta: **Aurora Hub no está en marcha en este equipo**. No es un
fallo del puente ni una degradación del MCP local.

### 7.4 MCP directo de Aurora, con el backend real en marcha

El backend Rust de Aurora **no estaba en ejecución** al empezar (nada
escuchando en el puerto 5000). Para diagnosticarlo se arrancó el binario ya
compilado del propio proyecto (`backend/target/debug/aurora-backend.exe`) con
`APP_HOST=127.0.0.1`, es decir **solo loopback, sin exponer su MCP a la LAN**,
y se detuvo al terminar la comprobación.

| Comprobación | Resultado observado |
| --- | --- |
| `POST /api/v1/mcp` `initialize` | `200`, `serverInfo: {name: "aurora-mcp", version: "1.0.0"}` |
| `POST /api/v1/mcp` `tools/list` | `200`; 17 herramientas del catálogo remoto de Aurora |
| `GET /api/v1/mcp/tools` | `200`; 59 herramientas, 20 de ellas `synapse.*` |
| `mcp_status` por el catálogo remoto | «Operación restringida: herramienta no disponible en el catálogo remoto» |
| `mcp.status` por el control plane | `status: executed`; `host_bridge`, `invocation_audit` y `synapse_bridge` |

**Causa del fallo directo registrado antes:** el servidor MCP de Aurora no
estaba levantado en el entorno. No era un problema de credencial del cliente.
Además, `mcp.status` es una herramienta del **control plane**, no del catálogo
remoto: invocarla por `tools/call` del MCP remoto es un error de ruta, no un
fallo de autenticación.

## 8. Verificación reproducible

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib
cargo build --manifest-path src-tauri/Cargo.toml --bin aurora-synapse
bun scripts/mcp/verify-foundation.mjs
bun scripts/mcp/verify-foundation.mjs --slow   # ~5 min; caducidad real
bun scripts/mcp/verify-mcp-panel.mjs
bun scripts/mcp/verify-agent-client.mjs         # cliente de agente real
bun scripts/mcp/verify-aurora-services.mjs
```

`verify-foundation.mjs` acepta `AURORA_SYNAPSE_BIN` para apuntar a un
ejecutable de otro directorio de compilación, útil cuando el frente paralelo
compila en el mismo checkout.

`verify-agent-client.mjs` necesita el CLI `claude` con credenciales: registra
el puente como servidor MCP y exige que el agente use las herramientas
nativas, incluida la lectura del recurso de capacidades.

Ninguna de estas pruebas descarga, escala, instala, abre apps ni muta estado
del teléfono.

## 9. Límites y bloqueos

1. **El runtime publicado de Aurora se puede leer, pero no controla este PC.**
   Informa `synapse_bridge.configured=false` (sin `AURORA_SYNAPSE_MCP_URL`) y
   `host_bridge=disconnected`. Falta un canal autenticado hasta el PC.
2. **Aurora Hub no está en marcha.** `host_bridge` refleja ese estado, no un
   fallo del puente. Su estado solo es observable desde el backend.
3. **Aurora → Synapse verificado es el enlace local.** La URL configurada es
   loopback: un runtime remoto de Aurora no alcanza este PC por esa vía. El
   canal remoto autenticado queda pendiente y no se sustituye por la lectura
   pública.
4. **La instancia de Synapse que estaba abierta antes de estos cambios
   conserva el código anterior.** El 2026-10-28 el puente en
   `127.0.0.1:49296` respondió al 404 con `{"error":"session_not_found"}` sin
   `detail`: es el binario previo. El panel y el puente hay que reiniciarlos
   para ver el contrato nuevo. No se reinició la interfaz del usuario.
5. **El backend de Aurora se levantó solo para el diagnóstico** (sección 7.4),
   con `APP_HOST=127.0.0.1` y se detuvo al terminar. Su MCP no quedó expuesto a
   la LAN en ningún momento.
