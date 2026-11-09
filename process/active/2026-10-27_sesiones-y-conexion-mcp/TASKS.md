# SYN-110-MCP — tareas

Ejecución del 2026-10-27 sobre `master` @ `58a1afa`, árbol con cambios locales previos. No se crearon commits, no se cambió la versión y no se publicó ni instaló nada. Detalle de evidencia en [VALIDATION.md](VALIDATION.md).

## Alcance autorizado

- [x] Registrar revisión, cambios locales y entorno real del consumidor; contrastar la base y repetir solo las pruebas necesarias.
  - `master` @ `58a1afa`; módulo `src-tauri/src/mcp/` y sondas sin seguimiento. Rust 1.99.1, Bun 1.4.2, Node 24.22.1. El binario **instalado** en ejecución es 1.0.1 del 26-10 21:30 y no aplica la caducidad; el binario recién compilado es el usado en las pruebas.
- [x] Revisar expiración, renovación, cierre y reinicialización tras 404; definir qué acredita el indicador de conectado.
  - 300 s reales verificados en `verify-foundation.mjs --slow` (315 s de espera): la sesión sin renovar caduca, el sondeo del panel no renueva, una petición aceptada sí, y `initialize` con cabecera caducada abre sesión nueva. «Conectado» = petición HTTP aceptada dentro de la ventana; no es un socket. stdio y clientes sin `initialize` no aparecen en el panel.
- [x] Verificar que catálogo y recurso derivan de las herramientas reales y no confunden soporte con disponibilidad ni conexión de un agente.
  - `tools/list` = 20 en HTTP y stdio; `aurora-synapse://capabilities` idéntico en ambos; `availability: not_probed` en el recurso y disponibilidad viva solo con `apps_list`. Gallery-DL figura sin herramientas MCP.
- [x] Probar HTTP loopback y stdio, autenticación/orígenes, aislamiento de sesiones y errores; conservar compatibilidad.
  - Sonda rápida: 401 sin clave y con clave incorrecta, 403 origen ajeno, cierre 204, 404 y recuperación, dos sesiones aisladas, `classify_intent` de solo lectura. El cliente HTTP permisivo sin sesión sigue aceptándose (no se contabiliza).
- [x] Configurar y probar un cliente agente real con lectura de capacidades y llamada de solo lectura.
  - Codex 0.164 por HTTP y por stdio lanzó `apps_list` y obtuvo estados reales (Super Galería online, Luna offline, Prisma online, Upscaler con CLI). En HTTP la sesión pasó de 1 a 0 al cerrar. No se registró Synapse como herramienta nativa de esta interfaz de chat.
- [x] Reintentar los accesos a Aurora y contrastar runtime, autenticación, `host_bridge` y `synapse_bridge`; identificar el fallo directo con evidencia accesible.
  - Público: 200, 8 herramientas, `host_bridge=disconnected`, `synapse_bridge.configured=false` por `AURORA_SYNAPSE_MCP_URL` ausente. Directo local (5017): 8 herramientas anónimas, 59 con frontera técnica; `mcp.status` y `aurora_site_context` responden; el `-32603` anterior no se reprodujo (nombre `mcp.status` frente a `mcp_status` y disponibilidad del lab son causas plausibles, sin confirmación única).
- [ ] Resolver Aurora → Synapse **público** mediante un canal autenticado que alcance el runtime correcto.
  - **Bloqueado**: el runtime público no tiene `AURORA_SYNAPSE_MCP_URL`; aunque se definiera, no alcanza `127.0.0.1` de este PC. Requiere cambio de despliegue y canal autenticado; no autorizado/implementado aquí. El runtime local sí se verificó: `connected=true`, 20 herramientas y `synapse.apps_list` ejecutado.
- [x] Entregar instrucciones reproducibles de arranque/conexión, pruebas y límites; recibir el contrato del otro frente sin modificar su LAN.
  - `docs/mcp/FOUNDATION.md` y `UI_CONTRACT.md` actualizados. No se tocaron `peer.rs`, `devices.rs`, receptor LAN, Android, `src/features/gallery/` ni documentos globales. No hubo contrato nuevo del frente INTEROP que integrar.
- [x] Actualizar VALIDATION y APPROVAL de este frente y entregar resumen de integración al proceso global.

## Pendientes y bloqueos externos

- [ ] **Reiniciar/actualizar la app instalada** para que el binario en ejecución incluya la caducidad. Sin autorización en esta pasada. Mientras tanto, su panel muestra sesiones retenidas (8 al cierre) y no debe usarse como evidencia de agentes conectados.
- [ ] **Aurora Hub**: no había proceso Hub; `host_bridge=disconnected` e `invocation_audit=unavailable` en público y local. Validar cuando el Host esté en marcha y emparejado; no es requisito del MCP local.
- [ ] **Consumidores esporádicos**: tras 300 s de inactividad el primer `tools/call` recibe 404. El puente de Ely/Aurora limpia la sesión y rehace el handshake en la llamada siguiente; no reintenta la fallida. Un refresco de 30 s evita el caso. Si se exige cero fallos, el consumidor debe reinicializar antes de actuar o reintentar solo lecturas.
- [ ] **Conversación completa de Ely Intelligence** y registro nativo en el chat: no verificados. La evidencia de agente real usa Codex. Android y teléfono pertenecen a SYN-110-INTEROP.

## Coordinación

- Archivos modificados en este frente: `scripts/mcp/verify-foundation.mjs`, `docs/mcp/FOUNDATION.md`, `docs/mcp/UI_CONTRACT.md`, este proceso (`TASKS`, `VALIDATION`, `APPROVAL`).
- Sin cambios en código Rust **en esta primera pasada**: la base existente superó las pruebas; no se añadieron adaptadores ni se alteró el contrato compartido. El cambio del guion es una ampliación de la sonda (`--slow`) para verificar la caducidad real. La segunda ejecución (2026-10-28) sí cambió el módulo MCP: ver la sección correspondiente.
- Binario probado: 09:13:45, SHA-256 `3235C099BD9A16A10CEEDEEEE0B7DEB0643964E906D5FF58B60C165D344CB7C7`.
- El frente INTEROP modificó `peer.rs` y `lib.rs` durante esta pasada (09:16 y 09:21) y tenía compilación activa desde las 09:31. No se tocaron, revirtieron ni se les atribuyen esos cambios. No se lanzaron compilaciones propias después de detectarlo, para no compartir artefactos. Su cambio posterior requiere compilación y prueba de integración propias.

## Segunda ejecución — 2026-10-28

Se ejecutó el frente de nuevo sobre el mismo `master` @ `58a1afa`, con el árbol
de cambios locales previo. Esta vez **sí se cambió el código** del módulo MCP:
la base anterior cumplía las pruebas, pero dejaba tres huecos de contrato
—renovación por petición rechazada, sesiones fantasma al reinitialize y 404 sin
guía— que se han cerrado.

- [x] Revisar expiración, renovación, cierre y reinicialización tras 404; definir qué acredita el indicador de conectado.
  - **Código cambiado** en `src-tauri/src/mcp/`: `SessionStore` separa *consulta* (`get`, no renueva) de *renovación* (`renew`/`touch`); el servidor renueva **solo** tras validar la petición; `initialize` retira la sesión anterior; el 404 lleva `error`, `detail` e `idle_timeout_seconds`; `initialize` reabre sesión aunque el cliente reenvíe la cabecera caducada.
  - `SessionMeta` registra `last_activity_utc` e `idle_seconds`; el panel los expone para que «Conectado» signifique actividad reciente y no un socket.
  - Pruebas Rust: 39 del módulo MCP → **48**; con las 14 de `peer.rs` del frente paralelo, **62** en total. Todas aprobadas.
- [x] Verificar que catálogo y recurso derivan de las herramientas reales y no confunden soporte con disponibilidad ni conexión de un agente.
  - El bloque `session` del recurso `aurora-synapse://capabilities` publica el contrato: cabecera, ventana de inactividad, qué renueva, qué no, cómo se recupera y que la sesión **no** es una credencial.
- [x] Probar HTTP loopback y stdio, autenticación/orígenes, aislamiento de sesiones y errores; conservar compatibilidad.
  - `verify-foundation.mjs` ampliado: clave incorrecta, origen ajeno, aislamiento entre dos clientes, recuperación tras 404 y contrato de sesión en el recurso. Sin ejecución mutante.
- [x] Configurar y probar un cliente agente real con lectura de capacidades y llamada de solo lectura.
  - **Claude Code conectado como cliente MCP nativo por stdio**: `ListMcpResourcesTool`, `ReadMcpResourceTool` y `mcp__aurora-synapse__classify_intent` registradas y ejecutadas en el chat. Es **registro nativo**, no una sonda JSON-RPC.
  - Reproducible con `bun scripts/mcp/verify-agent-client.mjs`. Evidencia: [docs/mcp/evidencia/agente-real-claude-code-2026-10-28.md](../../../docs/mcp/evidencia/agente-real-claude-code-2026-10-28.md).
- [x] Reintentar los accesos a Aurora y contrastar runtime, autenticación, `host_bridge` y `synapse_bridge`; identificar la causa del fallo directo con evidencia accesible.
  - Se arrancó el **backend real** de Aurora con `APP_HOST=127.0.0.1` (solo loopback) y se detuvo al terminar. MCP directo: `initialize` → `aurora-mcp 1.0.0`, 17 herramientas remotas, 59 en el control plane con 20 `synapse.*`.
  - Aurora → Synapse: **`synapse_bridge` `configured: true`, `connected: true`, `tools: 20`**, lectura real del puente en este PC. Aurora Hub: `host_bridge: disconnected` porque el proceso no está en marcha.
  - Evidencia: [docs/mcp/evidencia/aurora-backend-real-2026-10-28.md](../../../docs/mcp/evidencia/aurora-backend-real-2026-10-28.md).
- [ ] Resolver Aurora → Synapse **público** mediante un canal autenticado que alcance el runtime correcto.
  - **Sigue bloqueado**: el runtime publicado no tiene `AURORA_SYNAPSE_MCP_URL` y, aunque la tuviera, no alcanza el loopback de este PC. No se modificó ningún despliegue.
- [x] Entregar instrucciones reproducibles de arranque/conexión, pruebas y límites; recibir el contrato del otro frente sin modificar su LAN.
  - `docs/mcp/SESIONES.md` (documento canónico del frente), `docs/mcp/mcp-client.ejemplo.json` (configuración lista para copiar, sin secretos) y las sondas de `scripts/mcp/`.
- [x] Actualizar VALIDATION y APPROVAL de este frente y entregar resumen de integración al proceso global.

### Pendientes de esta segunda ejecución

- [ ] **Reiniciar la aplicación** para que el binario en ejecución recoja el contrato nuevo: el 2026-10-28 el puente en `127.0.0.1:49296` respondió al 404 sin `detail`, o sea con el binario anterior. No se reinició la interfaz del usuario.
- [ ] **Estado interno del runtime público de Aurora**: la sonda lo lee (`synapse_bridge.configured=false`, `host_bridge=disconnected`); una llamada directa a `mcp_status` sin credencial devuelve `invalid_token`. Lo pendiente no es leerlo, sino el canal autenticado hasta el PC.
- [ ] **Canal remoto autenticado** para Aurora → Synapse: dependencia externa, no resoluble en este repositorio.

### Archivos de la segunda ejecución (2026-10-28)

- `src-tauri/src/mcp/protocol.rs`, `server.rs`, `status.rs`, `capabilities.rs`: contrato de sesión.
- `src/features/mcp/types.ts`, `mcpService.ts`, `McpStatusPanel.tsx`: actividad del agente en el panel.
- `scripts/mcp/verify-foundation.mjs` (ampliado), `scripts/mcp/verify-agent-client.mjs` (nuevo).
- `docs/mcp/SESIONES.md`, `docs/mcp/mcp-client.ejemplo.json`, `docs/mcp/evidencia/aurora-backend-real-2026-10-28.md` (nuevo).
- Este proceso: `TASKS`, `VALIDATION`, `APPROVAL`.
- **No se tocaron** `peer.rs`, `devices.rs`, el receptor LAN de `lib.rs`, `src/features/gallery/`, Android ni los documentos globales compartidos.
