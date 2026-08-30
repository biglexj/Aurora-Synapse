# 🗺️ Hoja de Ruta — Aurora Synapse

Documento estratégico de evolución, características planificadas y registro de hitos completados de Aurora Synapse.

---

## 🔴 Pendientes activos (v0.2.0)
- [ ] **Servidor HTTP LAN embebido (Puerto 49295)**: Recepción directa de payloads `/synapse` enviados por clientes móviles sin intermediarios.
- [ ] **Descubrimiento UDP Multicast (Puerto 49289)**: Emparejamiento sin configuración (ZeroConf) entre instancias de escritorio y dispositivos Android en la misma red Wi-Fi.
- [ ] **Android Share Sheet Integration (`ReceiveShareActivity`)**: Integrar intent filter en Android para registrar Synapse en el menú nativo de "Compartir".

---

## 🟡 Intermedio (v0.3.0+)
- [ ] **Sincronización Opcional de Portapapeles (Universal Clipboard)**: Toggle en configuración para propagar texto copiado automáticamente entre dispositivos emparejados.
- [ ] **Handoff Multimedia Bidireccional**: Transferir reproducción en caliente entre Prisma en PC y teléfono móvil.
- [ ] **Atajo de Teclado Global (Global Hotkey)**: Invocar la barra de despacho rápido (`Alt + Espacio` o `Ctrl + Shift + S`) desde cualquier parte del sistema operativo.

---

## ⚪ Descartado / En Pausa
- [ ] Soporte para protocolos propietarios cerrados (Descartado: El ecosistema prioriza HTTP/JSON y esquemas URI abiertos).

---

## 🟢 Completado

### v0.1.0 — Lanzamiento Inicial
- [x] **Arquitectura Base Tauri v2 + Rust + React 19**.
- [x] **Clasificador Inteligente de Intención**.
- [x] **Launchpad Unificado con pestañas `Aplicaciones` y `Web`**.
- [x] **Rejilla fluida adaptativa de 2 a 6 columnas**.
- [x] **Persistencia de estado de ventana (Window State Persistence)**.
- [x] **Integración con System Tray, minimización continua y Auto-Run**.
- [x] **Modal de Configuración y enlaces oficiales de soporte**.
