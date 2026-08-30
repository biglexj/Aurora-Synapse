# Aurora Synapse — Agent Instructions

## Identity & Mission
**Aurora Synapse** es el orquestador global y puente universal multiplataforma del ecosistema **biglexj**.
Permite la recepción universal de enlaces, medios y archivos desde cualquier dispositivo (Android Share Sheet, navegador web, PC) y su enrutamiento inteligente determinista hacia las aplicaciones satélite del ecosistema (**Gallery-DL GUI**, **Luna Fetch**, **Prisma**, **Ely-Tesia**, **WinTTS**, **LyraFlow**).

## Stack Tecnológico
- **Core Framework**: Tauri v2 + Rust
- **Frontend**: React 19 + TypeScript + Vite + Bun
- **Estilos**: Vanilla CSS con Sistema de Diseño **Material 3 Expressive** (paleta tonal, micro-animaciones, contenedores elevados, responsive grid 2 columnas en móviles / 4 columnas en desktop).
- **Red & Protocolos**:
  - `49295`: Servidor HTTP LAN de Aurora Synapse.
  - `49289`: Beacon UDP para descubrimiento Zero-Config de dispositivos en red local.
  - `49286`: Named Pipe / Socket IPC loopback local (`\\.\pipe\aurora-synapse-ipc`).

## Reglas del Ecosistema
- **Licencia**: GNU GPL v2.0 (`GPL-2.0`)
- **Autor**: biglexj (2026)
- **Documentación Core**: `D:\Proyectos\biglexj\Core-Docs` (Consultar `features/aurora-synapse/` y `global/design/`).
