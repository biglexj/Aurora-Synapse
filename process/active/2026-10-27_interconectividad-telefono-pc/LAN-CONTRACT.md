# SYN-110-INTEROP — contrato de red del canal emparejado

Fecha: 2026-10-27. Versión del contrato: `synapse-lan-v1`. Implementación:
`src-tauri/src/peer.rs` (receptor y decisiones) y el cierre de despacho local en
`src-tauri/src/lib.rs`. Este documento no modifica el transporte MCP: el frente
**SYN-110-MCP** conserva `protocol.rs`, `server.rs`, `status.rs`,
`capabilities.rs`, `mod.rs` y el panel.

## 1. Identidad del PC

El QR es la única fuente de identidad del PC para el teléfono:

`aurora-synapse://pair?v=1&address=http%3A%2F%2F<ip-local>%3A49295&ticket=<64 hex>`

- 256 bits aleatorios por invitación, un solo uso, caducidad de 90 segundos.
- `address` debe ser una IP privada/link-local y el puerto debe ser `49295`.
- El QR nunca transporta credenciales MCP ni la credencial de Super Galería.
- La credencial del teléfono dura 12 horas o hasta que el proceso del PC
  termina; el PC conserva solo su SHA-256 en memoria. El teléfono la guarda en
  el almacén seguro de Android (Keystore), no en JSON.

## 2. Rutas del receptor LAN (0.0.0.0:49295)

| Método y ruta | Credencial | Efecto | Respuesta |
| --- | --- | --- | --- |
| `OPTIONS *` | No | Ninguno (sondeo CORS) | `204` |
| `GET /status` | No | Ninguno; estado informativo (`status`, `app`, `version`, `protocol`, `paired_protocol`) | `200` |
| `POST /pair/claim` | Ticket del QR | Canjea la invitación por una credencial de 12 h | `200 {success, token, pc_name, expires_in}`; `400` con motivo si el ticket caducó o no corresponde |
| `GET /paired/status` | Bearer | Ninguno; consulta de solo lectura | `200 {success, pc_name, linked, expires_in, protocol}` |
| `POST /paired/dispatch` | Bearer | Ejecuta una orden de apertura/despacho en el PC | `200` con `status: "ok"` o `status: "error"` y `message` real de la app |
| `POST /synapse`, `POST /dispatch` | Bearer | Lo mismo que la ruta canónica; **ruta heredada** | Igual, con `deprecated: true` |

Reglas que aplican a todas:

1. Solo se atienden pares de la red local (privadas, link-local o loopback).
2. Las rutas del teléfono (`/pair/claim`, `/paired/*`) rechazan cualquier
   petición con cabecera `Origin`: un navegador no puede suplantar al teléfono.
3. Un beacon de descubrimiento nunca acredita autorización. Solo el Bearer con
   una credencial vigente permite ejecutar una orden.
4. Cada petición autorizada produce **exactamente una** llamada al despachador
   local. No hay reintentos ni fallback a endpoints de la app ni a otras IP.
5. Un error de forma (`app_id` ausente/ inválido, contenido mayor de 8 KiB) es
   `400` y no ejecuta nada. Un rechazo de la app es `200` con
   `status: "error"` y el mensaje real, para que el teléfono lo muestre.

## 3. Resultado de una orden

```json
{
  "success": true,
  "status": "ok",
  "message": "Orden entregada a Prisma (prisma://open?path=...)",
  "app_id": "prisma",
  "delivery": "accepted",
  "verified": false,
  "canonical_route": "/paired/dispatch",
  "deprecated": false
}
```

- `delivery: "accepted"` significa que el PC recibió la orden y la entregó al
  sistema operativo o al contrato local de la app. **No** demuestra que la
  ventana esté visible.
- `verified: false` es deliberado: este canal no confirma ejecución visual.
- `delivery: "rejected"` acompaña a todo `status: "error"`. Los estados
  `sent`, `accepted` y `completed` conservan significados distintos; la
  transferencia de Super Galería usa `delivery: "completed"` porque comprueba
  tamaño y SHA-256 en el receptor.

## 4. Transición de las rutas LAN heredadas

Antes, `POST /synapse` y `POST /dispatch` ejecutaban una app en el PC sin
credencial y con un contrato (`{"action":"download","url":...}`) que las apps
ya no implementan. Ahora:

- Las tres rutas de despacho exigen Bearer a través de `peer::decide`.
- Las heredadas siguen funcionando para un cliente que ya tenga credencial, y
  reciben `deprecated: true` y `canonical_route` para migrar.
- Sin credencial devuelven `401` con `canonical_route`, `legacy_route` y la
  indicación de escanear de nuevo el QR. No ejecutan nada.
- El teléfono ya no intenta endpoints alternativos: usa solo el PC vinculado y
  propaga el rechazo.

## 5. Descubrimiento (49289/UDP)

- Beacons JSON con `device_id`, `device_name`, `device_type`, `port`, `os`,
  `target_app`, `capabilities` y la dirección del emisor.
- Synapse escucha sin reclamar el puerto en exclusiva (`SO_REUSEADDR`), igual
  que Prisma y Super Galería; caducan a los 60 segundos.
- Apps conocidas por beacon en esta entrega: `prisma` (PC, 49290),
  `supergallery` (Android, 49288) y `luna` (49288). Un beacon de Luna en el
  teléfono y Super Galería en el mismo teléfono comparten puerto: no pueden
  escuchar simultáneamente en el mismo dispositivo.
- Limitación registrada (repositorio Prisma, fuera de este frente): Prisma anuncia
  el puerto 49290 en el beacon, pero su servidor se vincula al primero libre de
  49290, 49292 y 49293. Si 49290 está ocupado, la dirección anunciada no es la
  real.

## 6. Qué consume el frente MCP (sin tocar su transporte)

- Estado de vinculación del teléfono: `peer::status()` y `peer::linked_remaining()`
  (Rust, solo lectura).
- Lectura del teléfono (Super Galería): `mcp::gallery::status()`, ya expuesto
  como `gallery_status` y `gallery_devices`. Verificado en vivo contra el
  teléfono físico.
- Despacho teléfono → PC: `POST /paired/dispatch` con Bearer; el frente MCP no
  debe abrir estas rutas a la LAN ni crear un segundo servidor.
- El canal no transporta credenciales MCP; el MCP local sigue en loopback.

## 7. Límites reales de Android registrados en esta entrega

- El teléfono no ejecuta un receptor de órdenes del PC: `start_lan_http_server`
  está compilado solo para escritorio. **PC → teléfono no puede abrir apps
  remotamente** y no se inventa un contrato para ello.
- Lo soportado PC → teléfono es: consulta de estado de Super Galería
  (`GET /api/v1/synapse/status`), catálogo autorizado (`/api/v1/synapse/agents/*`
  con credencial propia de la galería) y transferencia de archivos. La apertura
  es manual en el teléfono.
- Recibir una orden (`accepted`) no equivale a ejecución comprobada; por eso el
  resultado no se anuncia como `completed`.
