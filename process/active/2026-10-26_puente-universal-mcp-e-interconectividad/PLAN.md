# Puente universal MCP e interconectividad

Identificador global: **SYN-110**.

## Coordinación vigente — 2026-10-27

Biglex solicita preparar dos frentes para otros agentes en paralelo. La interconectividad se reactiva como frente propio; descargas y ampliación del catálogo de integraciones siguen aplazadas. Los planes están creados; su creación no significa que los agentes se hayan iniciado ni que se haya implementado su alcance.

| Identificador | Proceso | Responsabilidad |
| --- | --- | --- |
| SYN-110-INTEROP | [Interconectividad teléfono ↔ PC](../2026-10-27_interconectividad-telefono-pc/PLAN.md) | Descubrimiento, emparejamiento, LAN, destino y recorridos piloto bidireccionales |
| SYN-110-MCP | [Sesiones y conexión MCP](../2026-10-27_sesiones-y-conexion-mcp/PLAN.md) | Sesiones, catálogo, transporte MCP, conexión de agentes y diagnóstico Aurora |

Cada frente conserva tareas y evidencias propias. Sus PLAN asignan archivos y límites para evitar colisiones. Este proceso global integra los resultados y conserva el hito 1.1.0; no se cierra por el éxito aislado de uno de ellos. La decisión actual prevalece sobre el aplazamiento previo de toda la interconectividad.

Fecha: 2026-10-26. Estado: activo, base común en implementación. Primer proceso formal del proyecto. Hito objetivo: 1.1.0; la versión del binario permanece en 1.0.1 hasta preparar la entrega.

## Alcance vigente — decisión posterior de Biglex

En esta conversación Biglex autoriza ejecutar únicamente **la base común y probar la conexión del agente al MCP de Aurora**. Descargas, interacción teléfono ↔ PC e integraciones adicionales quedan para después. Esta decisión sustituye las etapas y criterios amplios de la propuesta inicial conservados más abajo como visión futura, no como requisitos de cierre de este hito.

Entregables actuales:

1. Contrastar servidor, catálogo y adaptadores existentes para reutilizarlos.
2. Publicar un contrato de capacidades de solo lectura derivado del catálogo ejecutable, con esquemas, soporte, transportes y límites explícitos. El descubrimiento no debe inventar disponibilidad ni ejecutar acciones.
3. Evitar sesiones HTTP indefinidas y documentar qué significa actividad reciente del agente.
4. Probar el binario real por HTTP loopback y stdio, con negociación, catálogo, autenticación, lectura y cierre de sesión. No ejecutar descargas ni transferencias.
5. Intentar conexión desde este agente al MCP de Aurora, consultar datos y distinguir el servicio público, Aurora Hub y el puente Synapse. Un fallo o configuración ausente se registra como límite; no se reconfiguran servidores remotos en silencio.
6. Documentar configuración reproducible para clientes y resultados por servicio.

Criterios de cierre de esta entrega acotada: contrato y sesiones probados; conexión local HTTP/stdio comprobada; intento real de Aurora con resultado y límites registrados; documentación sincronizada. Si Aurora responde pero su Synapse remoto no está configurado, no afirmar conexión Aurora → PC. La preparación/publicación de 1.1.0 y la conexión de todos los consumidores siguen pendientes.

No modificar rutas LAN heredadas ni contratos de apps satélite en esta entrega: corresponden al trabajo posterior de interconectividad/integraciones. Preservar y documentar esa deuda, sin afirmar que quedó resuelta por reforzar MCP local.

## TAREA

Convertir Aurora Synapse en el puente común de capacidades del ecosistema: aplicaciones, dispositivos y agentes deben poder descubrir qué se puede hacer, seleccionar dónde hacerlo, solicitar la acción y comprobar su resultado.

MCP es una interfaz fundamental para los agentes, incluidos Codex, Ely Intelligence y otros clientes compatibles. La interconectividad entre apps utiliza la misma lógica de capacidades y resultados mediante los transportes adecuados. Tener una app en el catálogo o poder abrirla no equivale a poder operarla por MCP.

## CONTEXTO

Biglex define esta dirección en la conversación del 2026-10-26 y solicita crear el proceso antes de seguir avanzando. Incluye operar las apps mediante agentes, ordenar acciones desde teléfono a PC y desde PC a teléfono, descargar con Luna Fetch y Gallery-DL GUI, y conservar las transferencias de Super Galería. La referencia oral «Luna File» se interpreta como Luna Fetch, nombre presente en el proyecto.

Este proceso amplía el alcance limitado del [plan del 23 de octubre](../../../docs/mcp/PLAN.md). Ese documento y su [verificación](../../../docs/mcp/VERIFICATION.md) se conservan como antecedentes; este PLAN y sus TASKS gobiernan el trabajo nuevo.

### Punto de partida

| Área | Evidencia disponible | Límite conocido |
| --- | --- | --- |
| MCP | Código HTTP loopback/stdio y catálogo de 20 herramientas | No acredita conexión de todos los clientes ni operación completa de todas las apps |
| Luna Fetch | Adaptador de estado, descarga, análisis, carpeta y foco | Falta descarga real con archivo final y contrato de seguimiento de trabajos |
| Gallery-DL GUI | Despacho previo y consulta de disponibilidad | No tiene herramientas MCP dedicadas en el catálogo actual; falta verificar el contrato de descarga |
| Prisma | Adaptador y lectura de estado registrada | Control y reproducción con comprobación posterior pendientes |
| Prisma Upscaler | Escalado CLI FSR1 documentado con salida nueva | NCNN y transporte HTTP real pendientes |
| Super Galería | Transferencias físicas con tamaño/SHA-256 documentadas; Biglex confirma en esta conversación que el envío funciona bien | No extender esa confirmación a todos los recorridos, clientes o persistencia Android |
| Teléfono ↔ PC | Emparejamiento QR implementado y confirmación histórica documentada | Falta matriz completa de acciones bidireccionales y disponibilidad en segundo plano |
| Calidad local | Revisión previa en esta conversación: build web, 23 comprobaciones del panel y 38 pruebas Rust correctas | No valida los recorridos completos de este proceso |

El trabajo reciente está parcialmente sin commit. La línea base debe registrar revisión Git y cambios locales, sin atribuir autoría ni validación a partir de fechas de archivo.

### Visión futura: casos de uso fuera de la entrega actual

1. «Descarga este enlace con Luna Fetch»: el agente descubre la capacidad, identifica dispositivo y destino, solicita la descarga y consulta su resultado. Solo anuncia finalización con evidencia del archivo o resultado final verificable.
2. «Descarga esta galería con Gallery-DL GUI»: mismo recorrido con su adaptador real, sin sustituir silenciosamente la app elegida.
3. Desde el teléfono, abrir una app instalada en el PC y solicitar una acción compatible; recibir aceptación, progreso cuando exista y resultado o error.
4. Desde el PC o un agente, abrir una app o solicitar una acción compatible en el teléfono emparejado, sujeto a las restricciones reales de Android.
5. Enviar un archivo a Super Galería o recuperarlo desde el catálogo autorizado sin perder las exclusiones ni la integridad de las transferencias existentes.
6. Encadenar capacidades cuando corresponda: descargar en el PC y después abrir el resultado con Prisma o enviarlo al teléfono, manteniendo trazabilidad de cada paso.

## FORMATO

### Arquitectura objetivo

`Usuario / agente / app → entrada MCP o inter-app → capacidades y permisos → dispositivo y adaptador → app receptora → estado y resultado`

- Un registro común identifica app, dispositivo, acciones, esquemas de argumentos, transporte, disponibilidad, permisos y forma de comprobar resultados.
- El agente consume herramientas MCP; el adaptador traduce a HTTP, IPC, URI o CLI realmente implementados en la app. No duplicar la lógica de negocio de cada app dentro de Synapse.
- Cada orden identifica el destino. Si hay varias instancias o dispositivos y la intención no permite resolverlo, se solicita elección antes de producir efectos.
- Distinguir app conocida, instalada, alcanzable, autorizada y capaz de ejecutar una acción. Un beacon o una respuesta HTTP no acredita todas esas condiciones.
- Distinguir apertura/foco de ejecución funcional. «Ejecutar desde el teléfono» significa ordenar la ejecución en el equipo que tiene instalada la app; no ejecutar un binario Windows dentro de Android ni implementar escritorio remoto.
- Las acciones largas necesitan identificador de trabajo, estados y resultado consultable, mediante contrato existente o una ampliación explícita en el repositorio propietario. `sent`, `accepted` y `completed` conservan significados distintos.
- La conexión MCP local permanece en loopback/stdio. La comunicación entre dispositivos utiliza un canal emparejado y autenticado; no se obtiene exponiendo el MCP local a la LAN.
- Clientes ejecutados fuera del PC necesitan una ruta autenticada específica. Registrar dónde corre cada cliente; no presentar loopback como acceso remoto.

### Propuesta inicial amplia — etapas futuras, alcance sustituido arriba

1. **Inventario y contratos:** comprobar las superficies reales y crear una matriz por app, dispositivo y acción. Registrar los cambios requeridos en los repositorios propietarios.
2. **Base fiable:** resolver sesiones MCP sin caducidad, autorización de rutas LAN heredadas, disponibilidad y enrutamiento por dispositivo. Mantener compatibilidad mediante una transición documentada.
3. **Primer recorrido con agentes:** conectar un cliente Codex compatible y Ely Intelligence; completar descargas con Luna Fetch y Gallery-DL GUI, incluyendo resultado final y errores.
4. **Interconectividad bidireccional:** completar apertura y acciones teléfono → PC y PC → teléfono, transferencia y consulta de resultados. Reutilizar emparejamiento y Super Galería.
5. **Extensión al ecosistema:** cerrar Prisma y Upscaler; incorporar por capacidad comprobada Ely-Tesia, WinTTS, LyraFlow, MouziFlow, DaVinci Flow y las demás apps del catálogo. Inventariar todas; no publicar herramientas ficticias para cubrir la lista.
6. **Consolidación:** documentación coherente, evidencias por recorrido y cambios identificables en Git. Las futuras ampliaciones deben usar el mismo contrato.

La matriz de inventario fijará las acciones concretas de las etapas 4 y 5 antes de implementarlas. Cada fila conserva su estado; el proceso no se cierra globalmente mientras queden capacidades comprometidas pendientes. Una reducción de alcance debe quedar explícita, sin convertir pendientes en completados.

### Criterios de la visión completa — aplazados

- Un agente descubre e invoca las capacidades del catálogo implementado y obtiene errores útiles cuando una app o dispositivo no está disponible.
- Descargas completas verificadas con Luna Fetch y Gallery-DL GUI desde un cliente Codex compatible y desde Ely Intelligence.
- Apertura y al menos una acción funcional acordada por dirección, teléfono → PC y PC → teléfono, comprobadas en equipos reales.
- Transferencias de Super Galería conservadas, comprobadas y con tamaño/SHA-256 cuando corresponda.
- Matriz de apps y acciones comprometidas completada con evidencia, incluidas las limitaciones justificadas de plataforma.
- Sesiones, permisos, ausencia de destino, reintentos y errores de receptor no producen falsos estados de conexión o éxito ni duplicación de tareas.
- Documentación, resultados de pruebas y referencias Git permiten distinguir implementación, validación técnica y aceptación del usuario.

## RESTRICCIONES

- La petición posterior autoriza implementar la base común y probar conexiones MCP. No autoriza publicar releases, instalar builds ni ejecutar las funciones aplazadas.
- Preservar las funciones existentes y las exclusiones de contenido del usuario. No sobrescribir archivos durante transferencias o pruebas.
- No ofrecer ejecución arbitraria de consola: las acciones remotas corresponden a capacidades explícitas de apps autorizadas.
- No introducir reintentos mutantes o cambios de app/dispositivo que dupliquen trabajos o alteren el destino pedido.
- No tratar contratos históricos de Core como pruebas de implementación actual. El código propietario y las pruebas reales determinan cada capacidad.
- Acceso por Internet, escritorio remoto, portapapeles universal y Share Sheet no son requisitos de cierre inicial. Documentar dependencias si un recorrido acordado los necesita; no ampliar el proceso por asociación.
- No modificar otras apps durante esta entrega acotada. Sus cambios necesarios se registrarán como dependencias y trabajo coordinado.

## Referencias

- [Tareas](TASKS.md), [validación](VALIDATION.md) y [autorización y cierre](APPROVAL.md).
- [Contrato del panel](../../../docs/mcp/UI_CONTRACT.md) y [conexión QR](../../../docs/mcp/PHONE-QR.md): contrastar con implementación vigente.
- [Flujo Core](../../../../Docs/global/documentation/process-workflow.md).
- [Arquitectura compartida](../../../../Docs/features/aurora-synapse/README.md), [puente MCP](../../../../Docs/features/aurora-synapse/bridge-as-mcp-server.md) y [entrada para agentes](../../../../Docs/features/aurora-synapse/agent-and-cli-gateway.md).
