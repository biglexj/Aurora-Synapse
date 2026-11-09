import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

interface GalleryDevice { device_id: string; device_name: string; address: string; port: number }
interface Connection { configured?: boolean; reachable?: boolean; paired?: boolean; url?: string }

const deviceUrl = (device: GalleryDevice) => `http://${device.address}:${device.port}`;

export function GalleryConnectionSettings({ isMobile }: { isMobile: boolean }) {
  const [devices, setDevices] = useState<GalleryDevice[]>([]);
  const [address, setAddress] = useState(isMobile ? "http://127.0.0.1:49288" : "");
  const [code, setCode] = useState("");
  const [connection, setConnection] = useState<Connection>({});
  const [message, setMessage] = useState("");
  const [busy, setBusy] = useState(false);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // El selector refleja la dirección activa: si coincide con un teléfono
  // detectado se muestra su nombre; si la dirección es manual, queda vacío.
  const activeDevice = devices.find(device => deviceUrl(device) === address);
  const selectedDevice = activeDevice ? address : "";

  useEffect(() => {
    let active = true;
    const refresh = async () => {
      const [found, state] = await Promise.allSettled([
        invoke<GalleryDevice[]>("gallery_devices"), invoke<Connection>("gallery_connection"),
      ]);
      if (!active) return;
      if (found.status === "fulfilled") setDevices(found.value);
      if (state.status === "fulfilled") {
        setConnection(state.value);
        if (state.value.url) setAddress(previous => previous || state.value.url!);
      }
    };
    void refresh();
    const timer = window.setInterval(() => { void refresh(); }, 10000);
    return () => { active = false; window.clearInterval(timer); };
  }, []);

  // Cerrar al hacer clic fuera o al presionar Escape
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setDropdownOpen(false);
      }
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setDropdownOpen(false);
    };
    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

  const pair = async () => {
    setBusy(true); setMessage("");
    try {
      await invoke("pair_gallery", { address: address.trim(), code });
      setCode("");
      setConnection(await invoke<Connection>("gallery_connection"));
      setMessage("Super Galería conectada. Los agentes ya pueden consultar y transferir medios.");
    } catch (error) { setMessage(String(error)); }
    finally { setBusy(false); }
  };

  return <section className="gallery-settings" aria-labelledby="gallery-settings-title">
    <div className="gallery-settings-heading"><img className="gallery-settings-icon" src="/assets/icons/lienzo-gallery/icon.webp" alt="" /><div>
      <span className="settings-group-title">Conexión con el teléfono</span>
      <h3 id="gallery-settings-title">Super Galería</h3>
    </div></div>
    <span className={`gallery-connection-badge ${connection.paired ? "is-connected" : ""}`}><span aria-hidden="true">●</span> {connection.paired ? "Conectada y autorizada" : connection.reachable ? "Disponible · falta autorizar" : "Sin conexión autorizada"}</span>
    <p className="gallery-settings-help">Consulta y transfiere medios usando la misma red local que Prisma.</p>
    <div className="gallery-pairing-hint"><strong>Vincular o renovar acceso</strong><p>En Super Galería, abre Ajustes → Conectar agentes con Synapse. Genera un código e introdúcelo aquí antes de que caduque.</p></div>
    
    {devices.length > 0 ? (
      <div className="gallery-field">
        <span className="gallery-field-label">Teléfono detectado</span>
        <div className="gallery-custom-select" ref={dropdownRef}>
          <button
            type="button"
            className={`gallery-select-trigger ${dropdownOpen ? "is-active" : ""} ${activeDevice ? "has-value" : ""}`}
            onClick={() => !busy && setDropdownOpen(!dropdownOpen)}
            disabled={busy}
            aria-haspopup="listbox"
            aria-expanded={dropdownOpen}
          >
            <div className="trigger-label-group">
              <span className="trigger-icon">{activeDevice ? "📱" : "🔍"}</span>
              <div className="trigger-text-wrapper">
                <span className="trigger-name">{activeDevice ? activeDevice.device_name : "Seleccionar teléfono"}</span>
                {activeDevice && <span className="trigger-sub">{deviceUrl(activeDevice)}</span>}
              </div>
            </div>
            <svg
              className={`trigger-chevron ${dropdownOpen ? "is-rotated" : ""}`}
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2.4"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <polyline points="6 9 12 15 18 9" />
            </svg>
          </button>

          {dropdownOpen && (
            <div className="gallery-select-dropdown" role="listbox">
              <div className="dropdown-header">Dispositivos con Super Galería en LAN</div>
              {devices.map((device) => {
                const url = deviceUrl(device);
                const isSelected = selectedDevice === url;
                return (
                  <button
                    key={device.device_id}
                    type="button"
                    role="option"
                    aria-selected={isSelected}
                    className={`dropdown-option ${isSelected ? "is-selected" : ""}`}
                    onClick={() => {
                      setAddress(url);
                      setDropdownOpen(false);
                    }}
                  >
                    <span className="option-icon">📱</span>
                    <div className="option-content">
                      <span className="option-title">{device.device_name}</span>
                      <span className="option-url">{url}</span>
                    </div>
                    {isSelected && <span className="option-check">✓</span>}
                  </button>
                );
              })}
            </div>
          )}
        </div>
      </div>
    ) : null}

    <label className="gallery-field">
      <span className="gallery-field-label">Dirección de Super Galería</span>
      <input className="synapse-input" value={address} onChange={event => setAddress(event.target.value)} placeholder="http://IP-del-teléfono:49288" disabled={busy} autoComplete="off" />
    </label>
    <label className="gallery-field">
      <span className="gallery-field-label">Código temporal del teléfono</span>
      <input className="synapse-input" type="password" inputMode="numeric" maxLength={6} autoComplete="off" value={code} onChange={event => setCode(event.target.value.replace(/\D/g, ""))} disabled={busy} />
    </label>
    <button className="gallery-connect-btn" onClick={() => { void pair(); }} disabled={busy || code.length !== 6 || !address.trim()}>{busy ? "Conectando…" : connection.paired ? "Renovar conexión" : "Conectar Super Galería"}</button>
    {message ? <p className="gallery-settings-help" role="status">{message}</p> : null}
  </section>;
}
