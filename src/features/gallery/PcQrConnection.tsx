import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Invitation { svg: string; address: string; expires_in: number }
interface Pairing { connected: boolean; device_name?: string; expires_in: number }
interface PcStatus { connected: boolean; ip: string; pc_name: string }

export function PcQrConnection({ isMobile, onConnected, children }: { isMobile: boolean; onConnected: (ip: string) => void; children?: ReactNode }) {
  const [invitation, setInvitation] = useState<Invitation | null>(null);
  const [pairing, setPairing] = useState<Pairing | null>(null);
  const [pc, setPc] = useState<PcStatus | null>(null);
  const [pendingQr, setPendingQr] = useState("");
  const [pendingAddress, setPendingAddress] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    if (!isMobile) return;
    let active = true;
    const refresh = async () => {
      try {
        const result = await invoke<PcStatus>("paired_pc_status");
        if (active) { setPc(result); if (result.connected) onConnected(result.ip); }
      } catch { if (active) setPc(null); }
    };
    void refresh();
    const timer = window.setInterval(() => { void refresh(); }, 10000);
    return () => { active = false; window.clearInterval(timer); };
  }, [isMobile]);

  useEffect(() => {
    if (!invitation || isMobile) return;
    let active = true;
    const refresh = async () => {
      try { const result = await invoke<Pairing>("pc_pairing_status"); if (active) setPairing(result); }
      catch { if (active) setError("No se pudo consultar la conexión del teléfono."); }
    };
    void refresh();
    const timer = window.setInterval(() => { void refresh(); }, 1000);
    return () => { active = false; window.clearInterval(timer); };
  }, [invitation, isMobile]);

  useEffect(() => () => { if (!isMobile) void invoke("cancel_pc_invitation").catch(() => {}); }, [isMobile]);

  const generate = async () => {
    setBusy(true); setError(""); setPairing(null);
    try { setInvitation(await invoke<Invitation>("create_pc_invitation")); }
    catch (error) { setError(String(error)); }
    finally { setBusy(false); }
  };
  const scan = async () => {
    setBusy(true); setError("");
    try {
      const raw = await invoke<string>("scan_pc_qr");
      const qr = new URL(raw);
      if (qr.protocol !== "aurora-synapse:" || qr.hostname !== "pair" || !qr.searchParams.get("address")) throw new Error("Este QR no es de conexión con Synapse.");
      setPendingQr(raw); setPendingAddress(qr.searchParams.get("address")!);
    } catch (error) { setError(String(error)); }
    finally { setBusy(false); }
  };
  const connect = async () => {
    setBusy(true); setError("");
    try {
      const result = await invoke<PcStatus>("connect_pc_from_qr", { qr: pendingQr });
      setPc(result); onConnected(result.ip); setPendingQr("");
    } catch (error) { setError(String(error)); }
    finally { setBusy(false); }
  };
  const connected = isMobile ? pc?.connected : pairing?.connected;
  const expired = pairing?.expires_in === 0;

  return <div className={`pc-qr-section ${invitation && !connected ? "showing-qr" : ""}`}>
    {(!invitation || connected) && children}
    <strong className="setting-title">{isMobile ? "Conectar con el PC por QR" : "Conectar un teléfono por QR"}</strong>
    <p className="gallery-settings-help">{isMobile ? "Abre Ajustes en Synapse del PC y escanea su código. Ambos deben estar en la misma red local." : "En Synapse del teléfono, abre Ajustes → Escanear QR del PC."}</p>
    {connected ? <p className="gallery-connection-badge is-connected" role="status">● {isMobile ? `Conectado a ${pc?.pc_name}` : `${pairing?.device_name} vinculado`}</p> : null}
    {isMobile ? <>
      <button type="button" className="lan-ping-btn" disabled={busy} onClick={() => { void scan(); }}>{busy ? "Conectando…" : "Escanear QR del PC"}</button>
      {pendingQr ? <div className="gallery-pairing-hint"><p>Conectar con <strong>{pendingAddress}</strong></p><div className="pc-qr-actions">
        <button type="button" className="gallery-connect-btn" disabled={busy} onClick={() => { void connect(); }}>Confirmar conexión</button>
        <button type="button" className="theme-chip" disabled={busy} onClick={() => setPendingQr("")}>Cancelar</button>
      </div></div> : null}
    </> : <>
      {invitation && !connected ? <div className="pc-qr-preview">
        {!expired ? <img src={`data:image/svg+xml,${encodeURIComponent(invitation.svg)}`} alt="Código QR temporal para conectar Synapse del teléfono" width="224" height="224" /> : null}
        <p className="gallery-settings-help" role="status">{expired ? "El QR caducó. Genera otro para conectar." : `Caduca en ${pairing?.expires_in ?? invitation.expires_in} s · ${invitation.address}`}</p>
      </div> : null}
      <div className="pc-qr-actions">
        <button type="button" className="lan-ping-btn" disabled={busy} onClick={() => { void generate(); }}>{busy ? "Generando…" : invitation ? "Generar otro QR" : "Mostrar QR de conexión"}</button>
        {invitation && !connected ? <button type="button" className="theme-chip" onClick={() => { void invoke("cancel_pc_invitation").catch(() => {}); setInvitation(null); setPairing(null); }}>Volver</button> : null}
      </div>
    </>}
    <p className="gallery-settings-help">El vínculo dura hasta 12 horas o hasta salir de Synapse del PC. El acceso a Super Galería se autoriza aparte.</p>
    {error ? <p className="gallery-settings-help" role="alert">{error}</p> : null}
  </div>;
}
