# 📋 Historial de Lanzamientos — Aurora Synapse

Todas las versiones oficiales y registros de cambios notables de Aurora Synapse se documentan de manera transparente en este archivo.

## ⚡ [1.0.1] — 2026-11-09

### 📦 Instalador de Windows modernizado, puente MCP y DaVinci Flow

- **Integración DaVinci Flow en el Servidor MCP**: Inclusión de DaVinci Flow y DaVinci Resolve como aplicación satélite en el puente universal MCP. Expone 7 herramientas nativas por stdio (`davinciflow_status`, `davinciflow_templates`, `davinciflow_capture`, `davinciflow_fetch_media`, `davinciflow_create_fusion_effect`, `davinciflow_insert_native_title`, `davinciflow_write_subtitles`) con esquemas validados y auditoría en tiempo real.
- **Descarga y catalogación de medios con Luna Fetch en Resolve**: Invocación transparente de Luna Fetch desde DaVinci Flow (`davinciflow_fetch_media`) para descargar pistas de audio o video e importarlas de forma inmediata y clasificada en el Media Pool de DaVinci Resolve (`Videos` y `Música`).

- **Instalador de Windows modernizado (Inno Setup)**: El instalador EXE de Windows se genera ahora con la plantilla propia de Biglex en Inno Setup, con una pantalla única de entrada (presentación, ruta editable y botón principal), progreso y éxito nativos, tema claro/oscuro automático según Windows, esquinas nativas de Windows 11 e ícono centrado. La instalación es por usuario y sin elevación, la actualización se aplica en el mismo directorio sin instalar en paralelo, y la coexistencia con el instalador anterior queda documentada en `packaging/windows/MIGRATION.md`.
- **Artefacto**: `Aurora-Synapse-v1.0.1-Setup.exe` (`release/`), generado desde el binario de la propia versión 1.0.1, no desde un binario de una carrera anterior.
- **Candado anti-build-obsoleto**: El empaquetador ahora **falla** si el ejecutable es más viejo que el código fuente o que `dist/`, en lugar de empaquetar en silencio un binario viejo. Motivo: Tauri incrusta `dist/` dentro del ejecutable, así que un `.exe` atrasado publicaba también un frontend atrasado aunque `dist/` estuviera al día. Conmutable con `-ForceStale` solo para depurar.
- **Sin rutas absolutas**: El empaquetador localiza `ISCC.exe` subiendo desde su propia ubicación hasta la copia del workspace y, en su defecto, en el `PATH`, conforme al estándar global de scripts (principio 4).
- **Versión coherente en toda la app**: *Acerca de*, el actualizador y el endpoint informativo `GET /status` (`:49295`) muestran ya la versión `1.0.1` real en lugar de la `1.0.0` heredada. La versión del endpoint y el *User-Agent* de la consulta de actualizaciones se derivan ahora del propio crate, de modo que no pueden volver a divergir. Los metadatos incrustados en el ejecutable (CompanyName, LegalCopyright) se alinean con los del instalador (`Biglex`, © 2026 Biglex J).
- **Sumas de verificación acotadas a lo publicado**: `SHA256SUMS.txt` se genera solo con los artefactos que se suben a la release, no con los ficheros históricos que permanecen en `release/`. Antes podía incluir entradas de versiones anteriores cuyos archivos no existían en la release publicada. Se escribe sin BOM para que `sha256sum -c` funcione también en Linux y macOS.
- **Un único instalador oficial**: Tauri deja de generar un instalador NSIS propio (`bundle.targets` vacío); el artefacto oficial es el de Inno Setup. Así no pueden coexistir dos instaladores distintos con el mismo nombre de app.
- **Icono del launcher de Android corregido**: el icono de la app se generaba aparte y **no usaba** el artwork de la carpeta de fuentes, así que el móvil seguía mostrando el diseño anterior. Ahora `bun run sync:icons` genera el set de Tauri y el launcher de Android desde `Aurora-Synapse-transparent.webp`, con el artwork dentro de la zona segura de 66dp del icono adaptativo. Antes los brazos del emblema se cortaban en cualquier launcher.
- **Servidor MCP**: Se incluye el servidor MCP de Synapse, cableado en el backend de Tauri con 27 herramientas en total y doble transporte (HTTP loopback y stdio).

---

## ⚡ [1.0.0] — 2026-09-10

### 🚀 Primera Versión Oficial de Producción
- **Integración Oficial de Pixi Store**: Reconocimiento automático y acceso directo al portal oficial de temas interactivos y presets (`https://pixi.qzz.io/`) desde el catálogo y clasificador de intenciones.
- **Lanzamiento Nativo Directo en Android**: Integración del plugin nativo Android para apertura limpia sin parámetros de **Luna Fetch**, **Super Galería** y **Ely-Tesia** vía Package Intents y enlace universal, eliminando errores de ActivityNotFound.
- **Despacho Universal Móvil ➔ PC LAN**: Botón inteligente `🖥️ Enviar a PC` en aplicaciones exclusivas de escritorio (como **Gallery-DL GUI**) que enruta descargas de forma determinista hacia la computadora por red local (`:18274` y `:49295`).
- **Servidor HTTP LAN Embebido (Puerto 49295)**: Listener asíncrono en Tokio con soporte CORS y endpoints `/status` y `/dispatch` para interconexión universal con clientes móviles.
- **Diferenciación y Badges de S.O.**: Clasificación visual en tarjetas (`📱 ANDROID`, `🖥️ PC SOLO`, `📱/🖥️ HÍBRIDO`, `🌐 WEB`) y pestañas de filtrado por Sistema Operativo.
- **Configuración LAN con Test de Conexión**: Selector de dirección IP de la PC en Ajustes con persistencia local y botón interactivo para probar conectividad en tiempo real.
- **Instancia Única (Single Instance)**: Integración del estándar de instancia única para evitar múltiples procesos concurrentes en producción, restaurando y enfocando la ventana activa si la aplicación ya se encuentra residente en el sistema.
- **Entorno de Desarrollo y Producción Independientes**: Capacidad de ejecutar sesiones de desarrollo concurrentes con la versión instalada sin colisiones de named pipes ni suplantación de ventanas.
- **Sistema de Auto-Actualizaciones Integrado**:
  - Verificación silenciosa en segundo plano al iniciar la aplicación.
  - Verificación interactiva manual desde Configuración con diálogo modal y notificación flotante (Toast) de 4 segundos.
  - Sanitización canónica de notas de versión en Markdown para lectura limpia en la interfaz de usuario.
- **Actualización de Marca e Iconografía**:
  - Sustitución del imagotipo transparente en el encabezado por el icono oficial con fondo unificado.
  - Actualización del icono de DaVinci Flow en el catálogo con su identidad moderna de timeline y ondas de audio.
  - Personalización del icono del instalador NSIS para Windows (`installerIcon`).
- **Resolución de Errores Críticos**:
  - Solución del error de conexión/404 al arrancar con Windows mediante el registro directo del binario compilado en `%LOCALAPPDATA%` con el argumento `--autostart`.
  - Eliminación de sombras oscuras discordantes en la tarjeta de despacho sobre los chips de categorías.

---

## ⚡ [0.1.0] — 2026-08-30

### 🚀 Novedades y Arquitectura Base
- **Lanzamiento Inicial del Orquestador Universal**: Implementación de la arquitectura central de enrutamiento y Launchpad multiplataforma construida con Tauri v2, Rust y React 19.
- **Clasificador Inteligente de Intención**: Análisis en tiempo real de enlaces web, plataformas multimedia y extensiones de archivo locales con recomendación automática de la aplicación satélite receptora.
- **Launchpad Unificado del Ecosistema**:
  - Pestaña **`Aplicaciones`**: Catálogo completo de aplicaciones nativas de escritorio y móviles (*Gallery-DL GUI*, *Luna Fetch*, *Prisma*, *Ely-Tesia*, *WinTTS*, *LyraFlow*, *DaVinci Flow*, *Super Galería*, *codex-go*, *MouziFlow* y *Pixi Store*).
  - Pestaña **`Web`**: Suite de utilidades y herramientas de desarrollo web de `biglexj.com` (*Tweet UI Clone*, *Color Gen*, *Calc. Aduana*, *Panel de Creador*, *Chat Global*, *Wallpapers*, *Buzón Feedback*, *Música*, *Instrumentales*, *Karaokes*, *Noticias* y *En Vivo*).
- **Diseño Adaptativo Material 3 Expressive**: Rejilla fluida que aprovecha el 100% del ancho de pantalla con escalado dinámico de 2 a 6 columnas y badges tonales (*NUEVA*, *ACTUALIZADA*, *EXPERIMENTAL*, *PROXIMAMENTE*, *DISPONIBLE*).
- **Integración con System Tray & Segundo Plano**: Icono interactivo en la bandeja del sistema con menú contextual para restauración rápida, acceso a configuración y salida limpia.
- **Auto-Run & Window State Persistence**: Soporte para autoinicio con Windows e intercepción de cierre para minimización continua en segundo plano sin destruir la sesión activa.
