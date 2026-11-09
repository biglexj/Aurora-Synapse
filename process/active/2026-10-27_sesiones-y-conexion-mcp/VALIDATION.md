# SYN-110-MCP — validación

Fecha: 2026-10-27. Estado: ejecución completada en el alcance autorizado; bloqueos externos separados. No se transfieren aprobaciones de pruebas históricas.

## Línea base

| Dato | Valor |
| --- | --- |
| Rama y revisión | `master` @ `58a1afa` |
| Árbol | Cambios locales previos extensos; `src-tauri/src/mcp/`, `src/features/mcp/`, `scripts/mcp/`, `docs/mcp/` y `process/` sin seguimiento |
| Toolchain | Rust/cargo 1.99.1, Bun 1.4.2, Node 24.22.1 |
| Binario probado | `src-tauri/target/debug/aurora-synapse.exe`, compilado 2026-10-27 09:13:45 (280.010.240 bytes), SHA-256 `3235C099BD9A16A10CEEDEEEE0B7DEB0643964E906D5FF58B60C165D344CB7C7` |
| App instalada en ejecución | 1.0.1, binario del 2026-10-26 21:30, PID 2976 desde las 07:42; **anterior** a los cambios actuales |
| Versión del proyecto | 1.0.1; sin cambio. Hito 1.1.0 no preparado en esta pasada |

No se crearon commits, no se publicó ni instaló nada, no se modificaron credenciales, despliegues, LAN ni Android.

## Evidencia por comprobación

| # | Prueba | Origen → destino | Runtime | Acción | Resultado observado | Evidencia |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Pruebas Rust | Local | `cargo test --lib` | Suite del crate | **57 aprobadas**, 0 fallidas | Salida de `cargo test` |
| 2 | Compilación | Local | `cargo build --bin aurora-synapse` | Build debug | Correcta en 22,61 s; 22 avisos no bloqueantes, sin impedir el binario | Salida de cargo |
| 3 | HTTP loopback | Cliente JSON-RPC (Bun) → `127.0.0.1:<puerto efímero>/mcp` | Binario nuevo, clave efímera | `initialize`, notificación, `resources/list`, `resources/read`, `tools/list`, `classify_intent` | 200; 20 herramientas; recurso idéntico al registro; `isError=false` | `verify-foundation.mjs`, salida `ok` |
| 4 | stdio | Proceso Bun → binario `--mcp-stdio` | Binario nuevo | Mismas peticiones por líneas JSON-RPC | Catálogo y recurso idénticos a HTTP; sin sesiones HTTP en ese proceso | `verify-foundation.mjs` |
| 5 | Autenticación y origen | Cliente → puerto efímero | Binario con clave | Sin `Authorization`; con clave incorrecta; con `Origin: https://example.com` | 401, 401 y 403 respectivamente | `verify-foundation.mjs` |
| 6 | Cierre y recuperación | Cliente → puerto efímero | Binario nuevo | `DELETE`; reutilizar sesión; `initialize` con cabecera caducada | 204; siguiente petición 404 `session_not_found`; `initialize` 200 con ID nuevo; `tools/list` 200 | `verify-foundation.mjs` |
| 7 | Aislamiento | Dos clientes → mismo puerto | Binario nuevo | Dos `initialize`; cerrar el segundo; `ping` del primero | Sesiones distintas; cerrar una no afecta a la otra | `verify-foundation.mjs` |
| 8 | **Caducidad real** | Cliente → puerto efímero | Binario nuevo | Espera de 315 s; sondeo de `/mcp/status` cada 15 s; renovar B a los ~267 s; `ping` a A y B; `initialize` con cabecera de A | A caducó (404); B viva (200); sondeo del panel **no** renovó; recuperación 200; máximo 2 sesiones | `verify-foundation.mjs --slow`: `wait_seconds=315`, `idle_expired=true`, `panel_polling_did_not_renew=true`, `renewed_session_alive=true`, `recovery_initialize_after_404=true` |
| 9 | Panel | Componentes React/TS en Bun | Contrato normalizado | 28 comprobaciones de payload, agrupación, tri-estado, agente y degradación | **28 correctas** | `verify-mcp-panel.mjs` |
| 10 | Agente real HTTP | Codex 0.164 → `127.0.0.1:49396/mcp` | Binario nuevo, puerto temporal | `initialize`, descubrimiento y `apps_list` | `apps_list` completada; Super Galería online, Luna offline, Prisma online, Upscaler con CLI; sesión visible 1 y luego 0 | Eventos JSONL de Codex; `/mcp/status` 09:15:18→09:15:30 |
| 11 | Agente real stdio | Codex 0.164 → binario `--mcp-stdio` | Binario nuevo | `initialize`, descubrimiento y `apps_list` | `apps_list` completada con los mismos estados | Eventos JSONL de Codex |
| 12 | Cliente HTTP sin sesión | Cliente → puerto temporal | Binario nuevo | `tools/list` sin `initialize` ni `Mcp-Session-Id` | 200 con 20 herramientas; sesiones 0→0: no aparece en el panel | Sonda directa |
| 13 | Aurora público | Local → `https://api.biglexj.com/api/v1/mcp` | Runtime publicado | `initialize`, `tools/list`, `list_posts(status=published, limit=1)`, `mcp.status` | 200; 8 herramientas públicas; una entrada publicada; `host_bridge=disconnected`; `invocation_audit=unavailable`; `synapse_bridge.configured=false` por `AURORA_SYNAPSE_MCP_URL` ausente | Respuestas JSON-RPC sin secretos |
| 14 | Agente real → Aurora público | Codex 0.164 → `https://api.biglexj.com/api/v1/mcp` | Runtime publicado | `mcp.status` y `aurora_site_context` | Leyó `host_bridge=disconnected`, `synapse_configured=false` y el contexto | Eventos JSONL de Codex |
| 15 | Aurora directo anónimo | Local → `http://127.0.0.1:5017/api/v1/mcp` | `mcp_platform_lab.exe` PID 20532 | `initialize`, `tools/list` | 200; 8 herramientas públicas; `mcp.status` inicial `configured=true, connected=false, tools=0` | Respuestas JSON-RPC |
| 16 | Aurora directo autenticado y refresco | Local → `127.0.0.1:5017` | Lab local con clave técnica del propio repo | `GET /api/v1/mcp/tools`, `GET /api/v1/mcp/synapse/status` | 59 herramientas (39 + 20); `synapse_bridge connected=true, tools=20, last_success_utc=2026-10-27T14:12:01Z` | Respuestas REST; no se imprimió la clave |
| 17 | Aurora → Synapse: invocación | Aurora local → `127.0.0.1:49296/mcp` vía puente | Lab local → binario instalado | `POST /api/v1/mcp/invoke` con `synapse.apps_list` | `status=executed`; devolvió estados reales de las apps; el contador de sesiones del puente pasó de 4 a 5. Esa sesión del lab queda retenida en el binario instalado porque no aplica TTL | Respuesta de invocación |
| 18 | Agente real → Aurora local | Codex 0.164 → servidor `aurora` existente (stdio → 5017) | Lab local | `mcp.status` y `aurora_site_context` | `synapse_configured=true`, `connected=true`, `tools=20`; `host_bridge=disconnected`; contexto de la plataforma leído | Eventos JSONL de Codex |
| 19 | Aurora Hub | Local | Backend local y público | Procesos y `mcp.status` | Sin proceso Hub; `host_bridge=disconnected`; `invocation_audit=unavailable` | Lista de procesos y `mcp.status` |
| 20 | **App instalada y caducidad** | Sonda → `127.0.0.1:49296/mcp` | Binario instalado 1.0.1 (26-10) | `initialize`, espera de 5 min, `ping` con la misma sesión | **HTTP 200: seguía viva**; el contador permaneció en 9 durante los 5 minutos (muestras cada 60 s); `DELETE` 204 y contador 8 | Resultado `ALIVE status=200`; no aplica la caducidad de 300 s |
| 21 | Estado del panel instalado | `GET /mcp/status` | Binario instalado | Lectura repetida | 8 sesiones retenidas al cierre; no son 8 agentes vivos demostrables | Contador y prueba 20 |
| 22 | Revisión de propiedad | Git y árbol | — | Comprobar archivos tocados | Solo sonda MCP, documentación MCP y documentos del proceso. Sin cambios **propios** en `peer.rs`, `devices.rs`, receptor LAN, Android, `src/features/gallery/` ni documentos globales. El árbol muestra cambios concurrentes del frente INTEROP y una compilación en curso; no se atribuyen a esta entrega | `git status`, mtimes y lista de procesos |

**Coordinación concurrente.** Durante esta pasada, `src-tauri/src/peer.rs` y `src-tauri/src/lib.rs` cambiaron fuera de este frente (mtimes 09:21:53 y 09:16:45) y había procesos `cargo`/`rustc` activos desde las 09:31. No se editaron ni se revirtieron esos archivos, y no se lanzaron compilaciones propias después de detectarlos, para no competir por los mismos artefactos. El binario probado es el del 09:13:45; los archivos MCP de transporte y sesión no cambiaron después de las 08:27, así que la evidencia de sesiones/transporte sigue referida al código MCP actual. Cualquier cambio del frente INTEROP posterior a esa marca requiere su propia compilación y prueba de integración.

La primera ejecución del modo lento **falló por el margen del guion** (renovaba a los ~300 s y la sesión B ya había caducado). Se corrigió el margen a 45 s y la segunda ejecución pasó. No fue un fallo del servidor. El agente Codex respondió `herramientas=17` en una prueba; fue un error aritmético del modelo: el catálogo real devuelve 20 y así lo registran la sonda y `tools/list`. No se usa ese texto como evidencia de conteo.

## Criterios del PLAN

- [x] Sesión inactiva vence; peticiones válidas renuevan; leer el panel no renueva; 404 permite reinicializar (pruebas 8 y 12).
- [x] HTTP y stdio publican el mismo catálogo y recursos; sin ejecución mutante durante las pruebas (pruebas 3 y 4).
- [x] Clave ausente/incorrecta y origen ajeno se rechazan; sesiones de clientes distintos no se confunden (pruebas 5, 6 y 7).
- [x] Conexión de agente real por HTTP y stdio, con lectura de capacidades y llamada de solo lectura (pruebas 10, 11, 14 y 18). Se distingue de un cliente JSON-RPC y no se presenta como registro nativo del chat.
- [x] Estado de Aurora público, MCP directo, Hub y Synapse documentado por separado (pruebas 13–19). Aurora → PC solo se valida por el puente local, no por la lectura pública.
- [x] Pruebas automatizadas y compilación proporcionales; comandos, resultado y cambios locales registrados.
- [x] Revisión de archivos compartidos; sin cambios en el contrato del frente INTEROP.

## Bloqueos y requisitos externos

| Bloqueo | Concreción | Qué falta | Estado |
| --- | --- | --- | --- |
| Aurora público → Synapse | El runtime publicado responde `AURORA_SYNAPSE_MCP_URL no está definido`; además no alcanza `127.0.0.1` de este PC | Configurar un runtime desplegado con un canal autenticado hacia el PC; no es un cambio de este repositorio ni se autorizó | Bloqueado; no se afirma conexión |
| Aurora Hub | Sin proceso Hub; `host_bridge=disconnected` | Iniciar/emparejar el Host y verificar el WebSocket del backend | Pendiente externo; no afecta al MCP local |
| Binario instalado sin caducidad | La app en ejecución es anterior a la compilación actual; retiene 8 sesiones | Reiniciar/actualizar con la compilación actual (hito 1.1.0); requiere autorización/ventana | Pendiente; no se reinició |
| Consumidor esporádico tras 300 s | El primer `tools/call` tras la inactividad recibe 404 | Reinicializar antes de actuar o reintentar solo lecturas; el refresco de 30 s evita el caso | Requisito de integración documentado |
| Conversación Ely Intelligence / registro nativo | No se probó una conversación completa ni el registro de Synapse como herramienta nativa de esta interfaz | Cliente/entorno de Ely Intelligence con su configuración real | Pendiente; fuera de esta evidencia |
| Android / teléfono | Pertenece a SYN-110-INTEROP | — | No tocado |

## Límites de la evidencia

- Las pruebas funcionales usan el binario recién compilado en un puerto loopback temporal, no la app instalada. La app instalada se usó solo para diagnosticar su estado real y demostrar que no aplica la caducidad.
- No se ejecutaron descargas, escalados, transferencias ni acciones mutantes. `apps_list` y `aurora_site_context` son lecturas.
- La lectura pública de Aurora no acredita control del PC ni conexión de Aurora Hub. No se configuró una URL remota al loopback local ni se abrió el MCP a la LAN.
- No se imprimieron claves, tokens ni rutas de credenciales. La prueba autenticada del lab usó la clave técnica ya presente en su entorno, sin mostrarla.
- El agente Codex es un cliente MCP real del entorno, no una conversación de Ely Intelligence ni una aprobación humana.

## Reproducción

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib
cargo build --manifest-path src-tauri/Cargo.toml --bin aurora-synapse
bun scripts/mcp/verify-foundation.mjs
bun scripts/mcp/verify-foundation.mjs --slow   # ~5 min; caducidad real
bun scripts/mcp/verify-mcp-panel.mjs
```

Aurora local (solo diagnóstico; no muta):

```powershell
# 8 herramientas públicas
$body = '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
Invoke-WebRequest -Uri 'http://127.0.0.1:5017/api/v1/mcp' -Method Post -ContentType 'application/json' -Body $body

# Estado del puente: requiere la frontera técnica local; no imprimir la clave
Invoke-WebRequest -Uri 'http://127.0.0.1:5017/api/v1/mcp/synapse/status' -Headers @{
  'x-ely-intelligence-key' = $env:ELY_INTELLIGENCE_SHARED_SECRET
  'X-Aurora-Principal'     = 'diagnostico-local'
}
```

---

# Segunda ejecución — 2026-10-28

Estado: ejecución completada en el alcance autorizado; bloqueos externos separados.
No se transfieren aprobaciones de pruebas históricas.

## Línea base de esta ejecución

| Dato | Valor |
| --- | --- |
| Rama y revisión | `master` @ `58a1afa` (sin commits nuevos) |
| Árbol | 135 entradas modificadas o sin seguimiento de partida; `src-tauri/src/mcp/`, `src/features/mcp/`, `scripts/mcp/`, `docs/mcp/` y `process/` sin seguimiento |
| Toolchain | Rust/cargo 1.99.1, Bun 1.4.2, Node 24.22.1, Vite 7.3.6 |
| Versión del proyecto | 1.0.1; sin cambio |
| Servicios presentes | Synapse MCP en `127.0.0.1:49296` (instancia del usuario, binario anterior), Prisma en `:49290`; **sin** backend de Aurora (`:5000`), sin Luna (`:49288`), sin Gallery-DL (`:18274`), sin proceso Aurora Hub |

No se crearon commits, no se publicaron releases, no se instalaron builds, no se
tocaron credenciales, despliegues, LAN, emparejamiento ni Android.

## Cambios de código de esta ejecución

| Archivo | Cambio |
| --- | --- |
| `src-tauri/src/mcp/protocol.rs` | `SessionStore` separa consulta (`get`, no renueva) de renovación (`renew`/`touch`); `SessionMeta` añade `last_activity_utc` e `idle_seconds()`; `backdate` solo para pruebas del crate |
| `src-tauri/src/mcp/server.rs` | El 404 de sesión lleva `error`, `detail` e `idle_timeout_seconds`; `initialize` reabre sesión con cabecera caducada y retira la anterior; la renovación ocurre solo tras validar la petición; el cuerpo se analiza una sola vez (`handle_request`) |
| `src-tauri/src/mcp/status.rs` | `McpAgentInfo` expone `last_activity_at` e `idle_seconds` |
| `src-tauri/src/mcp/capabilities.rs` | El recurso publica el bloque `session` con el contrato completo |
| `src/features/mcp/types.ts`, `mcpService.ts`, `McpStatusPanel.tsx` | El panel normaliza y muestra la actividad de cada cliente y aclara qué significa «Conectado» |
| `scripts/mcp/verify-foundation.mjs` | Acepta `AURORA_SYNAPSE_BIN`; añade clave incorrecta, origen ajeno, aislamiento entre clientes, recuperación tras 404 y comprobación del contrato de sesión |
| `scripts/mcp/verify-agent-client.mjs` (nuevo) | Conecta un cliente de agente real y exige que use las herramientas nativas |

## Evidencia por comprobación

| # | Prueba | Origen → destino | Runtime | Acción | Resultado observado | Evidencia |
| --- | --- | --- | --- | --- | --- | --- |
| 23 | Pruebas Rust | Local | `cargo test --lib` | Suite del crate | **62 aprobadas**, 0 fallidas: **48 del módulo MCP** (39 de partida + 8 nuevas de esta ejecución) y 14 del frente paralelo en `peer.rs` | Salida de `cargo test` |
| 24 | Compilación | Local | `cargo build --bin aurora-synapse` | Build debug | Correcta; el aviso `parse_invitation` sin uso es preexistente de `peer.rs` (frente INTEROP) | Salida de cargo |
| 25 | Typecheck y build web | Local | `tsc --noEmit` y `vite build` | Compilación del panel | `tsc` exit 0; `vite build` correcto (41 módulos) | Salida de ambos |
| 26 | Sonda HTTP/stdio ampliada | Cliente JSON-RPC (Bun) → `127.0.0.1:<puerto efímero>/mcp` | Binario nuevo, clave efímera | Clave incorrecta; `Origin` ajeno; dos `initialize`; `initialize` con cabecera caducada; `resources/read` | 401; 403; sesiones distintas; 404 → `initialize` 200 con ID nuevo y `tools/list` 200; el recurso trae `session.idle_timeout_seconds = 300` | `verify-foundation.mjs`, salida `ok` |
| 27 | Contrato de sesión en el recurso | → mismo puente | Binario nuevo | `resources/read` de `aurora-synapse://capabilities` | Bloque `session` con `header`, `idle_timeout_seconds`, `renewed_by`, `not_renewed_by`, `expires_with`, `recovery`, `semantics`, `authorization` | `verify-foundation.mjs` |
| 28 | Panel | Componentes React/TS en Bun | Contrato normalizado | Normalización, agrupación, tri-estado, agente y degradación | **28 correctas** (23 previas + 5 nuevas) | `verify-mcp-panel.mjs` |
| 29 | **Caducidad real con el código nuevo** | Cliente → puerto efímero | Binario nuevo | Espera de 315 s con panel sondeado | Sesión no renovada caduca; el panel no renueva; una petición aceptada renueva; `initialize` recupera | `verify-foundation.mjs --slow` |
| 30 | **Agente real, registro nativo** | Claude Code → binario `--mcp-stdio` | Cliente MCP nativo | `ListMcpResourcesTool`; `ReadMcpResourceTool`; `mcp__aurora-synapse__classify_intent` | Recurso listado y leído de forma nativa; `tool_count` 20 y `session.idle_timeout_seconds` 300 leídos del propio recurso; `classify_intent` → `detected_type: stream_video`, `recommended_app_id: luna` | `verify-agent-client.mjs`, salida `ok`; [evidencia](../../../docs/mcp/evidencia/agente-real-claude-code-2026-10-28.md) |
| 31 | MCP directo de Aurora, backend real | Local → `127.0.0.1:5000/api/v1/mcp` | `aurora-backend.exe` con `APP_HOST=127.0.0.1` | `initialize`; `tools/list`; `GET /api/v1/mcp/tools`; `mcp.status` por control plane | `200`, `serverInfo aurora-mcp 1.0.0`; 17 herramientas remotas; 59 en el control plane con 20 `synapse.*`; `host_bridge: disconnected`, `invocation_audit: unavailable` | [evidencia](../../../docs/mcp/evidencia/aurora-backend-real-2026-10-28.md) |
| 32 | **Aurora → Synapse, lectura real** | Backend de Aurora → `127.0.0.1:49296/mcp` | Mismo proceso | `GET /api/v1/mcp/synapse/status`; `mcp.status` → `synapse_bridge` | `configured: true`, `connected: true`, `tools: 20`, `last_error: null`; log «Catálogo del puente Aurora Synapse actualizado — count=20» | [evidencia](../../../docs/mcp/evidencia/aurora-backend-real-2026-10-28.md) |
| 33 | Aurora Hub | Local → backend | Procesos y control plane | Lista de procesos; `GET /api/v1/mcp/hosts/status` | Sin proceso Hub; `{"connected":false,"ready":false}`; `host_bridge: disconnected` | [evidencia](../../../docs/mcp/evidencia/aurora-backend-real-2026-10-28.md) |
| 34 | Conector público | Local → `https://api.biglexj.com/api/v1/mcp` | Runtime publicado | `initialize`; `tools/list`; `aurora_site_context`; `mcp_status` | 200 en las tres lecturas; `mcp_status` por llamada directa sin credencial → `{"error":"invalid_token"}`; la sonda `verify-aurora-services.mjs` sí lo lee: `host_bridge=disconnected`, `invocation_audit=unavailable`, `synapse_bridge.configured=false`, 8 herramientas públicas, `catalog_tools: 39` | Respuestas JSON-RPC, sin secretos |
| 35 | Instancia de Synapse en ejecución | Sonda → `127.0.0.1:49296/mcp` | Binario instalado (anterior) | `POST` con sesión inexistente | 404 con `{"error":"session_not_found"}` **sin** `detail`: es el binario previo a estos cambios | Respuesta HTTP |
| 36 | Revisión de propiedad | Git y árbol | — | Comprobar archivos tocados | Solo `src-tauri/src/mcp/`, `src/features/mcp/`, `scripts/mcp/`, `docs/mcp/` y este proceso. `peer.rs`, `devices.rs` y `src/features/gallery/` siguen siendo del frente INTEROP y `lib.rs` aparece modificado por ese frente: **no se tocaron, no se revirtieron y no se les atribuyen** | `git status` |

## Criterios del PLAN en esta ejecución

- [x] Sesión inactiva vence; peticiones válidas renuevan; leer el panel no renueva; 404 permite reinicializar (pruebas 26, 27, 29 y las de Rust).
- [x] HTTP y stdio publican el mismo catálogo y recursos; sin ejecución mutante durante las pruebas (prueba 26).
- [x] Clave ausente/incorrecta y origen ajeno se rechazan; sesiones de clientes distintos no se confunden (prueba 26).
- [x] Conexión de agente real con lectura de capacidades y llamada de solo lectura, **registrada de forma nativa en el chat** y no como sonda JSON-RPC (prueba 30).
- [x] Estado de Aurora público, MCP directo, Hub y Synapse documentado por separado (pruebas 31–34). Aurora → Synapse se validó con una lectura real del puente en este PC (prueba 32), no con la lectura pública.
- [x] Pruebas automatizadas y compilación proporcionales; comandos, resultado y cambios locales registrados (pruebas 23–30).
- [x] Revisión de archivos compartidos; sin cambios en el contrato del frente INTEROP (prueba 36).

## Bloqueos de esta ejecución

| Bloqueo | Concreción | Qué falta | Estado |
| --- | --- | --- | --- |
| Estado interno del runtime público de Aurora | Se puede leer con la sonda (`synapse_bridge.configured=false`, `host_bridge=disconnected`), pero una llamada directa sin credencial devuelve `invalid_token` | Credencial del despliegue público, si el operador decide proporcionarla | Lectura pública hecha; no acredita control del PC |
| Aurora → Synapse **público** | El runtime publicado no tiene `AURORA_SYNAPSE_MCP_URL` y, aunque la tuviera, no alcanza `127.0.0.1` de este PC | Despliegue con canal autenticado hacia el PC | Bloqueado; el enlace **local** sí está verificado (prueba 32) |
| Aurora Hub | Sin proceso Hub; `host_bridge: disconnected` | Iniciar/emparejar el Host | Pendiente externo; no afecta al MCP local |
| Binario en ejecución desfasado | El puente abierto en `:49296` responde sin `detail`: binario anterior | Reiniciar la aplicación con la compilación actual | Pendiente; no se reinició la interfaz del usuario |
| Canal remoto autenticado | Dependencia externa de despliegue | Definición del canal y su autenticación | Pendiente; no se abre el MCP a la LAN |

## Límites de la evidencia de esta ejecución

- Las pruebas del puente usan el binario recién compilado en un puerto loopback
  temporal. La instancia abierta por el usuario se usó solo para diagnosticar
  que arrastra el binario anterior.
- El backend de Aurora se levantó **solo** para el diagnóstico, con
  `APP_HOST=127.0.0.1`, y se detuvo al terminar: su MCP nunca quedó expuesto a
  la LAN.
- No se ejecutaron descargas, escalados, transferencias ni acciones mutantes.
  `apps_list`, `classify_intent` y `aurora_site_context` son lecturas.
- No se imprimieron claves, tokens ni rutas de credenciales.
- La prueba de agente real es un cliente MCP nativo en esta interfaz de chat,
  no una conversación de Ely Intelligence ni una aprobación humana.

## Reproducción de esta ejecución

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib
cargo build --manifest-path src-tauri/Cargo.toml --bin aurora-synapse
bun scripts/mcp/verify-foundation.mjs
bun scripts/mcp/verify-foundation.mjs --slow   # ~5 min; caducidad real
bun scripts/mcp/verify-mcp-panel.mjs
bun scripts/mcp/verify-agent-client.mjs         # cliente de agente real
```

Diagnóstico de Aurora con el backend local (solo loopback, sin secretos
impresos):

```powershell
cd ..\Aurora---Blog\backend
$env:APP_HOST='127.0.0.1'; $env:PORT='5000'
.\target\debug\aurora-backend.exe
```
