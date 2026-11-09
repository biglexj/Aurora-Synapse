# SYN-110-INTEROP — Interconectividad teléfono ↔ PC

Fecha: 2026-10-27. Estado: ACTIVE, plan preparado para ejecución por otro agente. Hito: consolidación 1.1.0; no cambiar versión al iniciar.

## TAREA

Consolidar la comunicación entre teléfono y PC reutilizando emparejamiento, descubrimiento y transportes existentes, con destino explícito, autorización y resultado verificable.

## CONTEXTO

Proceso global: [SYN-110 — puente universal MCP e interconectividad](../2026-10-26_puente-universal-mcp-e-interconectividad/PLAN.md). Biglex solicita dos frentes en paralelo; esta petición reactiva la interconectividad como frente separado del trabajo de sesiones/MCP. Descargas y ampliación de integraciones continúan aplazadas.

La base anterior tiene HTTP loopback/stdio, 20 herramientas, recurso de capacidades y caducidad de sesiones. Se registraron 43 pruebas Rust, 23 comprobaciones del panel y una sonda correcta; son antecedentes, no evidencia del nuevo trabajo. Aurora público respondió, el MCP directo falló y el estado público señaló Hub desconectado y Synapse sin configurar. Revalidar lo necesario: no tratar esos datos como estado actual.

## FORMATO

### Alcance

Inventariar recorridos ya implementados en Luna Fetch, Prisma y Super Galería sin reescribirlos. Consolidar en Synapse descubrimiento, selección de destino, apertura de una app y una acción de solo lectura por dirección cuando exista contrato real. Si falta una capacidad, registrar y diseñar la ampliación mínima antes de implementarla. Conservar las transferencias existentes; no ampliar descargas ni el catálogo de integraciones.

### Propiedad de archivos y coordinación

`src-tauri/src/peer.rs`, receptor LAN y comandos de emparejamiento en `src-tauri/src/lib.rs`, `src-tauri/src/mcp/devices.rs`, `src/features/gallery/` y los archivos Android estrictamente relacionados con conexión/credenciales/launcher. No editar `protocol.rs`, `server.rs`, `status.rs`, `capabilities.rs` ni el panel MCP: pertenecen al otro frente.

El frente paralelo es **SYN-110-MCP**. Cada agente actualiza únicamente los documentos de su proceso. El PLAN global, ROADMAP y RELEASE_NOTES quedan para integración final; no editarlos simultáneamente. Si hace falta un archivo del otro frente, registrar una solicitud concreta de cambio y coordinar antes de tocarlo. No renombrar contratos compartidos unilateralmente.

Preferir checkouts/worktrees separados si se ejecutan ambos agentes; no compartir directorios de compilación Cargo/Android entre trabajos concurrentes. Si se usa el mismo checkout, respetar la propiedad de archivos y serializar compilaciones que escriban los mismos artefactos. No revertir ni añadir a commits cambios ajenos. No se han creado worktrees en esta entrega.

### Cierre

Completar TASKS y VALIDATION con evidencia por recorrido. Un bloqueo externo se registra como pendiente, no como éxito. Entregar cambios y referencias para integración en SYN-110. El cierre de este frente no cierra el proceso global ni acredita toda la 1.1.0.

## RESTRICCIONES

No exponer el MCP local a la LAN ni crear un servidor paralelo para resolver el enrutamiento. No modificar la lógica de descargas, habilitar escritorio remoto ni ampliar integraciones. Cambios en repositorios satélite solo cuando sean dependencia imprescindible del piloto: comprobar instrucciones locales y registrar archivos/contrato antes de intervenir.

No instalar builds en dispositivos, publicar releases, forzar etiquetas o hacer push forzado como consecuencia automática del plan. Preparar y probar lo autorizado; si una prueba exige una instalación no autorizada, dejarla pendiente y explicarla. No atribuir aceptación humana al agente.

