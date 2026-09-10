---
trigger: always_on
---

# Instrucciones del proyecto

Entrada única de instrucciones: `.agents/rules/base.md`.

Consultar [Docs](../../../Docs/README.md) como fuente compartida. Seleccionar únicamente estándares, perfiles y skills pertinentes al encargo. Conservar las reglas específicas de este proyecto; su código y configuración vigentes determinan el comportamiento real.

# Agent Instructions - Aurora Synapse

## AI Models (CRITICAL)
Always use the next-generation models defined in the platform. Do NOT use legacy models like Gemini 1.5 or old GPT versions unless explicitly requested for legacy testing.

**Current Recommended Models (2026):**
- `gemini-3.5-flash` (Default for general chat/intelligence / Smart)
- `gemini-3.1-flash-lite` (Fast responses / G-3.1 Flash)
- `gemini-3.1-pro-preview` (Deep reasoning / Complex tasks / G-3.1 Pro)

## Project License & Author
- **License**: GNU GPL v2.0 (`GPL-2.0`)
- **Author**: biglexj (2026)

## Estructura & Lenguaje de Diseño
- **Material 3 Expressive**: Toda la interfaz debe utilizar el lenguaje visual moderno con transiciones fluidas, elevaciones tonales y soporte modo oscuro nativo.
- **Grid de Aplicaciones**: 2 columnas en pantallas móviles / smartphones (`< 640px`) y 4 columnas en pantallas de escritorio / PC (`>= 640px`).
- **Tono Científico y Elegante**: Expresarse con estructura y elegancia científica (*Dr. Stone* / Dr. Xeno).

## Reglas locales integradas desde .agents/rules/base.md

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
- **Documentación Core**: `D:\Proyectos\biglexj\Docs` (Consultar `features/aurora-synapse/` y `global/design/`).
