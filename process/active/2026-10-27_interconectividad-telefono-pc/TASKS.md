# SYN-110-INTEROP — tareas

Estado: ejecutado parcialmente el 2026-10-27. Rama `master` @ `58a1afaf`;
los cambios de este frente quedan sin commit. Cada casilla remite a
[VALIDATION](VALIDATION.md) y al [contrato LAN](LAN-CONTRACT.md).

- [x] Registrar rama, revisión y cambios locales; comprobar qué binario y dispositivos se usan.
  - `master` @ `58a1afaf5de21edec30ae99d61fef8c8218f4edc`, con trabajo previo sin
    commit (procesos, `peer.rs` y el adaptador de galería ya existían sin
    seguimiento).
  - PC: app instalada 1.0.1 en `C:\Users\biglexj\AppData\Local\Programs\Aurora
    Synapse\aurora-synapse.exe` (PID 2976; `0.0.0.0:49295`, MCP
    `127.0.0.1:49296`). El frente MCP mantenía instancias de `target\debug` en
    ejecución: no se detuvieron y la compilación usó
    `src-tauri\target\interop`.
  - Teléfono: Motorola edge 60 fusion (`192.168.1.103`) con
    `com.biglexj.aurorasynapse`, `com.biglexj.lienzo` (Super Galería),
    `com.biglexj.lunafetch` y `com.biglexj.elytesia` instalados.

- [x] Inventariar capacidades reales por dirección, app, transporte y dispositivo; acordar una apertura y una consulta de estado verificable por dirección como piloto, sin inventar soporte.
  - Matriz en VALIDATION §1. Piloto acordado:
    - teléfono → PC: apertura de una app del PC por `POST /paired/dispatch`
      (Bearer) y consulta `GET /paired/status`.
    - PC → teléfono: consulta `GET /api/v1/synapse/status` y `gallery_status`
      (credencial propia de la galería) más la transferencia existente. La
      apertura remota no existe y se registra como no soportada.

- [x] Reutilizar descubrimiento y QR, con identidad y destino explícitos; no deducir autorización de un beacon.
  - Descubrimiento 49289 y `devices.rs` reutilizados sin cambios; QR
    `aurora-synapse://pair?v=1` intacto; la autorización solo la concede la
    credencial validada por `peer::authorize`. La transferencia PC → teléfono
    exige que el dispositivo elegido sea `supergallery` y coincida con la URL
    conectada; un dispositivo distinto se rechaza sin enviar nada.

- [x] Diseñar y ejecutar transición de rutas LAN heredadas sin autenticación, preservando los envíos autorizados mediante compatibilidad explícita y pruebas de regresión.
  - `start_lan_http_server` ya no implementa su propia autorización: usa
    `peer::serve`/`peer::decide`. `/synapse` y `/dispatch` exigen Bearer,
    responden `deprecated: true` y `canonical_route`, y conservan los alias
    `app`/`url` de los clientes existentes. Cinco pruebas HTTP nuevas.

- [x] Implementar los recorridos piloto teléfono → PC y PC → teléfono, propagando rechazo y resultado de la app; si Android impide una apertura, registrar el resultado real y el comportamiento soportado.
  - Teléfono → PC: `dispatch_to_pc` usa solo el canal emparejado. Se eliminaron
    los fallbacks a `http://ip:puerto/synapse` y `http://ip:49295/dispatch`, que
    ejecutaban sin credencial y podían duplicar la orden. El resultado incluye
    `status`, `message` real, `app_id`, `delivery: accepted` y `verified:
    false`.
  - PC → teléfono: `dispatch_content` rechaza la apertura remota con motivo y
    conserva la transferencia a Super Galería solo hacia el destino anunciado.
    Android no admite apertura remota y el Synapse Android no compila receptor
    LAN.

- [x] Comprobar ausencia de app/dispositivo, credencial vencida/revocada, reconexión y reintentos sin duplicación.
  - Pruebas: credencial vencida → 401 sin ejecutar; `app_id` ausente/inválido →
    400 sin ejecutar; app que rechaza → 200 con `delivery: rejected` y una sola
    llamada; rutas heredadas sin credencial → 401 sin ejecutar; origen web →
    403 sin ejecutar.
  - En dispositivo: app en segundo plano → 200; proceso ausente → conexión
    rechazada; reapertura → servidor recuperado.
  - Pendiente físico: reconexión del teléfono tras 401 (requiere QR nuevo y
    escaneo del usuario).

- [ ] Probar físicamente las dos direcciones y preservar las transferencias de Super Galería, exclusiones e integridad.
  - Hecho: PC → teléfono (estado 200, `gallery_status` con `paired: true` y
    transferencia real de 57 B con SHA-256 coincidente); exclusiones e
    integridad sin cambios porque `gallery.rs` no se tocó.
  - Pendiente: teléfono → PC con el receptor nuevo. Requiere que el puerto 49295
    esté libre (lo ocupa la app instalada y hay instancias del otro frente) y
    que el teléfono escanee un QR nuevo. No se detuvo la app ni se instaló un
    build.

- [x] Registrar contrato de red consumible por el frente MCP sin tocar su transporte; documentar dependencias y resultados.
  - [LAN-CONTRACT.md](LAN-CONTRACT.md). No se editaron `protocol.rs`,
    `server.rs`, `status.rs`, `capabilities.rs`, `mod.rs`, el panel MCP ni los
    documentos globales.

- [x] Actualizar VALIDATION y APPROVAL de este frente y entregar resumen de integración al proceso global.
  - Hecho en esta entrega (VALIDATION §6). El proceso global y el hito 1.1.0 no
    se cierran aquí.

## Dependencias y pendientes

1. Físico teléfono → PC: liberar 49295 y escanear el QR nuevo con el usuario.
2. Prisma anuncia 49290 aunque puede escuchar en 49292/49293: cambio en su
   repositorio, fuera de este alcance.
3. `docs/mcp/PHONE-QR.md` y `docs/mcp/FOUNDATION.md` aún describen las rutas
   heredadas sin autenticación; se propone que SYN-110-MCP actualice esa nota
   al integrar (son documentación del otro frente).
4. `src/App.tsx` conserva las etiquetas «Abrir en Móvil» para apps Android sin
   apertura remota; el backend ya devuelve el motivo real. Ajuste de etiqueta
   pendiente, sin cambio de capacidad.
