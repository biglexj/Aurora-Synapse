# Validación

Fecha de apertura: 2026-10-26. Estado: entrega acotada de base común comprobada; preparación de 1.1.0 y conexión Aurora → Synapse pendientes. Los recorridos de descargas, teléfono ↔ PC e integraciones se aplazan por decisión de Biglex.

## Entrega actual — evidencia del 2026-10-26

Línea base Git: `master`, revisión `58a1afa`, con cambios locales previos extensos. Archivos de esta implementación: `src-tauri/src/mcp/{capabilities,protocol,server,mod}.rs`, sonda `scripts/mcp/verify-foundation.mjs` y documentación. Se reutilizan los adaptadores existentes; no se atribuye esta entrega al conjunto completo de archivos sin seguimiento.

- [x] `cargo test --manifest-path src-tauri/Cargo.toml --lib`: **43 pruebas**, todas correctas. Incluyen caducidad sin reanimación de sesión vencida, renovación por petición y no por panel, coherencia exacta del recurso con el catálogo, lectura HTTP y autorización del recurso.
- [x] `cargo build --manifest-path src-tauri/Cargo.toml --bin aurora-synapse`: correcto. Advertencia no bloqueante de `parse_invitation` sin uso en la compilación de escritorio.
- [x] Sonda `verify-foundation.mjs` ejecutada con Bun contra el binario nuevo: resultado `ok`, HTTP loopback y stdio, 20 herramientas, recurso de capacidades idéntico en ambos transportes, rechazo sin Bearer, negociación, `classify_intent` de solo lectura y cierre de sesión. Reutilizar la sesión cerrada devuelve 404.
- [x] `bun scripts/mcp/verify-mcp-panel.mjs`: **23 comprobaciones**, todas correctas.
- [x] Conector público Aurora MCP: `aurora_site_context`, `mcp_status` y `list_posts(status=published, limit=1)` responden. La consulta devuelve una entrada publicada; no se infiere control del PC de esa lectura.
- [x] Intento directo MCP `aurora`: `mcp_status` y `aurora_site_context` devuelven `-32603` con error genérico de servicio/autenticación. Causa concreta no identificada.
- [x] Estado público de Aurora registrado: `host_bridge=disconnected`, auditoría `unavailable`, Synapse sin configurar por ausencia de `AURORA_SYNAPSE_MCP_URL` en ese runtime. No se modificó el backend remoto ni se abrió el puerto MCP a la red.
- [x] Documentación y enlaces locales comprobados; `git diff --check` sin errores de espacios.

La prueba local es un cliente JSON-RPC ejecutado por este agente; no acredita que Synapse esté registrado como herramienta nativa del chat ni una conversación completa de Ely Intelligence. Los procesos de la sonda se terminaron al finalizar. No se ejecutaron descargas, transferencias o escalados ni se instalaron/publicaron builds. No se repitió build web por no modificar frontend; la regresión del panel sí se ejecutó.

El alcance de código de la base común queda entregado. La conexión completa Aurora → PC no queda conseguida: requiere configurar un canal autenticado accesible desde el runtime correcto. Esa limitación permanece abierta y no se sustituye por una URL loopback remota.

## Línea base conocida

La revisión anterior de esta conversación ejecutó correctamente `bun run build`, `bun scripts/mcp/verify-mcp-panel.mjs` (23 comprobaciones) y `cargo test --manifest-path src-tauri/Cargo.toml --lib` (38 pruebas). Son resultados previos a la implementación de este proceso y no cierran sus criterios funcionales.

La [verificación del 23 de octubre](../../../docs/mcp/VERIFICATION.md) registra transferencias físicas, escalado CLI y límites. [PHONE-QR.md](../../../docs/mcp/PHONE-QR.md) registra el emparejamiento. Son antecedentes, no pruebas repetidas al crear este proceso. Biglex confirma ahora que el envío de Super Galería funciona bien; la confirmación se limita a ese comportamiento.

## Registro de recorridos futuros — aplazados

Cada resultado nuevo debe identificar fecha, app y dispositivo de origen/destino, cliente agente cuando corresponda, revisión Git y cambios locales relevantes, acción, resultado esperado/observado y evidencia. Omitir secretos y minimizar contenido privado. Registrar bloqueos de entorno sin marcar la comprobación como aprobada.

| Recorrido | Evidencia requerida | Estado |
| --- | --- | --- |
| Descubrimiento desde Codex | Conexión real, catálogo y llamada con respuesta de la app | Pendiente |
| Conversación de Ely Intelligence | Petición → herramienta seleccionada → app → respuesta final sustentada | Pendiente |
| Luna Fetch desde ambos clientes | Trabajo correlacionado y archivo final verificable, no solo ACK | Pendiente |
| Gallery-DL GUI desde ambos clientes | Herramienta MCP real, trabajo y descarga final verificable | Pendiente |
| Teléfono → PC | Apertura y acción acordada ejecutadas en el PC, resultado recibido en origen | Pendiente |
| PC → teléfono | Apertura y acción acordada ejecutadas en Android, resultado recibido en origen | Pendiente |
| Super Galería | Envío/recepción, tamaño y SHA-256; exclusiones y destino ocupado respetados | Antecedente favorable; regresión pendiente |
| Prisma | Estado posterior confirma reproducción/control pedidos | Pendiente |
| Upscaler | Variante comprometida produce archivo nuevo válido | Antecedente CLI FSR1; cierre pendiente |
| Resto del catálogo | Cada acción comprometida en la matriz tiene evidencia propia | Pendiente |
| Encadenamiento | Cada paso conserva operación, destino y resultado comprobable | Pendiente |

## Comprobaciones de la visión amplia — aplazadas salvo evidencia anterior

- [ ] Build web y pruebas pertinentes a los cambios nuevos.
- [ ] MCP HTTP/stdio: negociación, errores, sesión y compatibilidad con los clientes usados.
- [ ] Cliente desaparecido o sesión inactiva deja de aparecer como conectado según contrato definido.
- [ ] Credencial ausente/revocada, origen no autorizado y dispositivo no emparejado no ejecutan acciones.
- [ ] App cerrada, no instalada, dispositivo ausente y varios destinos posibles generan estados precisos.
- [ ] Rechazo del receptor o HTTP 200 con fallo funcional no se presenta como éxito.
- [ ] Corte de conexión y reintento no duplican la acción; timeout no implica cancelación ni finalización.
- [ ] Destino ocupado, transferencia incompleta o hash incorrecto no sobrescriben ni se anuncian como completados.
- [ ] Segundo plano y reinicio Android/PC se prueban físicamente para las capacidades comprometidas.
- [ ] Documentación final coincide con las capacidades implementadas y verificadas.

## Revisión de esta entrega documental

- [x] Objetivo, etapas, tareas y criterios de cierre registrados sin declarar implementación nueva.
- [x] Separación explícita entre autorización documental, pruebas anteriores y validación funcional pendiente.
- [x] Enlaces locales de los documentos creados/modificados comprobados.

No se requiere recompilar la aplicación por esta entrega exclusivamente documental.
