# SYN-110-INTEROP — autorización y cierre

Fecha: 2026-10-27. Estado: **ejecutado parcialmente; no APPROVED.**

## Autorización

Biglex encarga ejecutar SYN-110-INTEROP según su PLAN. Esa instrucción autoriza
leer el proceso, contrastar la implementación, cambiar los archivos asignados a
este frente, probar en local y compilar de diagnóstico. No concede autorización
para publicar releases, firmar o forzar etiquetas, instalar builds en
dispositivos ni acceder a secretos. No se hizo ninguna de esas cosas.

Archivos editados por este frente:

- `src-tauri/src/peer.rs` (propiedad del frente): receptor LAN, contrato y
  pruebas.
- `src-tauri/src/lib.rs`, solo en el receptor LAN y en el cierre de los
  despachos teléfono → PC / PC → teléfono (propiedad asignada al frente).
- `process/active/2026-10-27_interconectividad-telefono-pc/{TASKS,VALIDATION,
  APPROVAL}.md` y el nuevo `LAN-CONTRACT.md`.

No se editaron `protocol.rs`, `server.rs`, `status.rs`, `capabilities.rs`,
`mod.rs`, `src/features/mcp/`, los scripts MCP, el panel MCP, `docs/mcp/*`, el
PLAN global, ROADMAP, RELEASE_NOTES ni archivos Android de
conexión/credenciales/launcher. La versión del proyecto sigue en 1.0.1 y no se
preparó entrega.

## Validación técnica

| Comprobación | Resultado |
| --- | --- |
| Línea base de pruebas | `cargo test --lib` → **57 passed** antes de los cambios |
| Pruebas después del cambio | `cargo test --lib` → **62 passed, 0 failed** (5 nuevas del receptor HTTP real) |
| Compilación de escritorio | `cargo build --bin aurora-synapse` → `Finished`, sin avisos |
| Compilación Android (`check`, sin APK) | `cargo check --lib --target aarch64-linux-android` → `Finished`; 2 avisos previos en `lib.rs`, ninguno del módulo `peer` |
| Aislamiento de compilación | `CARGO_TARGET_DIR=src-tauri\target\interop` (ignorado por Git) porque el otro frente mantenía `target\debug` en ejecución; no se detuvo ningún proceso ajeno |

## Validación funcional registrada

- **Teléfono → PC (protocolo):** intercambio de invitación de un solo uso,
  `401` sin credencial en las tres rutas de despacho —también las heredadas—,
  una sola ejecución por petición, rechazo de orígenes web, credencial vencida,
  `app_id` inválido y propagación del rechazo de la app. Cubierto por pruebas
  sobre TCP real.
- **Teléfono → PC (físico):** pendiente. Requiere liberar el puerto 49295 y
  escanear un QR nuevo. **No se presenta como validado.**
- **PC → teléfono (físico):** estado `HTTP 200`, `gallery_status` con
  `paired: true, reachable: true`, transferencia real de 57 B con SHA-256
  coincidente, segundo plano, ausencia de proceso y recuperación comprobados en
  el Motorola edge 60 fusion.
- **PC → teléfono (apertura):** no soportada por Android; el PC responde con el
  motivo y no se inventó contrato. Registrado como límite real, no como éxito.
- **Super Galería:** `gallery.rs` sin cambios; no se ampliaron descargas,
  exclusiones ni integraciones. El archivo de prueba se retiró del teléfono.

## Coordinación con el frente paralelo

- El frente SYN-110-MCP tenía dos instancias de `target\debug\aurora-synapse.exe`
  en ejecución durante la sesión. No se detuvieron, no se reclamaron sus puertos
  y no se tocaron sus archivos.
- La app instalada (1.0.1, PID 2976) permaneció en ejecución; su receptor sigue
  siendo el anterior a este cambio. Por eso la verificación en vivo del receptor
  nuevo quedó pendiente.
- El contrato de red para el otro frente está en
  [LAN-CONTRACT.md](LAN-CONTRACT.md); no se renombró ningún contrato compartido.

## Cierre

**No APPROVED.** El agente ejecutor no se atribuye aceptación humana ni
aprobación de Biglex. Quedan pendientes:

1. Prueba física teléfono → PC con el receptor nuevo (QR y puerto 49295 libres).
2. Revisión de los cambios por Biglex e integración en SYN-110; el proceso
   global y el hito 1.1.0 no se cierran con este frente.
3. Actualización de `docs/mcp/PHONE-QR.md` y `docs/mcp/FOUNDATION.md` (nota de
   rutas heredadas) por el frente SYN-110-MCP al integrar.
4. Ajuste de etiqueta en `src/App.tsx` para las apps Android sin apertura remota
   (archivo compartido, no editado aquí).
5. Diferencia de puerto del beacon de Prisma (repositorio Prisma, fuera de
   alcance).
