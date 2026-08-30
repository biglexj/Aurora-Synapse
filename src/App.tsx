import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

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
  const [isMobile, setIsMobile] = useState<boolean>(false);
  const [theme, setTheme] = useState<"dark" | "light">(() => {
    return (localStorage.getItem("synapse_theme") as "dark" | "light") || "dark";
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

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("synapse_theme", theme);
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
      const registeredApps = await invoke<AppTarget[]>("get_registered_apps");
      const pairedDevices = await invoke<DeviceNode[]>("get_paired_devices");
      const currentSettings = await invoke<SynapseSettings>("get_synapse_settings");
      const mobileCheck = await invoke<boolean>("is_mobile_platform").catch(() => /Android|iPhone|iPad/i.test(navigator.userAgent));
      const isMob = Boolean(mobileCheck) || /Android|iPhone|iPad/i.test(navigator.userAgent);
      
      setApps(registeredApps);
      setDevices(pairedDevices);
      setSettings(currentSettings);
      setIsMobile(isMob);

      // Default platform selection based on active OS
      if (isMob) {
        setSelectedPlatform("android");
        setSelectedDevice("android_phone");
      } else {
        setSelectedPlatform("windows");
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

  const handleDispatch = async (targetAppId?: string) => {
    const appId = targetAppId || classification?.recommended_app_id;
    if (!appId) return;

    const targetApp = apps.find((a) => a.id === appId);
    if (targetApp?.status === "PROXIMAMENTE") {
      setStatusMessage({ text: `${targetApp.name} estará disponible próximamente.`, type: "info" });
      return;
    }

    setIsDispatching(true);
    setStatusMessage({ text: "Despachando a través de Aurora Synapse...", type: "info" });

    try {
      const res = await invoke<string>("dispatch_content", {
        appId,
        content: inputText,
        targetDeviceId: selectedDevice,
      });
      setStatusMessage({ text: res, type: "success" });
      setInputText("");
    } catch (err) {
      setStatusMessage({ text: String(err), type: "error" });
    } finally {
      setIsDispatching(false);
      setTimeout(() => setStatusMessage(null), 4000);
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
      { id: "windows", label: "Windows", icon: "🪟" },
      { id: "android", label: "Android", icon: "📱" },
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

      const matchesPlatform =
        !selectedPlatform ||
        (app.platforms && app.platforms.includes(selectedPlatform));

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

  return (
    <div className="synapse-container">
      {/* Header */}
      <header className="synapse-header">
        <div className="header-brand">
          <div className="brand-logo">⚡</div>
          <div>
            <span className="launchpad-tag">LAUNCHPAD & ROUTER</span>
            <h1 className="brand-title">TODAS LAS APLICACIONES</h1>
            <p className="brand-subtitle">Orquestador Universal del Ecosistema Aurora</p>
          </div>
        </div>

        <button
          className="settings-trigger-btn"
          title="Configuración de Aurora Synapse"
          onClick={() => setIsSettingsOpen(true)}
        >
          ⚙️
        </button>
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
                      {isUpcoming
                        ? "Próximamente"
                        : inputText.trim()
                        ? "Enviar"
                        : app.is_web_app
                        ? "Visitar"
                        : "Lanzar"}
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
                  <span className="setting-desc">Personaliza la apariencia visual en modo oscuro medianoche o claro.</span>
                </div>
                <div className="theme-toggle-chips">
                  <button
                    className={`theme-chip ${theme === "dark" ? "active" : ""}`}
                    onClick={() => setTheme("dark")}
                  >
                    🌙 Oscuro
                  </button>
                  <button
                    className={`theme-chip ${theme === "light" ? "active" : ""}`}
                    onClick={() => setTheme("light")}
                  >
                    ☀️ Claro
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
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
