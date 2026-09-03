# 📋 Historial de Lanzamientos — Aurora Synapse

Todas las versiones oficiales y registros de cambios notables de Aurora Synapse se documentan de manera transparente en este archivo.

---

## ⚡ [1.0.0] — 2026-09-03

### 🚀 Primera Versión Oficial de Producción
- **Instancia Única (Single Instance)**: Integración del estándar de instancia única para evitar múltiples procesos concurrentes en producción, restaurando y enfocando la ventana activa si la aplicación ya se encuentra residente en el sistema.
- **Entorno de Desarrollo y Producción Independientes**: Capacidad de ejecutar sesiones de desarrollo concurrentes con la versión instalada sin colisiones de named pipes ni suplantación de ventanas.
- **Sistema de Auto-Actualizaciones Integrado**:
  - Verificación silenciosa en segundo plano al iniciar la aplicación.
  - Verificación interactiva manual desde Configuración con diálogo modal (80% ancho, máx 480px) y notificación flotante (Toast) de 4 segundos.
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
