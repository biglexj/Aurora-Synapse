# 🗺️ Hoja de Ruta — Aurora Synapse

Documento estratégico de evolución, características planificadas y registro de hitos completados de Aurora Synapse.

---

## 🔴 Pendientes activos (v1.1.0)
- [ ] **Descubrimiento UDP Multicast (Puerto 49289)**: Emparejamiento sin configuración (ZeroConf) entre instancias de escritorio y dispositivos Android en la misma red Wi-Fi.
- [ ] **Android Share Sheet Integration (`ReceiveShareActivity`)**: Integrar intent filter en Android para registrar Synapse en el menú nativo de "Compartir".

---

## 🟡 Intermedio (v1.2.0+)
- [ ] **Sincronización Opcional de Portapapeles (Universal Clipboard)**: Toggle en configuración para propagar texto copiado automáticamente entre dispositivos emparejados.
- [ ] **Handoff Multimedia Bidireccional**: Transferir reproducción en caliente entre Prisma en PC y teléfono móvil.
- [ ] **Atajo de Teclado Global (Global Hotkey)**: Invocar la barra de despacho rápido (`Alt + Espacio` o `Ctrl + Shift + S`) desde cualquier parte del sistema operativo.

---

## ⚪ Descartado / En Pausa
- [ ] Soporte para protocolos propietarios cerrados (Descartado: El ecosistema prioriza HTTP/JSON y esquemas URI abiertos).

---

## 🟢 Completado
 
### v1.0.0 — Primera Edición Oficial de Producción (2026-09-10)
- [x] **Consolidación de la Versión 1.0.0 Oficial Multiplataforma**.
- [x] **Integración Oficial de Pixi Store (`https://pixi.qzz.io/`)**.
- [x] **Lanzamiento Nativo Directo en Android (Luna Fetch, Super Galería, Ely-Tesia)**.
- [x] **Despacho Universal Móvil ➔ PC LAN (Gallery-DL GUI, Luna Fetch)**.
- [x] **Servidor HTTP LAN Embebido (Puerto 49295)**.
- [x] **Diferenciación y Badges de S.O. con Filtro Segmentado**.
- [x] **Configuración LAN con Test de Conexión en Tiempo Real**.
- [x] **Bloqueo Inteligente de Instancia Única (Single Instance)**.
- [x] **Coexistencia Limpia de Instancias Dev y Release**.
- [x] **Sistema Canónico de Auto-Actualización con GitHub Releases**.
- [x] **Actualización de Icono de Marca (fondo oficial) e icono DaVinci Flow**.
- [x] **Distribución Unificada: Instalador NSIS .exe y APK Universal**.
- [x] **Corrección de Autostart y eliminación de error 404 en Windows**.

### v0.1.0 — Lanzamiento Inicial
- [x] **Arquitectura Base Tauri v2 + Rust + React 19**.
- [x] **Clasificador Inteligente de Intención**.
- [x] **Launchpad Unificado con pestañas `Aplicaciones` y `Web`**.
- [x] **Rejilla fluida adaptativa de 2 a 6 columnas**.
- [x] **Persistencia de estado de ventana (Window State Persistence)**.
- [x] **Integración con System Tray, minimización continua y Auto-Run**.
- [x] **Modal de Configuración y enlaces oficiales de soporte**.
