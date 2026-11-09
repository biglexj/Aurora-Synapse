# SYN-110-INTEROP — validación

Fecha: 2026-10-27. Rama `master`, revisión `58a1afaf5de21edec30ae99d61fef8c8218f4edc`,
árbol con trabajo previo sin commit. Este frente añadió código en
`src-tauri/src/peer.rs` (receptor y contrato) y en
`src-tauri/src/lib.rs` (cierre del despacho local y resolución de destino);
ver [lan-contract](LAN-CONTRACT.md). No se editaron archivos del frente
**SYN-110-MCP** (`protocol.rs`, `server.rs`, `status.rs`, `capabilities.rs`,
`mod.rs`, panel MCP) ni documentos globales.

## 1. Inventario contrastado de las conexiones existentes

Comprobado en el equipo real el 2026-10-27 entre 09:10 y 09:25 (America/Bogota).
PC `192.168.1.121`; teléfono Motorola edge 60 fusion `192.168.1.103`.

| App | Superficie real | Autorización | Dirección | Estado observado | Piloto soportado |
| --- | --- | --- | --- | --- | --- |
| Super Galería (Android) | HTTP `0.0.0.0:49288`; beacon UDP 49289 (`target_app: supergallery`); `/api/v1/synapse/status` sin credencial; `/api/v1/synapse/agents/{pair,status,media,upload}` con credencial de código temporal | `POST agents/pair` con código de 6 dígitos → token de agente; el estado público no exige token | PC → teléfono | `GET /status` → HTTP 200; `gallery_status` → `configured: true, paired: true, reachable: true`; dispositivo `Motorola motorola edge 60 fusion`; beacon `supergallery_mobile_..._scout` | Consulta de estado y transferencia (apertura remota no existe) |
| Prisma (PC) | HTTP: primero libre de `49290/49292/49293`; beacon anuncia `49290` (`target_app: prisma`); `/api/v1/synapse/{status,playback,artwork,lyrics,handoff,remote,upload}`; deep link `prisma://` | Sin credencial: solo red local | teléfono → PC (y app ↔ app) | `prisma_status` → `reachable: true`, dispositivo `BIGLEXJ`; servidor en `0.0.0.0:49290` | Apertura en el PC por orden del teléfono; consulta de estado vía contrato de Prisma |
| Luna Fetch (PC) | Servidor LAN `0.0.0.0:49288` (`/api/v1/synapse/{ping,push-download,sync-history,status,announce-release}`); despachador de una línea en `127.0.0.1:49282` (`luna://`); beacon `target_app: luna` | Sin credencial: solo red local; el despachador es solo loopback | teléfono → PC | `luna_status` → offline (`127.0.0.1:49288` sin respuesta); la app de escritorio no estaba en ejecución | Apertura/despacho en el PC una vez instalada/abierta; no se amplió nada |
| Aurora Synapse LAN (PC) | HTTP `0.0.0.0:49295` (canal emparejado); recibe beacons en 49289; MCP solo loopback `127.0.0.1:49296` | Bearer de 12 h para `/paired/*`; ticket QR de un solo uso para `/pair/claim` | ambas | Receptor nuevo probado sobre TCP real en pruebas Rust; la app instalada (1.0.1, PID 2976) sigue con el receptor anterior | Apertura en el PC y consulta `/paired/status` |

Límites registrados del inventario:

- Prisma anuncia el puerto 49290 aunque su servidor puede vincularse a 49292 o
  49293 si el primero está ocupado: la dirección del beacon no siempre coincide
  con la real. Depende del repositorio Prisma; no se modificó.
- Luna Fetch y Super Galería usan ambos el puerto 49288: en un mismo dispositivo
  no pueden atender simultáneamente.
- Synapse no emite beacon propio; el teléfono lo alcanza por el QR, no por
  descubrimiento.

## 2. Recorrido piloto teléfono → PC

Origen: teléfono emparejado. Destino: app del PC. Runtime: receptor nuevo
ejercitado sobre TCP real en las pruebas y build Android instalada, sin cambios
en esta entrega.

| Aspecto | Resultado |
| --- | --- |
| Apertura | `POST /paired/dispatch` con `app_id` y `content`; el PC resuelve la app en su catálogo y entrega la URI/orden al sistema. Devuelve `message` real, `app_id`, `delivery: accepted` y `verified: false`. Un `app_id` desconocido o una app `PROXIMAMENTE`/`INTERNO` devuelve `status: error` con el motivo. |
| Consulta de estado | `GET /paired/status` con Bearer → `{success, pc_name, linked, expires_in, protocol}`. |
| Sin autorización | `POST /paired/dispatch`, `POST /synapse` y `POST /dispatch` sin Bearer → `401` con `canonical_route`, `legacy_route` y la indicación de reescanear el QR; **no se ejecuta ninguna orden**. |
| Rechazo propagado | El fallo de la app no se convierte en fallo de red: `200` con `status: "error"`, `delivery: "rejected"` y el mensaje exacto para el teléfono. |
| Reintentos | Una petición autorizada produce exactamente una llamada al despachador. Se eliminaron los fallbacks del teléfono a `http://ip:puerto/synapse` y `http://ip:49295/dispatch`, que podían duplicar la orden y ejecutaban sin credencial. |

Evidencia automatizada (TCP real, mismo código que usa la app):
`el_receptor_http_exige_credencial_y_ejecuta_una_sola_vez`,
`el_receptor_http_completa_el_intercambio_y_el_estado`,
`el_receptor_http_rechaza_origenes_web_y_ordenes_invalidas`,
`un_rechazo_de_la_app_se_propaga_sin_segundo_intento`,
`una_credencial_vencida_no_ejecuta`.

**Pendiente físico:** el emparejamiento real teléfono → PC con el receptor nuevo
no se ejecutó. Bloqueo: el puerto 49295 lo ocupa la app instalada (1.0.1) y el
frente MCP mantiene instancias de `target\debug` en ejecución; además, el
teléfono tendría que escanear un QR nuevo, paso que requiere al usuario. La app
instalada no se detuvo y no se instaló ningún APK.

## 3. Recorrido piloto PC → teléfono

Origen: PC. Destino: Super Galería del teléfono físico. Acción: consulta de solo
lectura y transferencia existente. La transferencia no se amplió.

| Prueba | Origen → destino | Acción | Resultado observado | Evidencia sin secretos |
| --- | --- | --- | --- | --- |
| Estado de la app | PC → teléfono `192.168.1.103:49288` | `GET /api/v1/synapse/status` | HTTP 200; `target_app: supergallery`, `paused: false`, `device_name: Motorola motorola edge 60 fusion` | respuesta JSON, sin credenciales |
| Estado a través de Synapse | PC (MCP loopback de la app instalada) → teléfono | `gallery_status` | `configured: true, paired: true, reachable: true`; credencial de galería válida; dispositivo y URL reales | `Mcp-Session-Id` + resultado JSON |
| Transferencia existente | PC → teléfono | `gallery_send` de un archivo temporal de 57 B | `delivery: completed`; el teléfono devolvió `size_bytes: 57` y SHA-256 `7f1c7aaf…ccf010b`, idéntico al local; archivo presente en `/storage/emulated/0/Download/SuperGallery/` | salida JSON; archivo retirado del teléfono después |
| Segundo plano | PC → teléfono | App en segundo plano (HOME) y consulta | HTTP 200: el receptor de la galería funciona en segundo plano | salida de la consulta |
| Ausencia de proceso | PC → teléfono | `am force-stop com.biglexj.lienzo` y consulta | Conexión rechazada; el servidor no responde sin proceso | mensaje de error de conexión |
| Recuperación | PC → teléfono | `am start -n com.biglexj.lienzo/com.biglexj.supergallery.MainActivity` | HTTP 200 de nuevo, mismo `target_app` y dispositivo | respuesta JSON |

La apertura remota de una app en el teléfono **no está soportada**: Android no
ofrece un contrato para ello y el Synapse Android compilado no tiene receptor
LAN (`start_lan_http_server` es solo de escritorio). El PC ahora responde con un
motivo explícito en lugar de prometer una apertura. No se inventó una ruta.

## 4. Transición de rutas LAN y pruebas automatizadas

Comandos ejecutados (el árbol compartido tiene instancias del otro frente en
ejecución; se usó un directorio de compilación separado, ignorado por Git):

```powershell
# Línea base, antes de los cambios
cargo test --manifest-path src-tauri/Cargo.toml --lib          # 57 passed

# Con los cambios (57 pruebas base; 5 nuevas del receptor)
$env:CARGO_TARGET_DIR='D:\Proyectos\biglexj\Aurora-Synapse\src-tauri\target\interop'
cargo test --manifest-path src-tauri/Cargo.toml --lib          # 62 passed; 0 failed
cargo build --manifest-path src-tauri/Cargo.toml --bin aurora-synapse   # Finished, exit 0
```

`cargo test` cubre el receptor HTTP real (`peer::serve`) sobre un puerto efímero:
intercambio de invitación de un solo uso, estado del PC para el teléfono, rechazo
de orígenes web, credencial vencida, `app_id` inválido, app que rechaza la orden,
rutas heredadas con y sin credencial, alias `app`/`url` para clientes
existentes y una única ejecución por petición. También se conservan las pruebas
del QR y del lector de cuerpos fragmentados.

Las pruebas 5 nuevas son:

1. `el_receptor_http_exige_credencial_y_ejecuta_una_sola_vez`
2. `el_receptor_http_completa_el_intercambio_y_el_estado`
3. `el_receptor_http_rechaza_origenes_web_y_ordenes_invalidas`
4. `un_rechazo_de_la_app_se_propaga_sin_segundo_intento`
5. `una_credencial_vencida_no_ejecuta`

Compilación de la rama Android (solo `check`, sin generar ni instalar APK),
porque el cambio toca rutas `cfg(target_os = "android")`:

```powershell
$env:ANDROID_HOME='C:\Users\biglexj\AppData\Local\Android\Sdk'
$env:NDK_HOME='C:\Users\biglexj\AppData\Local\Android\Sdk\ndk\29.0.13846066'
$env:CARGO_TARGET_DIR='D:\Proyectos\biglexj\Aurora-Synapse\src-tauri\target\interop'
cargo check --manifest-path src-tauri/Cargo.toml --lib --target aarch64-linux-android
# Finished `dev` profile; sin errores.
```

Quedan 2 avisos previos en `lib.rs` (`is_dev_mode` sin usar y
`dispatch_locally_desktop` sin uso en Android), ajenos a este cambio. El módulo
`peer` no emite avisos en Android: el receptor es de escritorio y las funciones
del contrato no se usan en runtime allí (documentado con `allow(dead_code)`).

## 5. Regresión de Super Galería

- `src-tauri/src/mcp/gallery.rs` y `gallery_credentials.rs` no se modificaron:
  exclusión de contenido, `create_new` (sin sobrescritura), tamaño y SHA-256
  conservan su comportamiento.
- La transferencia real verificada devolvió `completed` con tamaño y SHA-256
  coincidentes; el archivo se retiró del teléfono después de la prueba.
- El catálogo autorizado (`agents/media`) no se consultó con contenido real para
  no leer contenido personal del teléfono; el estado autenticado sí se comprobó
  a través de `gallery_status`.
- Un envío PC → teléfono ahora exige que el dispositivo elegido sea
  `supergallery` y que coincida con la URL conectada; un destino distinto se
  rechaza sin enviar nada.

## 6. Resumen para integración en SYN-110

- El receptor LAN dejó de ejecutar despachos sin autenticación; las rutas
  heredadas siguen disponibles con Bearer y aviso de migración.
- El teléfono ya no intenta endpoints alternativos: una sola orden por petición.
- El resultado de una orden distingue `accepted`/`rejected` y `verified: false`.
- El contrato de red está en [LAN-CONTRACT.md](LAN-CONTRACT.md) para que
  SYN-110-MCP lo consuma sin tocar su transporte.
- La consulta PC → teléfono y la transferencia existente se comprobaron contra
  el dispositivo físico.

Pendientes y bloqueos externos:

1. **Teléfono → PC físico**: requiere liberar el puerto 49295 (app instalada y
   frente MCP en ejecución) y un escaneo de QR por el usuario. No es un fallo
   del cambio; no se presenta como aprobado.
2. **Diferencia de puerto de Prisma** entre beacon y servidor real: cambio en el
   repositorio Prisma, fuera de este frente.
3. **Documentación compartida**: `docs/mcp/PHONE-QR.md` y `docs/mcp/FOUNDATION.md`
   conservan la nota de «rutas LAN heredadas sin autenticación». Pertenecen a la
   documentación MCP; se propone al frente SYN-110-MCP actualizarla al integrar.
4. **UI compartida**: `src/App.tsx` no se tocó; las tarjetas aún ofrecen «Abrir
   en Móvil» para apps Android sin apertura remota. El backend ahora responde
   con el motivo real. Es un ajuste de etiqueta pendiente, no una capacidad.

## 7. Alcance de la evidencia

No se instaló ningún build en el teléfono, no se publicó ninguna release, no se
leyeron ni imprimieron secretos, no se detuvo la app instalada y no se modificó
el proceso global. Las pruebas automáticas acreditan el receptor y la
transición; no acreditan por sí solas la aceptación del usuario ni la
interconectividad física completa.
