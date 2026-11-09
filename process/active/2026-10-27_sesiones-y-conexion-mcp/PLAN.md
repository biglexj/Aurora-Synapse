# SYN-110-MCP — Sesiones y conexión MCP de agentes

Fecha: 2026-10-27. Estado: ACTIVE, plan preparado para ejecución por otro agente. Hito: consolidación 1.1.0; no cambiar versión al iniciar.

## TAREA

Consolidar sesiones, descubrimiento y conexión real de agentes con Synapse y Aurora, partiendo de la base común existente y distinguiendo los servicios locales y remotos.

## CONTEXTO

Proceso global: [SYN-110 — puente universal MCP e interconectividad](../2026-10-26_puente-universal-mcp-e-interconectividad/PLAN.md). Biglex solicita dos frentes en paralelo; esta petición reactiva la interconectividad como frente separado del trabajo de sesiones/MCP. Descargas y ampliación de integraciones continúan aplazadas.

La base anterior tiene HTTP loopback/stdio, 20 herramientas, recurso de capacidades y caducidad de sesiones. Se registraron 43 pruebas Rust, 23 comprobaciones del panel y una sonda correcta; son antecedentes, no evidencia del nuevo trabajo. Aurora público respondió, el MCP directo falló y el estado público señaló Hub desconectado y Synapse sin configurar. Revalidar lo necesario: no tratar esos datos como estado actual.

## FORMATO

### Alcance

Contrastar la caducidad de cinco minutos ya implementada, definir renovación/reconexión y semántica del panel, verificar recursos y herramientas por HTTP/stdio y conectar un cliente agente real cuando el entorno lo permita. Diagnosticar por separado MCP directo de Aurora, conector público, Aurora Hub y Aurora → Synapse. No sustituir una conexión real por una respuesta pública o una sonda local.

### Propiedad de archivos y coordinación

`src-tauri/src/mcp/protocol.rs`, `server.rs`, `status.rs`, `capabilities.rs`, `mod.rs`, `src/features/mcp/`, `scripts/mcp/verify-foundation.mjs`, `verify-mcp-panel.mjs` y documentación MCP de sesiones/conexión. No editar `peer.rs`, `devices.rs`, receptor LAN de `lib.rs`, `src/features/gallery/` ni launcher/credenciales Android: pertenecen al otro frente.

El frente paralelo es **SYN-110-INTEROP**. Cada agente actualiza únicamente los documentos de su proceso. El PLAN global, ROADMAP y RELEASE_NOTES quedan para integración final; no editarlos simultáneamente. Si hace falta un archivo del otro frente, registrar una solicitud concreta de cambio y coordinar antes de tocarlo. No renombrar contratos compartidos unilateralmente.

Preferir checkouts/worktrees separados si se ejecutan ambos agentes; no compartir directorios de compilación Cargo/Android entre trabajos concurrentes. Si se usa el mismo checkout, respetar la propiedad de archivos y serializar compilaciones que escriban los mismos artefactos. No revertir ni añadir a commits cambios ajenos. No se han creado worktrees en esta entrega.

### Cierre

Completar TASKS y VALIDATION con evidencia por recorrido. Un bloqueo externo se registra como pendiente, no como éxito. Entregar cambios y referencias para integración en SYN-110. El cierre de este frente no cierra el proceso global ni acredita toda la 1.1.0.

## RESTRICCIONES

No abrir el MCP a la LAN ni apuntar un backend remoto al loopback del PC como supuesto enlace. No cambiar modelos/proveedores ni crear credenciales, publicar despliegues o editar configuración global ajena sin autorización aplicable. No implementar descargas, acciones teléfono ↔ PC ni nuevos adaptadores. No imprimir secretos.

No instalar builds en dispositivos, publicar releases, forzar etiquetas o hacer push forzado como consecuencia automática del plan. Preparar y probar lo autorizado; si una prueba exige una instalación no autorizada, dejarla pendiente y explicarla. No atribuir aceptación humana al agente.

