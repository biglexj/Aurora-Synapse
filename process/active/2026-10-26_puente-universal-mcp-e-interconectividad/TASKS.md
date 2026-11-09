# Tareas técnicas

## Frentes preparados el 2026-10-27

- [x] Crear y delimitar [SYN-110-INTEROP](../2026-10-27_interconectividad-telefono-pc/PLAN.md) y [SYN-110-MCP](../2026-10-27_sesiones-y-conexion-mcp/PLAN.md) para trabajo paralelo de otros agentes.
- [ ] Integrar resultados y evidencias de ambos frentes en SYN-110, sin declarar cerrada la versión por una prueba parcial.

La interconectividad queda reactivada en su frente; descargas y ampliación de integraciones siguen aplazadas. Los apartados siguientes conservan la entrega anterior como antecedente.

Alcance vigente: base común y prueba del MCP de Aurora, por decisión posterior de Biglex. El inventario y las etapas amplias de abajo se conservan como pendientes futuros y no bloquean el cierre de esta entrega acotada.

## Entrega actual — base común

- [x] Contrastar la base existente: revisión Git `58a1afa` con cambios locales; servidor HTTP/stdio, registro de 20 herramientas y adaptadores presentes. No atribuir todo el trabajo local a esta entrega.
- [x] Publicar `aurora-synapse://capabilities` mediante recursos MCP, derivado de las herramientas ejecutables y sus esquemas, sin sondas ni efectos.
- [x] Caducar sesiones HTTP tras cinco minutos sin peticiones; el panel no renueva la actividad al consultarlas.
- [x] Añadir pruebas de coherencia del catálogo, recursos, autenticación y sesiones.
- [x] Ejecutar pruebas y conexión de solo lectura con el binario nuevo por HTTP y stdio: 43 pruebas Rust, 23 comprobaciones de panel y sonda real correctas.
- [x] Intentar MCP directo de Aurora y conector público; registrar respuesta funcional y estado del puente hacia Synapse.
- [x] Completar documentación de configuración y evidencias, sin cambiar versión ni publicar.
- [ ] Resolver conexión completa Aurora → Synapse desde el runtime correspondiente: el conector público responde, pero su puente Synapse no está configurado y Aurora Hub figura desconectado. No afirmar control del PC.

## Pendientes aplazados

Las secciones siguientes son la visión completa inicial. Descargas, teléfono ↔ PC, ampliación de integraciones y endurecimiento LAN quedan para una entrega posterior.

## 1. Inventario y contratos

- [ ] Registrar rama, revisión Git y cambios locales de la línea base.
- [ ] Crear matriz de todas las apps del catálogo: repositorio propietario, plataforma, dispositivo, acciones reales, transporte, argumentos, permisos, seguimiento y evidencia.
- [ ] Separar capacidades existentes, parciales y ausentes; contrastar catálogo, documentación Core y código propietario.
- [ ] Fijar las acciones comprometidas por etapa y las dependencias en otras apps, con referencias al trabajo correspondiente.
- [ ] Definir contrato común de destino, identificador de operación, estados, errores y resultados, conservando las semánticas de envío, aceptación y finalización.

## 2. Base fiable

- [ ] Resolver caducidad/actividad de sesiones MCP y ajustar el significado de agente conectado en backend e interfaz.
- [ ] Diseñar y ejecutar transición de las rutas LAN heredadas hacia autorización efectiva sin romper los envíos existentes.
- [ ] Unificar resolución de app/dispositivo, disponibilidad y permisos para entradas MCP e inter-app.
- [ ] Preservar instancia única y evitar duplicados ante reintentos, desconexiones y apertura de apps cerradas.
- [ ] Registrar auditoría útil por operación sin exponer credenciales ni contenido privado.

## 3. Agentes y descargas prioritarias

- [ ] Documentar y comprobar conexión de Codex mediante cliente MCP disponible, transporte y configuración reales; registrar limitaciones del entorno.
- [ ] Verificar descubrimiento, selección e invocación desde conversación real de Ely Intelligence.
- [ ] Completar Luna Fetch: recepción, trabajo identificable, consulta y archivo final verificable; ampliar la app propietaria si su contrato solo permite ACK.
- [ ] Inspeccionar Gallery-DL GUI e implementar adaptador MCP sobre sus capacidades reales, con seguimiento y resultado final.
- [ ] Validar errores, cancelación cuando exista y reintentos sin descarga duplicada; respetar la app elegida por el usuario.

## 4. Teléfono y PC

- [ ] Definir ruta autenticada entre dispositivos reutilizando el emparejamiento existente y manteniendo MCP local en loopback.
- [ ] Implementar apertura y una acción funcional acordada de teléfono → PC con retorno de estado y resultado.
- [ ] Implementar apertura y una acción funcional acordada de PC → teléfono con retorno de estado y resultado.
- [ ] Resolver disponibilidad con pantalla apagada, app en segundo plano y proceso ausente según restricciones reales de Android; documentar estados no soportados.
- [ ] Comprobar transferencias de Super Galería y exclusiones; conservar integridad y no sobrescritura.
- [ ] Completar un encadenamiento acordado entre apps/dispositivos con trazabilidad de cada paso.

## 5. Ecosistema y consolidación

- [ ] Completar apertura, reproducción y control de Prisma con comprobación posterior.
- [ ] Completar las variantes comprometidas de Prisma Upscaler con salida verificable y requisitos explícitos.
- [ ] Implementar las capacidades acordadas para las demás apps inventariadas; registrar como pendientes las que todavía carezcan de contrato externo.
- [ ] Sincronizar README, ROADMAP y docs/mcp con la implementación; corregir el contrato obsoleto del panel y las afirmaciones prematuras sobre Share Sheet y descubrimiento.
- [ ] Registrar cambios y pruebas por entrega, sin mezclar cambios ajenos ni atribuir aprobación humana.
- [ ] Actualizar RELEASE_NOTES al entregar implementación y completar VALIDATION/APPROVAL antes del cierre; sincronizar catálogos globales si posteriormente cambia la versión o se publica.
