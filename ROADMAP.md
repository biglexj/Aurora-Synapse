# 🗺️ Hoja de Ruta — Aurora Synapse

Documento estratégico de evolución, características planificadas y registro de hitos completados de Aurora Synapse.

## Prioridad actual — puente universal MCP e interconectividad

Coordinación del 2026-10-27: [SYN-110-INTEROP](process/active/2026-10-27_interconectividad-telefono-pc/PLAN.md)
y [SYN-110-MCP](process/active/2026-10-27_sesiones-y-conexion-mcp/PLAN.md) preparados
para agentes en paralelo bajo SYN-110. Se reactiva el frente teléfono ↔ PC;
descargas y nuevas integraciones permanecen aplazadas. [Índice de procesos](process/README.md).

Dirección definida por Biglex el 2026-10-26: Synapse permite a las aplicaciones
y a los agentes (incluidos Codex y Ely Intelligence) descubrir capacidades,
ejecutar acciones y comprobar resultados en las apps del ecosistema, también
entre teléfono y PC. [Primer proceso formal: alcance, etapas y cierre](process/active/2026-10-26_puente-universal-mcp-e-interconectividad/PLAN.md).

El proceso amplía el alcance del [plan del 23 de octubre](docs/mcp/PLAN.md),
conservado como antecedente. Prioriza inventario y contratos, base fiable,
descargas con Luna Fetch y Gallery-DL GUI, recorridos bidireccionales y extensión
por capacidades reales.

**Hito objetivo 1.1.0 — alcance acotado posteriormente por Biglex:** implementar
la base común y probar conexión del agente con el MCP de Aurora. Descargas,
interacción teléfono ↔ PC y ampliación de integraciones quedan aplazadas.
La versión actual del proyecto sigue siendo 1.0.1; no se publica en esta etapa.
Contrato y uso: [base común MCP](docs/mcp/FOUNDATION.md).

- [x] Disponibilidad real, validación de argumentos, sonda PowerShell y escalado
  mediante CLI con salida nueva comprobada localmente.
- [ ] Cerrar los recorridos reales restantes de Luna Fetch, reproducción/control
  de Prisma y conversación con Ely Intelligence. Ver [evidencia y pendientes](docs/mcp/VERIFICATION.md).
- [ ] Incorporar Gallery-DL GUI al catálogo MCP con descarga y resultado verificables.
- [ ] Comprobar operación desde Codex y Ely Intelligence y acciones teléfono → PC / PC → teléfono.
- [ ] Conservar transferencias de Super Galería y completar la matriz de capacidades del ecosistema.

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
 
### v1.0.1 — Instalador de Windows modernizado (2026-10-26)
- [x] **Instalador oficial en Inno Setup** con la plantilla compartida de Biglex: pantalla única de entrada, progreso y éxito nativos, tema claro/oscuro automático y esquinas nativas de Windows 11.
- [x] **Instalación por usuario sin elevación**, aplicada en el mismo directorio para evitar instalaciones paralelas de la misma app.
- [x] **Candado anti-build-obsoleto** en el empaquetador: falla si el ejecutable es más viejo que el código fuente o que `dist/`.
- [x] **Coherencia de versión** en `package.json`, `tauri.conf.json`, el crate de Rust, *Acerca de*, el actualizador y el endpoint `GET /status`.
- [x] **Un único instalador oficial**: Tauri deja de emitir instalador NSIS propio.
- [x] **Sumas SHA-256 acotadas** a los artefactos efectivamente publicados.

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
