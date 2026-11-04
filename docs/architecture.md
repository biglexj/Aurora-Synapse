# Arquitectura local de Aurora Synapse

Alcance: describe **la implementación de este repositorio**. El diseño del protocolo, los contratos de payload, el registro de puertos y las reglas de versión viven en Core y se enlazan desde aquí; no se replican.

- Protocolo y arquitectura canónicas: [Aurora Synapse en Core](../../Docs/features/aurora-synapse/README.md).
- Puerto y red canónicos: [registro de puertos y red](../../Docs/features/aurora-synapse/port-and-network-registry.md).
- Comandos: [estándar Mise](../../Docs/global/architecture/mise-command-standard.md) · [scripts y alias](../../Docs/global/architecture/command-scripts-standard.md).

## Identidad

**Aurora Synapse** es el orquestador global y el puente universal multiplataforma del ecosistema **biglexj**. Recibe contenido desde el *Share Sheet* de Android, la barra de despacho de escritorio, enlaces y archivos, y lo enruta de forma determinista hacia las aplicaciones satélite (Gallery-DL GUI, Luna Fetch, Prisma, Ely-Tesia, WinTTS, LyraFlow, Super Galería).

## Stack

| Capa | Tecnología |
|---|---|
| Contenedor | Tauri v2 + Rust (`src-tauri/`) |
| Interfaz | React 19 + TypeScript + Vite, empaquetado con Bun |
| Estilos | CSS propio (`src/App.css`) con paleta tonal, micro-animaciones y contenedores elevados, dentro del [sistema de diseño de Core](../../Docs/global/design/design_system.md) |
| Puente MCP | Módulo Rust propio (`src-tauri/src/mcp/`) con transporte HTTP loopback y stdio |

## Mapa del repositorio

| Ruta | Contenido |
|---|---|
| `src/App.tsx`, `src/App.css` | Interfaz principal: catálogo de aplicaciones, panel de conexiones y rejilla adaptativa. |
| `src/features/mcp/` | Panel de estado del puente MCP y cliente de la interfaz. |
| `src/features/gallery/` | Ajustes de conexión y emparejamiento por QR con Super Galería. |
| `src/features/updater/` | Servicio de actualización y diálogo de actualización. |
| `src/shared/ecosystem.json` | Catálogo del ecosistema que la interfaz consume. |
| `src-tauri/src/mcp/` | Servidor MCP: protocolo, herramientas, despacho, validación, estado y credenciales de galería. |
| `src-tauri/src/peer.rs` | Pares del ecosistema para el descubrimiento y la conexión entre aplicaciones. |
| `packaging/windows/` | Instalador Inno Setup y su migración. |
| `scripts/` | Implementación de comandos: `mise-task.ts`, `build/`, `packaging/`, `mcp/`, `sync-icons.ps1`. |
| `docs/mcp/` | Documentación funcional del puente: base, plan, sesiones, contrato de interfaz y verificación. |
| `process/` | Procesos del proyecto (`SYN-110*`). |

## Puertos y servicios de esta implementación

| Puerto | Uso | Nota |
|---|---|---|
| `49295` | Servidor HTTP LAN de Synapse (`POST /paired/dispatch`, `GET /status`) | Escucha en `0.0.0.0`. Las rutas heredadas `/synapse` y `/dispatch` siguen funcionando con credencial, pero responden con el aviso de obsolescencia. |
| `49289` | Beacon UDP de descubrimiento Zero-Config | Difusión compartida por diseño; el beacon anuncia el puerto HTTP del emisor. |
| `49296` | Puente MCP | Solo loopback (`127.0.0.1:49296`); configurable con `AURORA_SYNAPSE_MCP_ADDR`. |
| `49286` | Canal IPC propio | **Reservado y no implementado** en este repositorio: no hay named pipe propia. |

Otros puertos (`49288` Luna Fetch, `49290` Prisma, `49291` Ely-Tesia, `49292` LyraFlow, `49285` WinTTS) son de otras aplicaciones del ecosistema y se consultan, no se sirven aquí.

## Puente MCP

- Transporte HTTP: `http://127.0.0.1:49296/mcp`, o por línea de comandos con `aurora-synapse.exe --mcp-http`.
- Transporte stdio: `aurora-synapse.exe --mcp-stdio`.
- Recurso de capacidades: `aurora-synapse://capabilities`, derivado del registro ejecutable de herramientas.
- Variable de entorno del consumidor: `AURORA_SYNAPSE_MCP_URL`; para escalado con GPU, `AURORA_SYNAPSE_TIMEOUT_MS=150000`.
- Detalle completo del contrato: [docs/mcp/FOUNDATION.md](mcp/FOUNDATION.md) · [docs/mcp/SESIONES.md](mcp/SESIONES.md) · [docs/mcp/UI_CONTRACT.md](mcp/UI_CONTRACT.md).

## Decisiones de producto locales

- **Rejilla de aplicaciones**: 2 columnas por defecto; 3 desde 680 px, 4 desde 960 px, 5 desde 1280 px y 6 desde 1560 px.
- **Residencia en segundo plano**: cerrar la ventana minimiza a la bandeja del sistema para mantener la escucha reactiva.
- **Persistencia de ventana**: tamaño, posición y estado maximizado mediante `tauri-plugin-window-state`.
- **Instancia única**: la segunda instancia entrega sus argumentos a la instancia viva (`tauri-plugin-single-instance`).
- **Actualizaciones**: el diálogo de actualización y su servicio viven en `src/features/updater/`; el flujo de publicación es el del ecosistema.

## Operación

Los comandos de uso son los de Mise; el detalle está en el [README](../README.md).

| Tarea | Efecto |
|---|---|
| `mise dev` | Ejecuta la aplicación en desarrollo (`tauri dev`). |
| `mise build` | Compila frontend y binario de escritorio (`--no-bundle`), sin instalador. |
| `mise check` | Verificación estática de tipos (TypeScript) y del código Rust, sin emitir artefactos de distribución. |
| `mise test` | Pruebas unitarias del módulo Rust del puente MCP. |
| `mise release` | Prepara el instalador Inno Setup, el APK y `SHA256SUMS.txt` en `release/`; no publica. |
| `mise publish` | Publicación explícita: requiere autorización. `--plan` y `--check` son de solo lectura. |

`release` y `publish` nunca se encadenan: `release` no modifica el remoto, y `publish` no se ejecuta como efecto secundario de `build` ni de `release`.
