# ⚡ Aurora Synapse v1.0.1 — Instalador modernizado e Integración DaVinci Flow MCP

**Aurora Synapse 1.0.1** moderniza el instalador oficial de Windows con la plantilla nativa de Biglex en Inno Setup e incorpora **DaVinci Flow** al puente universal MCP, permitiendo la automatización de DaVinci Resolve y la descarga organizada de medios mediante Luna Fetch.

---

### Novedades destacadas

- **Integración con DaVinci Flow (MCP):** Expone 7 herramientas nativas sobre stdio para interactuar con DaVinci Resolve (inspección de estado, captura de subtítulos, catálogo de plantillas, inserción de títulos nativos, efectos Fusion animados y subtitulado).
- **Descarga e importación con Luna Fetch:** Permite descargar medios (video o música) desde cualquier enlace e importarlos automáticamente clasificados en el Media Pool de DaVinci Resolve (`Videos` y `Música`).
- **Instalador de Windows modernizado (Inno Setup):** Pantalla única de bienvenida con ruta editable, esquinas nativas de Windows 11, detección de tema claro/oscuro e instalación por usuario sin elevación ni instalaciones paralelas.
- **Build verificado y candado anti-obsolescencia:** El empaquetador comprueba que el ejecutable generado sea estrictamente más reciente que el código fuente y que `dist/`, evitando publicar binarios desactualizados.
- **Coherencia de versión en todo el ecosistema:** Versión 1.0.1 unificada en la interfaz (*Acerca de*), el actualizador automático y el endpoint de estado HTTP (`GET /status`).

---

### Distribución

- **Windows x64:** `Aurora-Synapse-v1.0.1-Setup.exe` (Inno Setup, carpeta `release/`). Instalación por usuario, sin elevación.
- **Android:** `Aurora-Synapse-v1.0.1.apk` (APK autónomo optimizado para arquitectura ARM64).
- **Integridad:** `SHA256SUMS.txt` con las sumas SHA-256 de los artefactos oficiales. Verificar con `sha256sum -c SHA256SUMS.txt`.

> Requiere **WebView2 Runtime** en Windows. Si existía una instalación previa, consultar [`packaging/windows/MIGRATION.md`](packaging/windows/MIGRATION.md).

---

### Soporte

- **Sitio oficial:** https://www.biglexj.com/
- **GitHub:** https://github.com/biglexj
