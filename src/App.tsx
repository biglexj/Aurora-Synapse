import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";
import { UpdateModalDialog } from "./features/updater/UpdateModalDialog";
import { Toast } from "./features/updater/Toast";
import { checkAppUpdates, APP_CURRENT_VERSION } from "./features/updater/updaterService";
import { UpdateCheckResult } from "./features/updater/types";

interface AppTarget {
  id: string;
  name: string;
  category: string;
  description: string;
  icon_path: string;
  icon_emoji?: string;
  accent_color: string;
  uri_scheme: string;
  web_url?: string;
  port: number;
  supported_domains: string[];
  is_web_app: boolean;
  status: "NUEVA" | "ACTUALIZADA" | "BETA" | "EXPERIMENTAL" | "PROXIMAMENTE" | "DISPONIBLE" | string;
  platforms: string[];
}

interface DeviceNode {
  id: string;
  name: string;
  device_type: string;
  ip: string;
  is_local: boolean;
  os: string;
}

interface ClassificationResult {
  recommended_app_id: string;
  reason: string;
  detected_type: string;
}

interface SynapseSettings {
  autostart: boolean;
  minimize_to_tray: boolean;
  lan_discovery: boolean;
}

export default function App() {
  const [apps, setApps] = useState<AppTarget[]>([]);
  const [devices, setDevices] = useState<DeviceNode[]>([]);
  const [selectedDevice, setSelectedDevice] = useState<string>("local_pc");
  const [selectedPlatform, setSelectedPlatform] = useState<string>("windows");
  const [inputText, setInputText] = useState<string>("");
  const [searchQuery, setSearchQuery] = useState<string>("");
  const [selectedCategory, setSelectedCategory] = useState<string>("Aplicaciones");
  const [classification, setClassification] = useState<ClassificationResult | null>(null);
  const [statusMessage, setStatusMessage] = useState<{ text: string; type: "success" | "error" | "info" } | null>(null);
  const [isDispatching, setIsDispatching] = useState<boolean>(false);
  const [failedIcons, setFailedIcons] = useState<Record<string, boolean>>({});

  // Mobile & Theme State
  type ThemeMode = "system" | "dark" | "light";
  const [isMobile, setIsMobile] = useState<boolean>(false);
  const [theme, setTheme] = useState<ThemeMode>(() => {
    return (localStorage.getItem("synapse_theme") as ThemeMode) || "system";
  });
  const [androidBgResident, setAndroidBgResident] = useState<boolean>(true);
  const [androidAutoBoot, setAndroidAutoBoot] = useState<boolean>(false);

  // Settings Modal
  const [isSettingsOpen, setIsSettingsOpen] = useState<boolean>(false);
  const [settings, setSettings] = useState<SynapseSettings>({
    autostart: false,
    minimize_to_tray: true,
    lan_discovery: true,
  });

  // LAN PC IP State (Configuración de destino para móvil -> PC)
  const [pcIp, setPcIp] = useState<string>(() => {
    return localStorage.getItem("synapse_pc_ip") || "192.168.1.121";
  });
  const [pingStatus, setPingStatus] = useState<"idle" | "testing" | "ok" | "fail">("idle");

  const handlePcIpChange = (ip: string) => {
    setPcIp(ip);
    localStorage.setItem("synapse_pc_ip", ip);
  };

  const testPcConnection = async () => {
    setPingStatus("testing");
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 3000);
      const res = await fetch(`http://${pcIp}:49295/status`, {
        signal: controller.signal,
        headers: { Accept: "application/json" },
      });
      clearTimeout(timeoutId);
      if (res.ok) {
        setPingStatus("ok");
        setToastMessage(`✅ PC Synapse Online en http://${pcIp}:49295`);
        setToastType("success");
      } else {
        setPingStatus("fail");
        setToastMessage(`⚠️ Respuesta anormal (${res.status}) desde ${pcIp}:49295`);
        setToastType("warning");
      }
    } catch {
      setPingStatus("fail");
      setToastMessage(`❌ Sin conexión con ${pcIp}:49295. Verifica Wi-Fi y firewall.`);
      setToastType("warning");
    }
  };

  // Auto-updater State (Core-Docs Standard)
  const [pendingUpdate, setPendingUpdate] = useState<UpdateCheckResult | null>(null);
  const [showUpdateModal, setShowUpdateModal] = useState<boolean>(false);
  const [isCheckingUpdates, setIsCheckingUpdates] = useState<boolean>(false);
  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const [toastType, setToastType] = useState<"success" | "warning" | "info">("success");

  // Silent update check on mount (Core-Docs Req #1)
  useEffect(() => {
    checkAppUpdates()
      .then((res) => {
        if (res.available) {
          setPendingUpdate(res);
        }
      })
      .catch(() => {
        // Silencioso al inicio en caso de fallas de red
      });
  }, []);

  const handleManualUpdateCheck = async () => {
    setIsCheckingUpdates(true);
    try {
      const res = await checkAppUpdates();
      if (res.available) {
        setIsSettingsOpen(false); // Cierre síncrono del modal secundario (Core-Docs Req #3)
        setPendingUpdate(res);
        setShowUpdateModal(true);
      } else {
        setToastMessage("¡Estás en la última versión de Aurora Synapse!");
        setToastType("success");
      }
    } catch {
      setToastMessage("No se pudo comprobar la actualización. Revisa tu conexión a internet.");
      setToastType("warning");
    } finally {
      setIsCheckingUpdates(false);
    }
  };

  useEffect(() => {
    const applyTheme = () => {
      let effectiveTheme: "dark" | "light" = "dark";
      if (theme === "system") {
        const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
        effectiveTheme = prefersDark ? "dark" : "light";
      } else {
        effectiveTheme = theme;
      }
      document.documentElement.setAttribute("data-theme", effectiveTheme);
      localStorage.setItem("synapse_theme", theme);
    };

    applyTheme();

    if (theme === "system") {
      const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      const handler = () => applyTheme();
      mediaQuery.addEventListener("change", handler);
      return () => mediaQuery.removeEventListener("change", handler);
    }
  }, [theme]);

  useEffect(() => {
    loadInitialData();

    // Listen for tray settings event
    const unlisten = listen("synapse://open-settings", () => {
      setIsSettingsOpen(true);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const loadInitialData = async () => {
    try {
      const mobileCheck = await invoke<boolean>("is_mobile_platform").catch(() => /Android|iPhone|iPad/i.test(navigator.userAgent));
      const isMob = Boolean(mobileCheck) || /Android|iPhone|iPad/i.test(navigator.userAgent);
      const registeredApps = await invoke<AppTarget[]>("get_registered_apps");
      const pairedDevices = await invoke<DeviceNode[]>("get_paired_devices", { isMobile: isMob });
      const currentSettings = await invoke<SynapseSettings>("get_synapse_settings");
      
      setApps(registeredApps);
      setDevices(pairedDevices);
      setSettings(currentSettings);
      setIsMobile(isMob);

      // Default platform selection based on active OS
      if (isMob) {
        setSelectedPlatform("android");
        setSelectedDevice("android_phone");
      } else {
        setSelectedPlatform("all");
        setSelectedDevice("local_pc");
      }
    } catch (err) {
      console.error("Error al cargar datos iniciales:", err);
    }
  };

  useEffect(() => {
    if (!inputText.trim()) {
      setClassification(null);
      return;
    }
    const timer = setTimeout(async () => {
      try {
        const result = await invoke<ClassificationResult>("classify_intent", { input: inputText });
        setClassification(result);
      } catch (err) {
        console.error("Error clasificando intención:", err);
      }
    }, 150);

    return () => clearTimeout(timer);
  }, [inputText]);

  const handleDeviceChange = (devId: string) => {
    setSelectedDevice(devId);
    const dev = devices.find((d) => d.id === devId);
    if (dev?.os) {
      setSelectedPlatform(dev.os);
    }
  };

  const handleDispatch = async (targetAppId?: string, overrideDeviceId?: string) => {
    const appId = targetAppId || classification?.recommended_app_id;
    if (!appId) return;

    const targetApp = apps.find((a) => a.id === appId);
    if (targetApp?.status === "PROXIMAMENTE") {
      setStatusMessage({ text: `${targetApp.name} estará disponible próximamente en el ecosistema.`, type: "info" });
      return;
    }

    // Si es móvil y la app es exclusiva de PC (no soporta Android ni es web), enrutar a la PC central
    let devId = overrideDeviceId || selectedDevice;
    if (isMobile && targetApp && !targetApp.platforms.includes("android") && !targetApp.is_web_app) {
      devId = "local_pc";
    }

    setIsDispatching(true);
    setStatusMessage({
      text: devId === "local_pc" && isMobile
        ? `Enviando a ${targetApp?.name} en la PC (${pcIp})...`
        : `Despachando a través de Aurora Synapse...`,
      type: "info",
    });

    try {
      const res = await invoke<string>("dispatch_content", {
        appId,
        content: inputText,
        targetDeviceId: devId,
        pcIp: pcIp,
      });
      setStatusMessage({ text: res, type: "success" });
      setInputText("");
    } catch (err) {
      setStatusMessage({ text: String(err), type: "error" });
    } finally {
      setIsDispatching(false);
      setTimeout(() => setStatusMessage(null), 5000);
    }
  };

  const handleToggleAutostart = async () => {
    try {
      const nextState = !settings.autostart;
      await invoke("set_autostart_setting", { enable: nextState });
      setSettings((prev) => ({ ...prev, autostart: nextState }));
    } catch (err) {
      console.error("Error al cambiar autostart:", err);
    }
  };

  const handleToggleMinimizeToTray = async () => {
    try {
      const nextState = !settings.minimize_to_tray;
      await invoke("set_minimize_to_tray_setting", { enable: nextState });
      setSettings((prev) => ({ ...prev, minimize_to_tray: nextState }));
    } catch (err) {
      console.error("Error al cambiar minimizar a bandeja:", err);
    }
  };

  const categories = useMemo(() => {
    return ["Aplicaciones", "Web", "Multimedia", "IA", "Desarrollo", "Utilidades", "Experimentos"];
  }, []);

  const platformsList = useMemo(() => {
    return [
      { id: "all", label: "Todos los S.O.", icon: "🌌" },
      { id: "android", label: "Android (Móvil)", icon: "📱" },
      { id: "windows", label: "Windows (PC)", icon: "🖥️" },
      { id: "web", label: "Web Apps", icon: "🌐" },
      { id: "linux", label: "Linux", icon: "🐧" },
    ];
  }, []);

  const filteredApps = useMemo(() => {
    return apps.filter((app) => {
      const matchesSearch =
        app.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        app.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
        app.category.toLowerCase().includes(searchQuery.toLowerCase());

      let matchesCat = false;
      if (selectedCategory === "Aplicaciones") {
        matchesCat = !app.is_web_app;
      } else if (selectedCategory === "Web") {
        matchesCat = app.is_web_app;
      } else {
        matchesCat = app.category === selectedCategory;
      }

      let matchesPlatform = true;
      if (selectedPlatform === "all") {
        matchesPlatform = true;
      } else if (selectedPlatform === "web") {
        matchesPlatform = app.is_web_app || (app.platforms && app.platforms.includes("web"));
      } else if (selectedPlatform === "android") {
        matchesPlatform = app.platforms && app.platforms.includes("android");
      } else if (selectedPlatform === "windows") {
        matchesPlatform = app.platforms && app.platforms.includes("windows");
      } else if (selectedPlatform === "linux") {
        matchesPlatform = app.platforms && app.platforms.includes("linux");
      }

      return matchesSearch && matchesCat && matchesPlatform;
    });
  }, [apps, searchQuery, selectedCategory, selectedPlatform]);

  const recommendedApp = apps.find((a) => a.id === classification?.recommended_app_id);

  const getStatusBadgeClass = (status: string) => {
    switch (status) {
      case "NUEVA":
        return "badge-nueva";
      case "ACTUALIZADA":
        return "badge-actualizada";
      case "BETA":
        return "badge-beta";
      case "EXPERIMENTAL":
        return "badge-experimental";
      case "PROXIMAMENTE":
        return "badge-proximamente";
      case "DISPONIBLE":
        return "badge-disponible";
      default:
        return "badge-default";
    }
  };

  const getCardActionText = (app: AppTarget) => {
    if (app.status === "PROXIMAMENTE") return "Próximamente";
    const hasInput = Boolean(inputText.trim());
    const isAndroidNative = app.platforms?.includes("android");
    const isPcOnly = !isAndroidNative && !app.is_web_app;

    if (isMobile) {
      if (app.is_web_app) {
        return hasInput ? "🌐 Despachar Web" : "🌐 Abrir Web";
      }
      if (isPcOnly) {
        return hasInput ? "🖥️ Enviar a PC" : "🖥️ Abrir en PC";
      }
      return hasInput ? "📱 Despachar" : "📱 Abrir App";
    } else {
      if (selectedDevice === "android_phone") {
        if (isAndroidNative) {
          return hasInput ? "📱 Enviar a Móvil" : "📱 Abrir en Móvil";
        }
      }
      if (app.is_web_app) {
        return hasInput ? "🌐 Abrir Portal" : "🌐 Visitar";
      }
      return hasInput ? "🚀 Enviar" : "⚡ Lanzar";
    }
  };

  return (
    <div className="synapse-container">
      {/* Header */}
      <header className="synapse-header">
        <div className="header-brand">
          <div className="brand-logo">
            <img
              src="/assets/branding/icons/icon.webp"
              alt="Aurora Synapse"
              className="brand-logo-img"
              onError={(e) => {
                const target = e.currentTarget;
                if (!target.src.endsWith(".png")) {
                  target.src = "/assets/branding/icons/icon.png";
                  return;
                }
                target.style.display = "none";
                if (target.parentElement) {
                  target.parentElement.innerText = "⚡";
                }
              }}
            />
          </div>
          <div>
            <span className="launchpad-tag">LAUNCHPAD & ROUTER</span>
            <h1 className="brand-title">TODAS LAS APLICACIONES</h1>
            <p className="brand-subtitle">Orquestador Universal del Ecosistema Aurora</p>
          </div>
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          {pendingUpdate && (
            <button
              className="header-update-badge"
              title="Nueva versión disponible"
              onClick={() => setShowUpdateModal(true)}
            >
              🚀 v{pendingUpdate.latest_version} disponible
            </button>
          )}

          <button
            className="settings-trigger-btn"
            title="Configuración de Aurora Synapse"
            onClick={() => setIsSettingsOpen(true)}
          >
            ⚙️
          </button>
        </div>
      </header>

      {/* Target Device Switcher (Full Width) */}
      <section className="device-switcher-section">
        <div className="device-switcher">
          <span className="device-label">Dispositivo Destino:</span>
          <div className="device-pills">
            {devices.map((dev) => (
              <button
                key={dev.id}
                className={`device-pill ${selectedDevice === dev.id ? "active" : ""}`}
                onClick={() => handleDeviceChange(dev.id)}
              >
                <span>{dev.device_type === "mobile" ? "📱" : "💻"}</span>
                <span>{dev.name}</span>
              </button>
            ))}
          </div>
        </div>
      </section>

      {/* Smart Input & Dispatch Area */}
      <section className="dispatch-card">
        <div className="input-group">
          <input
            type="text"
            className="synapse-input"
            placeholder="Pega un enlace (DeviantArt, YouTube, Pixiv), archivo local o texto para enrutar..."
            value={inputText}
            onChange={(e) => setInputText(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleDispatch();
            }}
          />
          <button
            className="dispatch-btn"
            disabled={!inputText.trim() || isDispatching}
            onClick={() => handleDispatch()}
          >
            {isDispatching ? "Enviando..." : "Despachar 🚀"}
          </button>
        </div>

        {/* Dynamic Recommendation Banner */}
        {classification && recommendedApp && (
          <div className="recommendation-banner" style={{ borderColor: recommendedApp.accent_color }}>
            <div className="rec-info">
              <span className="rec-badge" style={{ backgroundColor: recommendedApp.accent_color }}>
                {recommendedApp.name}
              </span>
              <span className="rec-reason">{classification.reason}</span>
            </div>
            <button
              className="rec-action-btn"
              onClick={() => handleDispatch(recommendedApp.id)}
              style={{ backgroundColor: recommendedApp.accent_color }}
            >
              Enviar a {recommendedApp.name}
            </button>
          </div>
        )}

        {/* Status Toast */}
        {statusMessage && (
          <div className={`status-toast ${statusMessage.type}`}>
            {statusMessage.type === "success" && "✅ "}
            {statusMessage.type === "error" && "❌ "}
            {statusMessage.type === "info" && "ℹ️ "}
            {statusMessage.text}
          </div>
        )}
      </section>

      {/* Catalog Search & Category Filters */}
      <section className="catalog-toolbar">
        <div className="toolbar-top-row">
          <div className="category-pills">
            {categories.map((cat) => (
              <button
                key={cat}
                className={`cat-pill ${selectedCategory === cat ? "active" : ""}`}
                onClick={() => setSelectedCategory(cat)}
              >
                {cat}
              </button>
            ))}
          </div>

          <div className="search-box">
            <input
              type="text"
              className="search-input"
              placeholder="🔍 Buscar aplicación..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
        </div>

        {/* Platform Sub-Filters */}
        <div className="platform-pills">
          <span className="platform-filter-label">S.O. Compatible:</span>
          {platformsList.map((p) => (
            <button
              key={p.id}
              className={`platform-pill ${selectedPlatform === p.id ? "active" : ""}`}
              onClick={() => setSelectedPlatform(p.id)}
            >
              <span>{p.icon}</span>
              <span>{p.label}</span>
            </button>
          ))}
        </div>
      </section>

      {/* Grid de Aplicaciones Satélite (2 cols en móvil, 4 a 6 cols en PC) */}
      <section className="apps-section">
        {filteredApps.length === 0 ? (
          <div className="empty-apps-state">
            <span className="empty-icon">🔍</span>
            <h3>No se encontraron aplicaciones</h3>
            <p>Prueba seleccionando otro S.O. o cambiando la categoría activa.</p>
            <button
              className="empty-reset-btn"
              onClick={() => {
                setSelectedPlatform(isMobile ? "android" : "windows");
                setSelectedCategory("Aplicaciones");
                setSearchQuery("");
              }}
            >
              Restablecer Filtros
            </button>
          </div>
        ) : (
          <div className="apps-grid">
            {filteredApps.map((app) => {
              const isRec = classification?.recommended_app_id === app.id;
              const isUpcoming = app.status === "PROXIMAMENTE";
              const hasValidImage = app.icon_path && !failedIcons[app.id];

              return (
                <div
                  key={app.id}
                  className={`app-card ${isRec ? "card-highlighted" : ""} ${isUpcoming ? "card-upcoming" : ""}`}
                  style={{ "--card-accent": app.accent_color } as React.CSSProperties}
                  onClick={() => {
                    if (!isUpcoming) handleDispatch(app.id);
                  }}
                >
                  <div className="app-card-top">
                    <div
                      className="app-icon-wrap"
                      style={{
                        borderColor: !hasValidImage ? app.accent_color : undefined,
                        background: !hasValidImage
                          ? `linear-gradient(135deg, ${app.accent_color}22, ${app.accent_color}44)`
                          : undefined,
                      }}
                    >
                      {hasValidImage ? (
                        <img
                          src={app.icon_path}
                          alt={app.name}
                          className="app-icon-img"
                          onError={() => {
                            setFailedIcons((prev) => ({ ...prev, [app.id]: true }));
                          }}
                        />
                      ) : (
                        <span className="app-icon-emoji">{app.icon_emoji || "⚡"}</span>
                      )}
                    </div>
                    
                    <div className="badges-stack">
                      {app.is_web_app ? (
                        <span className="platform-os-badge badge-web">🌐 WEB</span>
                      ) : app.platforms?.includes("android") && app.platforms?.includes("windows") ? (
                        <span className="platform-os-badge badge-hybrid">📱/🖥️ HÍBRIDO</span>
                      ) : app.platforms?.includes("android") ? (
                        <span className="platform-os-badge badge-android">📱 ANDROID</span>
                      ) : (
                        <span className="platform-os-badge badge-pc">🖥️ PC SOLO</span>
                      )}
                      <span className={`status-badge ${getStatusBadgeClass(app.status)}`}>
                        {app.status}
                      </span>
                    </div>
                  </div>

                  <h3 className="app-name">{app.name}</h3>
                  <p className="app-desc">{app.description}</p>

                  <div className="app-footer">
                    <div className="app-platforms-tag">
                      {app.platforms?.includes("windows") && <span title="Windows">🪟</span>}
                      {app.platforms?.includes("android") && <span title="Android">📱</span>}
                      {app.platforms?.includes("linux") && <span title="Linux">🐧</span>}
                      {app.platforms?.includes("web") && <span title="Web">🌐</span>}
                    </div>

                    <button
                      className="card-dispatch-btn"
                      disabled={isUpcoming}
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDispatch(app.id);
                      }}
                    >
                      {getCardActionText(app)}
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </section>

      {/* Settings Modal */}
      {isSettingsOpen && (
        <div className="modal-backdrop" onClick={() => setIsSettingsOpen(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <div className="modal-header-title">
                <span className="modal-icon">⚙️</span>
                <h2>Configuración de Aurora Synapse</h2>
              </div>
              <button className="modal-close-btn" onClick={() => setIsSettingsOpen(false)}>
                ✕
              </button>
            </div>

            <div className="modal-body">
              {/* Theme Preference */}
              <div className="setting-row">
                <div className="setting-info">
                  <span className="setting-title">Tema de la Interfaz</span>
                  <span className="setting-desc">Apariencia en modo automático, oscuro o claro.</span>
                </div>
                <div className="theme-toggle-chips">
                  <button
                    className={`theme-chip-icon ${theme === "system" ? "active" : ""}`}
                    title="Automático (Sigue el tema del sistema / dispositivo)"
                    onClick={() => setTheme("system")}
                  >
                    💻
                  </button>
                  <button
                    className={`theme-chip-icon ${theme === "dark" ? "active" : ""}`}
                    title="Modo Oscuro"
                    onClick={() => setTheme("dark")}
                  >
                    🌙
                  </button>
                  <button
                    className={`theme-chip-icon ${theme === "light" ? "active" : ""}`}
                    title="Modo Claro"
                    onClick={() => setTheme("light")}
                  >
                    ☀️
                  </button>
                </div>
              </div>

              {!isMobile ? (
                <>
                  <div className="setting-row">
                    <div className="setting-info">
                      <span className="setting-title">Iniciar con Windows (Auto-Run)</span>
                      <span className="setting-desc">Ejecuta Aurora Synapse automáticamente al iniciar el sistema operativo.</span>
                    </div>
                    <label className="toggle-switch">
                      <input
                        type="checkbox"
                        checked={settings.autostart}
                        onChange={handleToggleAutostart}
                      />
                      <span className="toggle-slider"></span>
                    </label>
                  </div>

                  <div className="setting-row">
                    <div className="setting-info">
                      <span className="setting-title">Minimizar a la Bandeja al Cerrar (X)</span>
                      <span className="setting-desc">Mantiene el proceso en segundo plano en el System Tray para responder al enrutamiento.</span>
                    </div>
                    <label className="toggle-switch">
                      <input
                        type="checkbox"
                        checked={settings.minimize_to_tray}
                        onChange={handleToggleMinimizeToTray}
                      />
                      <span className="toggle-slider"></span>
                    </label>
                  </div>
                </>
              ) : (
                <>
                  <div className="setting-row">
                    <div className="setting-info">
                      <span className="setting-title">Mantenerse en Segundo Plano (Servicio Android)</span>
                      <span className="setting-desc">Mantiene activo el listener de enrutamiento y recepción en Android.</span>
                    </div>
                    <label className="toggle-switch">
                      <input
                        type="checkbox"
                        checked={androidBgResident}
                        onChange={() => setAndroidBgResident(!androidBgResident)}
                      />
                      <span className="toggle-slider"></span>
                    </label>
                  </div>

                  <div className="setting-row">
                    <div className="setting-info">
                      <span className="setting-title">Auto-inicio con el Dispositivo (Boot)</span>
                      <span className="setting-desc">Inicia el enrutador de Synapse al encender tu teléfono.</span>
                    </div>
                    <label className="toggle-switch">
                      <input
                        type="checkbox"
                        checked={androidAutoBoot}
                        onChange={() => setAndroidAutoBoot(!androidAutoBoot)}
                      />
                      <span className="toggle-slider"></span>
                    </label>
                  </div>
                </>
              )}

              <div className="setting-row">
                <div className="setting-info">
                  <span className="setting-title">Descubrimiento LAN (Puerto 49295)</span>
                  <span className="setting-desc">Permite enviar y recibir enlaces y medios entre dispositivos en la misma red local.</span>
                </div>
                <span className="active-badge">Activo</span>
              </div>

              <div className="setting-lan-box">
                <div className="setting-info">
                  <span className="setting-title">💻 Dirección IP de la PC (LAN)</span>
                  <span className="setting-desc">
                    Permite enviar descargas desde el teléfono hacia Gallery-DL y Luna Fetch ejecutándose en la PC ({pcIp}:49295 / 18274).
                  </span>
                </div>
                <div className="lan-ip-control-row">
                  <input
                    type="text"
                    className="lan-ip-input"
                    value={pcIp}
                    placeholder="192.168.1.121"
                    onChange={(e) => handlePcIpChange(e.target.value)}
                  />
                  <button
                    className={`lan-ping-btn ${pingStatus}`}
                    disabled={pingStatus === "testing"}
                    onClick={testPcConnection}
                  >
                    {pingStatus === "testing"
                      ? "⏳ Conectando..."
                      : pingStatus === "ok"
                      ? "✅ Conectado"
                      : pingStatus === "fail"
                      ? "❌ Sin conexión"
                      : "🔍 Probar Conexión"}
                  </button>
                </div>
              </div>

              <hr className="modal-divider" />

              <div className="about-section">
                <h3>Acerca de Aurora Synapse</h3>
                <p className="about-text">
                  Orquestador Universal del Ecosistema <strong>Aurora</strong>. Creado por <strong>Biglex J</strong> bajo licencia GNU GPL v2.0 (2026).
                </p>
                <div className="about-links">
                  <a href="https://www.biglexj.com/donaciones" target="_blank" rel="noreferrer" className="about-link donation">
                    💖 Apoyar Proyecto
                  </a>
                  <a href="https://buymeacoffee.com/biglexj" target="_blank" rel="noreferrer" className="about-link coffee">
                    ☕ Buy Me a Coffee
                  </a>
                  <a href="https://github.com/biglexj" target="_blank" rel="noreferrer" className="about-link github">
                    🐙 GitHub Oficial
                  </a>
                </div>

                <div className="check-update-trigger">
                  <div>
                    <span style={{ fontSize: "12px", fontWeight: 600, color: "#94a3b8", display: "block" }}>
                      Versión Actual: v{APP_CURRENT_VERSION}
                    </span>
                    <span style={{ fontSize: "11px", color: "#64748b" }}>
                      Canal oficial GitHub Releases
                    </span>
                  </div>
                  <button
                    className="check-update-btn"
                    disabled={isCheckingUpdates}
                    onClick={handleManualUpdateCheck}
                  >
                    {isCheckingUpdates ? "Buscando..." : "Buscar actualizaciones"}
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Toast Flotante Centrado Superior (Core-Docs Req #2 & #7) */}
      {toastMessage && (
        <Toast
          message={toastMessage}
          type={toastType}
          durationMs={4000}
          onClose={() => setToastMessage(null)}
        />
      )}

      {/* Modal Central Interactivo de Actualización (Core-Docs Req #4) */}
      {showUpdateModal && pendingUpdate && (
        <UpdateModalDialog
          update={pendingUpdate}
          isMobile={isMobile}
          onClose={() => setShowUpdateModal(false)}
        />
      )}
    </div>
  );
}
