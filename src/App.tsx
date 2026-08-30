import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface AppTarget {
  id: string;
  name: string;
  category: string;
  description: string;
  icon_path: string;
  accent_color: string;
  uri_scheme: string;
  web_url?: string;
  port: number;
  supported_domains: string[];
  is_web_app: boolean;
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

export default function App() {
  const [apps, setApps] = useState<AppTarget[]>([]);
  const [devices, setDevices] = useState<DeviceNode[]>([]);
  const [selectedDevice, setSelectedDevice] = useState<string>("local_pc");
  const [inputText, setInputText] = useState<string>("");
  const [searchQuery, setSearchQuery] = useState<string>("");
  const [selectedCategory, setSelectedCategory] = useState<string>("Todos");
  const [classification, setClassification] = useState<ClassificationResult | null>(null);
  const [statusMessage, setStatusMessage] = useState<{ text: string; type: "success" | "error" | "info" } | null>(null);
  const [isDispatching, setIsDispatching] = useState<boolean>(false);

  useEffect(() => {
    loadInitialData();
  }, []);

  const loadInitialData = async () => {
    try {
      const registeredApps = await invoke<AppTarget[]>("get_registered_apps");
      const pairedDevices = await invoke<DeviceNode[]>("get_paired_devices");
      setApps(registeredApps);
      setDevices(pairedDevices);
    } catch (err) {
      console.error("Error al cargar apps iniciales:", err);
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

  const categories = useMemo(() => {
    const cats = new Set<string>();
    apps.forEach((a) => cats.add(a.category));
    return ["Todos", ...Array.from(cats)];
  }, [apps]);

  const filteredApps = useMemo(() => {
    return apps.filter((app) => {
      const matchesSearch =
        app.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        app.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
        app.category.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesCat = selectedCategory === "Todos" || app.category === selectedCategory;
      return matchesSearch && matchesCat;
    });
  }, [apps, searchQuery, selectedCategory]);

  const recommendedApp = apps.find((a) => a.id === classification?.recommended_app_id);

  return (
    <div className="synapse-container">
      {/* Header */}
      <header className="synapse-header">
        <div className="header-brand">
          <div className="brand-logo">⚡</div>
          <div>
            <h1 className="brand-title">Aurora Synapse</h1>
            <p className="brand-subtitle">Orquestador Universal & Router Inteligente de Intención</p>
          </div>
        </div>

        {/* Device Switcher */}
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
      </header>

      {/* Smart Input & Dispatch Area */}
      <section className="dispatch-card">
        <div className="input-group">
          <input
            type="text"
            className="synapse-input"
            placeholder="Pega un enlace (DeviantArt, YouTube, Pixiv), ruta de archivo local o texto..."
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
            placeholder="Buscar aplicación..."
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
            return (
              <div
                key={app.id}
                className={`app-card ${isRec ? "card-highlighted" : ""}`}
                style={{ "--card-accent": app.accent_color } as React.CSSProperties}
                onClick={() => handleDispatch(app.id)}
              >
                <div className="app-card-top">
                  <div className="app-icon-wrap">
                    <img
                      src={app.icon_path}
                      alt={app.name}
                      className="app-icon-img"
                      onError={(e) => {
                        // Fallback to initial letter icon
                        (e.currentTarget as HTMLElement).style.display = "none";
                      }}
                    />
                  </div>
                  <span className="app-category">{app.category}</span>
                </div>

                <h3 className="app-name">{app.name}</h3>
                <p className="app-desc">{app.description}</p>

                <div className="app-footer">
                  <span className="app-status">
                    {app.is_web_app ? "🌐 Web App" : `Puerto ${app.port}`}
                  </span>
                  <button
                    className="card-dispatch-btn"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDispatch(app.id);
                    }}
                  >
                    {inputText.trim() ? "Enviar" : app.is_web_app ? "Visitar" : "Abrir"}
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      </section>
    </div>
  );
}
