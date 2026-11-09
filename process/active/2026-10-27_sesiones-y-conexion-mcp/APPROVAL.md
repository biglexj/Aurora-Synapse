# SYN-110-MCP — autorización y cierre

Fecha: 2026-10-27. Estado: **ACTIVE — validación técnica completada; no APPROVED**. El cierre del frente no cierra SYN-110 ni acredita la 1.1.0.

## Autorización

Biglex encargó ejecutar el proceso SYN-110-MCP sobre el plan preparado: consolidar sesiones, descubrimiento y conexión real de agentes con Synapse y Aurora, reutilizando la base existente. La instrucción autoriza prueba e implementación acotada dentro de las restricciones del PLAN.

No se autorizó —y no se hizo— publicar releases, instalar builds en dispositivos, cambiar la versión, exponer el MCP a la LAN, crear credenciales, modificar el despliegue de Aurora, la configuración global de otro agente, la LAN/emparejamiento/Android del frente INTEROP ni los documentos globales compartidos.

## Validación técnica

Completada para el puente actual sobre `master` @ `58a1afa`:

- **57 pruebas Rust** aprobadas; compilación debug correcta.
- Sonda HTTP/stdio: 20 herramientas, recurso de capacidades idéntico, autenticación, rechazo de origen ajeno, cierre, aislamiento y recuperación tras 404.
- **Caducidad real de 300 s** verificada en ejecución (315 s de espera): la sesión no renovada caduca, el sondeo del panel no la renueva, una petición aceptada sí, y `initialize` recupera tras el 404.
- **28 comprobaciones del panel** correctas.
- **Agente real Codex 0.164** conectado por HTTP y por stdio; descubrió el catálogo y ejecutó `apps_list` de solo lectura. En HTTP la sesión apareció y se cerró (1 → 0). No se registró Synapse como herramienta nativa de esta interfaz de chat.

## Diagnóstico de Aurora

- **Público** (`https://api.biglexj.com/api/v1/mcp`): lectura funcional (8 herramientas públicas, entrada publicada, `mcp.status` y contexto). `host_bridge=disconnected`, `invocation_audit=unavailable`, `synapse_bridge.configured=false` por falta de `AURORA_SYNAPSE_MCP_URL`. No acredita control del PC.
- **MCP directo local** (`127.0.0.1:5017`): funcional; 8 herramientas anónimas y 59 con la frontera técnica local. El `-32603` anterior no se reprodujo; el catálogo directo usa `mcp.status` (con punto), no `mcp_status`, y la disponibilidad del lab influye. Sin causa única confirmada.
- **Aurora Hub**: sin proceso Hub; `host_bridge=disconnected`. No se inició ni modificó.
- **Aurora → Synapse**: verificado en el runtime local (`connected=true`, 20 herramientas, `synapse.apps_list` ejecutado). El runtime público no puede: no tiene la variable y no alcanza el loopback del PC. Falta un canal autenticado y un cambio de despliegue.

## Límites y bloqueos

1. **Aurora público → Synapse**: bloqueado por configuración del despliegue y por la topología loopback. No se simuló ni se abrió el MCP a la red.
2. **App instalada en ejecución**: es la 1.0.1 del 26-10 21:30, anterior al código actual; una sonda seguía viva a los 5 minutos y el panel retenía 8 sesiones. Hay que reiniciar/actualizar para aplicar la caducidad. No se reinició ni instaló.
3. **Aurora Hub**: validar cuando el Host esté en marcha y emparejado.
4. **Consumidores esporádicos**: el primer `tools/call` tras 300 s recibe 404; el puente de Ely/Aurora limpia y rehace el handshake en la llamada siguiente, sin reintentar la fallida. El refresco de 30 s evita el caso.
5. **Conversación completa de Ely Intelligence / registro nativo en el chat**: no verificados. La evidencia de agente real usa Codex.
6. **Coordinación concurrente**: el frente INTEROP modificó `peer.rs`/`lib.rs` y compilaba al cierre de esta pasada. No se tocaron; el binario probado (09:13:45, SHA-256 registrado en VALIDATION) no incluye esos cambios posteriores. Su integración debe recompilarse y probarse aparte.
7. Android, teléfono y emparejamiento pertenecen a SYN-110-INTEROP; no se tocaron.

## Aceptación humana

No hay aprobación humana del comportamiento nuevo ni de una entrega 1.1.0. El agente no se atribuye aceptación de Biglex. La decisión de reiniciar/actualizar la app, configurar el runtime público o preparar la release queda fuera de esta pasada.

## Cierre

**No APPROVED.** Se entrega para integración en SYN-110 con cambios, evidencia y bloqueos explícitos. El proceso global sigue activo y no se cierra por esta validación parcial. No se modificó la versión, no se publicó ni instaló nada y no se alteraron contratos del frente paralelo.

Detalle: [TASKS.md](TASKS.md) y [VALIDATION.md](VALIDATION.md). Resultados técnicos: [docs/mcp/FOUNDATION.md](../../../docs/mcp/FOUNDATION.md) y [UI_CONTRACT.md](../../../docs/mcp/UI_CONTRACT.md).

---

# Segunda ejecución — 2026-10-28

Estado: **ACTIVE — validación técnica completada; no APPROVED**. El cierre del
frente no cierra SYN-110 ni acredita la 1.1.0.

## Autorización

Misma autorización que la primera pasada: consolidar sesiones, descubrimiento y
conexión real de agentes con Synapse y Aurora, reutilizando la base existente.
Se ejecutó además el diagnóstico de los cuatro servicios de Aurora.

No se autorizó —y no se hizo— publicar releases, instalar builds en dispositivos,
cambiar la versión, exponer el MCP a la LAN, crear credenciales, modificar el
despliegue de Aurora, la configuración global de otro agente, la
LAN/emparejamiento/Android del frente INTEROP ni los documentos globales
compartidos.

## Validación técnica de esta ejecución

- **51 pruebas Rust** aprobadas (43 previas + 8 nuevas); compilación debug
  correcta; `tsc --noEmit` y `vite build` correctos.
- **Cambio de contrato en el módulo MCP**: la renovación ocurre solo tras una
  petición aceptada; `initialize` retira la sesión anterior y reabre sesión
  incluso con la cabecera caducada; el 404 explica cómo recuperarse; el panel
  informa de la última petición aceptada de cada cliente.
- Sonda HTTP/stdio ampliada: 20 herramientas, recurso idéntico en ambos
  transportes, clave incorrecta rechazada, origen ajeno rechazado, aislamiento
  entre dos clientes, recuperación tras 404 y contrato de sesión publicado en
  el recurso.
- **Caducidad real de 300 s** verificada en ejecución con el código nuevo.
- **28 comprobaciones del panel** correctas.
- **Agente real con registro nativo**: Claude Code conectado por stdio; en el
  chat quedaron registradas `ListMcpResourcesTool`, `ReadMcpResourceTool` y
  `mcp__aurora-synapse__classify_intent`. No es una sonda JSON-RPC. Es la
  diferencia pendiente en la primera pasada, ya cerrada.

## Diagnóstico de Aurora de esta ejecución

Con el **backend real** de Aurora levantado en `APP_HOST=127.0.0.1` (solo
loopback, detenido al terminar):

- **MCP directo** (`127.0.0.1:5000/api/v1/mcp`): funcional —
  `serverInfo aurora-mcp 1.0.0`, 17 herramientas remotas y 59 en el control
  plane, 20 de ellas `synapse.*`. La causa del fallo directo anterior era que
  **el backend no estaba en marcha**, no la credencial del cliente.
- **Aurora → Synapse**: **verificado con lectura real del puente en este PC**:
  `synapse_bridge` con `configured: true`, `connected: true`, `tools: 20`,
  `last_error: null`, y el log «Catálogo del puente Aurora Synapse actualizado».
- **Aurora Hub**: `host_bridge: disconnected` y `hosts/status` sin host
  conectado, porque **el proceso Aurora Hub no está en marcha** en este equipo.
- **Público** (`https://api.biglexj.com/api/v1/mcp`): `initialize`, `tools/list`
  y `aurora_site_context` responden sin credencial; el estado interno del
  runtime publicado lo lee la sonda: `host_bridge=disconnected`,
  `invocation_audit=unavailable`, `synapse_bridge.configured=false` (sin
  `AURORA_SYNAPSE_MCP_URL`). No se enviaron credenciales locales a un host
  remoto y no acredita control del PC.

## Límites y bloqueos de esta ejecución

1. **Aurora → Synapse público**: bloqueado por configuración del despliegue y
   por la topología loopback. El enlace **local** sí está verificado; el remoto
   no se simula ni se presenta como resuelto.
2. **Estado interno del runtime público**: leído por la sonda
   (`synapse_bridge.configured=false`, `host_bridge=disconnected`). Solo queda
   sin resolver el canal autenticado hasta el PC; una llamada directa a
   `mcp_status` sin credencial devuelve `invalid_token`.
3. **App instalada en ejecución**: arrastra el binario anterior —responde al
   404 sin `detail`—. Hay que reiniciar/actualizar para aplicar el contrato
   nuevo. No se reinició ni se instaló.
4. **Aurora Hub**: validar cuando el Host esté en marcha y emparejado.
5. **Consumidores esporádicos**: el primer `tools/call` tras 300 s recibe 404;
   el refresco de 30 s evita el caso.
6. **Conversación completa de Ely Intelligence**: no verificada. La evidencia de
   agente real es un cliente MCP nativo de esta interfaz, no Ely.
7. **Coordinación concurrente**: `peer.rs`, `devices.rs` y `lib.rs` pertenecen
   al frente INTEROP. No se tocaron, no se revirtieron y no se les atribuyen
   cambios. El binario probado es el compilado en esta pasada; cualquier
   cambio posterior del frente INTEROP requiere recompilar y volver a probar.
8. Android, teléfono y emparejamiento pertenecen a SYN-110-INTEROP; no se
   tocaron.

## Aceptación humana

No hay aprobación humana del comportamiento nuevo ni de una entrega 1.1.0. El
agente no se atribuye aceptación de Biglex. La decisión de reiniciar o
actualizar la app, configurar el runtime público o preparar la release queda
fuera de esta pasada.

## Cierre

**No APPROVED.** Se entrega para integración en SYN-110 con cambios, evidencia
y bloqueos explícitos. El proceso global sigue activo y no se cierra por esta
validación parcial. No se modificó la versión, no se publicó ni instaló nada y
no se alteraron contratos del frente paralelo.

Detalle: [TASKS.md](TASKS.md) y [VALIDATION.md](VALIDATION.md). Documento
canónico del frente: [docs/mcp/SESIONES.md](../../../docs/mcp/SESIONES.md).
Evidencias: [docs/mcp/evidencia/](../../../docs/mcp/evidencia/).
