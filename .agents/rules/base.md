---
trigger: always_on
---

# Instrucciones del proyecto

## Documentación Core — entrada compartida

Consultar primero [Documentación Core](../../../Docs/README.md) y seleccionar solo los módulos pertinentes. Los estándares compartidos se mantienen allí; este archivo conserva el alcance y los contratos específicos del proyecto.

- [Calidad y entrega proporcionales](../../../Docs/global/quality/quality-baseline.md): build para ajustes visuales pequeños; pruebas pertinentes para elementos, lógica o páginas nuevos y cambios grandes; revisión acotada para documentación.
- [Procesos según alcance](../../../Docs/global/documentation/process-workflow.md): no añadir fases, archivos ni nuevas aprobaciones por rutina a ajustes pequeños ya autorizados.
- [Diseño común](../../../Docs/global/design/design_system.md): contenido preciso, espacio útil, mínimo scroll innecesario, fondo global y presets compartidos, conservando la identidad del producto.
- [Diagnósticos reutilizables](../../../Docs/troubleshooting/README.md) y [promoción a Core](../../../Docs/global/documentation/core-promotion-workflow.md): documentar en Core los fallos confirmados o aprendizajes generalizables y enlazarlos desde el proyecto; mantener locales los detalles exclusivos.

En estos criterios comunes, aplicar las bases vigentes enlazadas en lugar de copias históricas de este archivo. Respetar la dirección actual de Biglex y los contratos específicos del producto; ante una contradicción, comprobar la implementación vigente y resolverla explícitamente. Las skills generales importadas son apoyo técnico, no una autoridad para redefinir estas reglas.

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
