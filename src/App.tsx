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
}

interface DeviceNode {
  id: string;
  name: string;
  device_type: string;
  ip: string;
  is_local: boolean;
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
  const [inputText, setInputText] = useState<string>("");
  const [searchQuery, setSearchQuery] = useState<string>("");
  const [selectedCategory, setSelectedCategory] = useState<string>("Aplicaciones");
  const [classification, setClassification] = useState<ClassificationResult | null>(null);
  const [statusMessage, setStatusMessage] = useState<{ text: string; type: "success" | "error" | "info" } | null>(null);
  const [isDispatching, setIsDispatching] = useState<boolean>(false);
  const [failedIcons, setFailedIcons] = useState<Record<string, boolean>>({});

  // Settings State
  const [isSettingsOpen, setIsSettingsOpen] = useState<boolean>(false);
  const [settings, setSettings] = useState<SynapseSettings>({
    autostart: false,
    minimize_to_tray: true,
    lan_discovery: true,
  });

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
      setApps(registeredApps);
      setDevices(pairedDevices);
      setSettings(currentSettings);
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
        content: inputText.trim(),
        targetDeviceId: selectedDevice,
      });
      setStatusMessage({ text: res, type: "success" });
      if (inputText.trim()) {
        setInputText("");
        setClassification(null);
      }
    } catch (err: any) {
      setStatusMessage({ text: String(err), type: "error" });
    } finally {
      setIsDispatching(false);
      setTimeout(() => {
        setStatusMessage(null);
      }, 5000);
    }
  };

  const toggleAutostart = async () => {
    try {
      const nextState = !settings.autostart;
      await invoke("set_autostart_setting", { enable: nextState });
      setSettings((prev) => ({ ...prev, autostart: nextState }));
    } catch (err) {
      console.error("Error al cambiar auto-inicio:", err);
    }
  };

  const toggleMinimizeToTray = async () => {
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

      return matchesSearch && matchesCat;
    });
  }, [apps, searchQuery, selectedCategory]);

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

        {/* Right Header Actions: Device Switcher & Settings */}
        <div className="header-right-actions">
          <div className="device-switcher">
            <span className="device-label">Dispositivo Destino:</span>
            <div className="device-pills">
              {devices.map((dev) => (
                <button
                  key={dev.id}
                  className={`device-pill ${selectedDevice === dev.id ? "active" : ""}`}
                  onClick={() => setSelectedDevice(dev.id)}
                >
                  <span>{dev.device_type === "mobile" ? "📱" : "💻"}</span>
                  <span>{dev.name}</span>
                </button>
              ))}
            </div>
          </div>

          <button
            className="settings-trigger-btn"
            title="Configuración de Aurora Synapse"
            onClick={() => setIsSettingsOpen(true)}
          >
            ⚙️
          </button>
        </div>
      </header>

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
      </section>

      {/* Grid de Aplicaciones Satélite (2 cols en móvil, 4 cols en PC) */}
      <section className="apps-section">
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
                  <span className={`status-badge ${getStatusBadgeClass(app.status)}`}>
                    {app.status}
                  </span>
                </div>

                <h3 className="app-name">{app.name}</h3>
                <p className="app-desc">{app.description}</p>

                <div className="app-footer">
                  <span className="app-status-info">
                    {app.is_web_app ? "🌐 Web App" : `Puerto ${app.port}`}
                  </span>
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
              {/* Preferences Section */}
              <div className="settings-group">
                <h3 className="settings-group-title">COMPORTAMIENTO DEL SISTEMA</h3>

                <label className="setting-row">
                  <div className="setting-info">
                    <span className="setting-label">Iniciar con Windows (Auto-Run)</span>
                    <span className="setting-desc">
                      Inicia Aurora Synapse en segundo plano al encender el equipo.
                    </span>
                  </div>
                  <input
                    type="checkbox"
                    className="setting-checkbox"
                    checked={settings.autostart}
                    onChange={toggleAutostart}
                  />
                </label>

                <label className="setting-row">
                  <div className="setting-info">
                    <span className="setting-label">Minimizar a la Bandeja al Cerrar (X)</span>
                    <span className="setting-desc">
                      Mantiene la aplicación residente en el System Tray para enrutamiento instantáneo.
                    </span>
                  </div>
                  <input
                    type="checkbox"
                    className="setting-checkbox"
                    checked={settings.minimize_to_tray}
                    onChange={toggleMinimizeToTray}
                  />
                </label>

                <label className="setting-row">
                  <div className="setting-info">
                    <span className="setting-label">Descubrimiento LAN (Puerto 49295)</span>
                    <span className="setting-desc">
                      Escucha conexiones de aplicaciones satélite y dispositivos móviles en la red local.
                    </span>
                  </div>
                  <input
                    type="checkbox"
                    className="setting-checkbox"
                    checked={settings.lan_discovery}
                    readOnly
                  />
                </label>
              </div>

              {/* About Ecosystem Section */}
              <div className="settings-group about-section">
                <h3 className="settings-group-title">ACERCA DEL ECOSISTEMA</h3>
                <div className="about-card">
                  <div className="about-logo">⚡</div>
                  <div className="about-details">
                    <h4 className="about-app-name">Aurora Synapse v0.1.0</h4>
                    <p className="about-author">Desarrollado por <strong>Biglex J</strong> · 2026</p>
                    <p className="about-license">Licencia GNU GPL v2.0 (GPL-2.0)</p>
                  </div>
                </div>

                <div className="about-buttons">
                  <button
                    className="about-btn primary"
                    onClick={() => invoke("dispatch_content", {
                      appId: "about-donations",
                      content: "",
                      targetDeviceId: "local_pc",
                    }).catch(() => {})}
                  >
                    💖 Donaciones Oficiales (Yape / Plin / Web)
                  </button>
                  <button
                    className="about-btn secondary"
                    onClick={() => window.open("https://buymeacoffee.com/biglexj", "_blank")}
                  >
                    ☕ Buy Me a Coffee
                  </button>
                  <button
                    className="about-btn secondary"
                    onClick={() => window.open("https://github.com/biglexj", "_blank")}
                  >
                    ⭐ GitHub Oficial
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
