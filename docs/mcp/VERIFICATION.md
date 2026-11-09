# Verificación de los adaptadores MCP

Fecha: 2026-10-23; **actualizado el 2026-10-28**. Estado: primera parte
implementada; integración completa pendiente.

> Documento de antecedentes. El estado vigente del frente de sesiones y
> conexión está en [SESIONES.md](SESIONES.md) y en
> [FOUNDATION.md](FOUNDATION.md); las evidencias en [evidencia/](evidencia/).

## Ejecutado

- `cargo test --manifest-path src-tauri/Cargo.toml mcp:: -- --nocapture`:
  28 pruebas aprobadas el 2026-10-23; **48 el 2026-10-28** tras incorporar el
  contrato de sesión (48 + 14 de `peer.rs` = 62 en el crate completo).
  Incluyen argumentos inválidos antes de producir efectos,
  rechazo de una app aunque responda HTTP 200, PNG inválido y protección frente
  a sobrescritura de un destino que aparece durante la operación.
- `cargo build --manifest-path src-tauri/Cargo.toml --bin aurora-synapse`:
  compilación de desarrollo correcta.
- `node scripts/mcp/verify-local.mjs`: binario real en un puerto loopback temporal,
  handshake, notificación, 20 herramientas por HTTP y stdio y cierre del proceso
  creado para la prueba. No arranca ni reinicia la interfaz.
- La sonda pasó en `pwsh` y `powershell.exe`: invocación correcta, retorno 1 ante
  error de herramienta y limpieza de sesiones en ambos casos.
- Prisma instalado respondió a `prisma_status` a través del puente nuevo.
- Luna Fetch no respondió: el catálogo informa `supported: true`,
  `reachable: false`, `availability: offline`.
- Upscaler sin HTTP y con CLI instalada informa `cli_available`; no confunde la
  instalación con un servicio en ejecución.
- Escalado real con CLI instalada 1.0.2, motor `fsr1`, escala 2 y nitidez 0.5.
  Entrada: `src-tauri/icons/32x32.png`. Salida: PNG RGBA de 64 × 64, 5403 bytes,
  decodificable y verificado además con Pillow. Un segundo intento sobre la
  misma ruta devolvió error y preservó todos sus bytes.
- Artefacto de esa prueba:
  `C:\Users\biglexj\AppData\Local\Temp\aurora-synapse-mcp-KMjhSh\resultado FSR1.png`.

## Límites y trabajo pendiente

Actualización de la etapa de teléfono e Intelligence:

- Compilación Tauri de escritorio correcta y abierta en 127.0.0.1:49296.
- APK debug de Synapse y Super Galería instalados en el Motorola; `dumpsys package`
  confirma actualizaciones de 2026-10-23 a las 08:19. No son releases publicadas.
- Backend local: 83 pruebas MCP aprobadas y una ignorada. La regresión cubre
  catálogo remoto, invocación, confirmación y propagación de rechazo de la app.
- La API autenticada de Intelligence descubre 20 herramientas, cinco de galería,
  y ejecuta `synapse.gallery_status`. El teléfono quedó conectado y autorizado
  mediante el código introducido directamente por el usuario.
- La conversación real detectó una falta de traducción entre
  `synapse_gallery_status` y `synapse.gallery_status`; se corrigió la resolución
  contra los nombres publicados y su regresión pasó. La repetición agotó 150 s
  esperando al proveedor; no hay evidencia de chat completo correcto.
- El proveedor usó fallback a Gemini 3.5 Flash Lite durante esa prueba; no hay
  evidencia de respuesta de Gemini 3.8. No se cambió la política de proveedores.
- Transferencias físicas por la API autenticada de Intelligence verificadas:
  envío de PNG de 1041 bytes, confirmación de tamaño/SHA-256 y descarga idéntica.
  SHA-256: `6573691b16e6d66be9c3bab383aad5b58054993d0774aae4d59c8dcbcd1c1eae`.
  Rechazo de sobrescritura verificado a través del endpoint de confirmación.
- Download está excluida por configuración del usuario. La imagen enviada allí
  queda fuera del catálogo, como corresponde. Se copió únicamente esa imagen
  mediante ADB a Pictures/SynapseVerification y se verificó su consulta/descarga
  por ID (`1000066964`). No se modificaron las exclusiones del usuario.
- Sonda de recepción ejecutada:
  `node scripts/mcp/verify-intelligence.mjs --gallery-receive=synapse_mcp_1792779614547.png`.
  Salida: `%TEMP%/synapse-intelligence-gallery-Mad2p2/recibida.png`.
- Synapse Android está instalado, pero su emparejamiento local y un servidor MCP
  persistente en segundo plano del teléfono no se verificaron. El MCP comprobado
  corre en el PC y accede al catálogo del Motorola por LAN.

- No se ejecutó una descarga real de Luna Fetch ni se comprobó su archivo final.
  Su endpoint actual confirma recepción y no ofrece seguimiento de finalización.
- Prisma respondió en lectura; apertura y control mutantes con comprobación
  posterior de reproducción siguen pendientes.
- La API local está verificada; no se afirma acceso desde un backend remoto
  o desde la web publicada.
- No se verificaron NCNN local ni el servidor HTTP del Upscaler en vivo. NCNN
  requiere `PRISMA_UPSCALER_BIN` y `PRISMA_MODELS_DIR` configurados para la CLI.
- El tiempo de espera del consumidor debe permitir un escalado de hasta 120 s;
  el cliente de Ely Intelligence ahora usa un valor predeterminado de 150 s.
- La comprobación del resultado en el puente valida cabecera PNG, dimensiones,
  guardado y bytes; la decodificación completa con Pillow se hizo en esta prueba.
- No se publicó una release. Las apps aplazadas no están integradas.

La comprobación usa rutas locales y aplicaciones instaladas en el equipo donde
se ejecuta. Repetirla escribe un resultado nuevo en `%TEMP%` y su auditoría local.
