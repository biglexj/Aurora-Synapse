# Contrato de datos del panel de estado del MCP

Fecha: 2026-10-27. Alcance: interfaz de Aurora Synapse. Sustituye el contrato del 2026-10-23, que no incluía la actividad de sesión ni los campos `supported`/`reachable`.

## Qué muestra el panel

El botón **MCP** de la cabecera abre un panel con tres bloques separados. Los tres conceptos no son intercambiables y la interfaz nunca los mezcla:

1. **Servidor MCP activo**: el puente de Aurora Synapse está escuchando y responde. Es una propiedad del proceso local.
2. **Aplicación disponible**: la aplicación satélite responde ahora mismo, o el adaptador la soporta aunque no esté disponible.
3. **Agente conectado**: hay un cliente MCP con una sesión HTTP viva (actividad aceptada dentro de la ventana). Un servidor encendido NO implica un agente conectado.

## Comando nativo implementado

`mcp_server_status` está registrado en `src-tauri/src/lib.rs` y construye su payload en `src-tauri/src/mcp/status.rs`:

```rust
#[tauri::command]
async fn mcp_server_status() -> mcp::status::McpServerStatus;
```

El frontend lo invoca así (`src/features/mcp/mcpService.ts`):

```ts
const raw = await invoke<McpServerStatus>("mcp_server_status");
```

En binarios donde el comando no exista, el adaptador degrada de forma explícita al estado **"No disponible"**, sin datos simulados.

## Payload esperado

Convención del proyecto: `snake_case`, igual que `AppTarget` y `DeviceNode`.

```rust
#[derive(Serialize)]
struct McpServerStatus {
    running: bool,
    transport: String,             // "http" | "none"
    endpoint: Option<String>,      // p. ej. "http://127.0.0.1:49296/mcp"
    host: Option<String>,          // siempre loopback
    port: Option<u16>,
    protocol_version: Option<String>,
    api_key_required: bool,
    tool_count: usize,
    tools: Vec<McpToolInfo>,
    apps: Vec<McpAppInfo>,
    agents: Vec<McpAgentInfo>,
    agent_connected: Option<bool>, // None = sin evidencia, NO "desconectado"
    started_at: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct McpToolInfo {
    name: String,
    description: Option<String>,
    app_id: String,                // aurora_synapse | super_gallery | luna_fetch | prisma | prisma_upscaler
    read_only: bool,
    destructive: bool,
}

#[derive(Serialize)]
struct McpAppInfo {
    id: String,
    name: String,
    supported: bool,               // el puente publica herramientas para esta app
    reachable: bool,               // la app respondió en la sonda de este panel
    availability: Option<String>,  // "available" | "offline"
    detail: Option<String>,
}

#[derive(Serialize)]
struct McpAgentInfo {
    client: String,
    connected: bool,               // sesión viva en el instante de la consulta
    connected_at: Option<String>,  // ISO-8601 UTC de creación
    last_activity_at: Option<String>, // ISO-8601 UTC de la última petición aceptada
    idle_seconds: Option<u64>,     // segundos desde esa petición
    protocol_version: Option<String>,
}
```

## Semántica de cada campo

- `running`: el puente escucha y responde. No se deduce de que el proceso de la app esté vivo. Con `false`, la interfaz no debe afirmar apps ni agentes: el payload llega vacío y `agent_connected` es `null`.
- `endpoint` / `host` / `port`: dirección efectiva. El puerto por defecto es `127.0.0.1:49296` (`mcp::server::DEFAULT_BIND_ADDR`); `mcp::start` prueba hasta diez puertos siguientes si está ocupado, así que hay que publicar el real. Un arranque con `--mcp-http` fija el puerto y no busca alternativas.
- `transport`: `"http"` mientras el puente está publicado. El modo stdio no publica runtime en el proceso de la interfaz; no se presenta como puente activo en el panel.
- `tools[].app_id`: agrupación por aplicación; si el backend lo omite, la interfaz lo deduce del prefijo real (`gallery_`, `luna_`, `upscaler_`, `prisma_`, resto = `aurora_synapse`).
- `apps[].supported`: el puente publica herramientas MCP para esa app. Gallery-DL figura con `false` porque su integración dedicada está aplazada, aunque su sonda pueda responder.
- `apps[].reachable` / `availability`: `true`/`"available"` es evidencia de respuesta en la sonda actual; `false`/`"offline"` es evidencia de que no responde. Sin ninguno de los dos, la interfaz muestra **"Desconocido"**. `supported: true` no convierte una app ausente en disponible.
- `agents`: solo aparecen clientes con sesión HTTP viva. Una sesión viva significa actividad aceptada dentro de la ventana de inactividad; no es un socket permanente.
- `connected_at` y `last_activity_at` permiten distinguir un cliente que acaba de hablar de uno que lleva minutos callado. `idle_seconds` es la edad de la última petición aceptada.
- `agent_connected`: `Some(false)` = no hay sesiones; `Some(true)` = hay al menos una; `None` = el backend no puede saberlo. La interfaz nunca convierte `None` en "conectado" ni en "desconectado".
- Consultar el panel **no renueva** ninguna sesión: solo la lectura de estado. Una petición MCP aceptada sí la renueva.

## Sesiones y caducidad

- Las sesiones HTTP caducan tras **300 s** sin peticiones aceptadas. Una petición con sesión válida renueva `last_activity_at`; las peticiones rechazadas (404 de sesión, JSON inválido o versión incorrecta) no renuevan.
- Tras un **HTTP 404** con `session_not_found`, el cliente debe enviar `initialize`; el servidor responde con un `Mcp-Session-Id` nuevo y retira el anterior. Reenviar la cabecera caducada en `initialize` es la vía de recuperación prevista.
- `DELETE /mcp` cierra la sesión de ese cliente sin afectar a las demás.
- El panel puede seguir mostrando un cliente hasta **300 s** después de que haya desaparecido: es el coste de no mantener un socket ni una conexión permanente.
- Un cliente **stdio** no aparece en el panel: se ejecuta en otro proceso y su canal no es una sesión HTTP. Un cliente HTTP **sin `initialize`** puede operar de forma permisiva, pero no crea sesión; por tanto, tampoco aparece.
- Con el binario actual, el refresco periódico de un consumidor (por ejemplo, el puente de Aurora/Ely cada 30 s) mantiene viva su sesión. Un consumidor esporádico que deje pasar más de 300 s recibirá 404 en su primera llamada; debe reinicializar o reintentar solo lecturas.

## Advertencia de binario instalado

El 2026-10-27 la app instalada en ejecución era la 1.0.1 del 26-10 a las 21:30, anterior a los cambios actuales. Una sonda creada a las 09:17:55 seguía viva después de 5 minutos: ese binario **no aplica la caducidad**. Su panel mostraba 8 sesiones retenidas que no pueden interpretarse como 8 agentes vivos.

Hasta reiniciar con la compilación actual, ese panel no es evidencia fiable. Las pruebas de esta entrega usaron el binario recién compilado en un puerto loopback temporal.

## Verificación

```powershell
bun scripts/mcp/verify-mcp-panel.mjs          # 28 comprobaciones
bun scripts/mcp/verify-foundation.mjs         # sonda rápida de sesiones y transportes
bun scripts/mcp/verify-foundation.mjs --slow  # caducidad real (unos 5 min)
```

La verificación del panel cubre la normalización del payload, la agrupación por aplicación, el tri-estado de disponibilidad, la regla que impide afirmar un agente conectado sin evidencia, la presentación de `idle_seconds` y la degradación cuando el comando nativo no existe. Resultados y límites completos: [FOUNDATION.md](FOUNDATION.md) y [VALIDATION](../../process/active/2026-10-27_sesiones-y-conexion-mcp/VALIDATION.md).
