import { invoke } from "@tauri-apps/api/core";
import { UpdateCheckResult } from "./types";

export const APP_CURRENT_VERSION = "1.0.0";

/**
 * Sanitiza canónicamente el markdown del body de la release para su presentación en UI
 * de acuerdo a los estándares del Core (Core-Docs).
 */
export function sanitizeReleaseNotes(body: string): string {
  if (!body) return "Mejoras de estabilidad, rendimiento y enrutamiento en el ecosistema Aurora.";
  return body
    .replace(/^#+\s*(.*)$/gm, "$1") // Convierte encabezados # a texto plano
    .replace(/^[-*]\s+/gm, "• ") // Convierte bullets a puntos estilizados
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1") // Elimina links en markdown conservando texto
    .replace(/[*_~`]/g, "") // Remueve marcas de énfasis
    .trim();
}

/**
 * Consulta la última versión disponible en GitHub Releases.
 * Utiliza el comando nativo de Tauri con fallback transparente a fetch web.
 */
export async function checkAppUpdates(): Promise<UpdateCheckResult> {
  try {
    const result = await invoke<UpdateCheckResult>("check_for_updates");
    return result;
  } catch (tauriErr) {
    console.warn("Comando nativo check_for_updates falló o en web, usando fetch directo:", tauriErr);
    
    const res = await fetch("https://api.github.com/repos/biglexj/Aurora-Synapse/releases/latest", {
      headers: { Accept: "application/vnd.github.v3+json" }
    });

    if (!res.ok) {
      throw new Error(`GitHub API HTTP ${res.status}`);
    }

    const data = await res.json();
    const tag = (data.tag_name || "").replace(/^v/, "");
    const available = compareSemVer(tag, APP_CURRENT_VERSION) > 0;
    
    let exe_url: string | undefined;
    let apk_url: string | undefined;
    const assets = (data.assets || []).map((a: any) => {
      if (typeof a.name === "string" && a.name.endsWith(".exe") && !a.name.includes("-portable")) {
        exe_url = a.browser_download_url;
      }
      if (typeof a.name === "string" && a.name.endsWith(".apk")) {
        apk_url = a.browser_download_url;
      }
      return {
        name: a.name || "",
        browser_download_url: a.browser_download_url || "",
        size: a.size || 0,
      };
    });

    return {
      available,
      current_version: APP_CURRENT_VERSION,
      latest_version: tag || APP_CURRENT_VERSION,
      release_name: data.name || data.tag_name || `Aurora Synapse v${tag}`,
      release_notes: data.body || "",
      published_at: data.published_at || "",
      html_url: data.html_url || "https://github.com/biglexj/Aurora-Synapse/releases",
      exe_url,
      apk_url,
      assets,
    };
  }
}

function compareSemVer(v1: string, v2: string): number {
  const p1 = v1.split(".").map((s) => parseInt(s.replace(/\D/g, ""), 10) || 0);
  const p2 = v2.split(".").map((s) => parseInt(s.replace(/\D/g, ""), 10) || 0);
  const len = Math.max(p1.length, p2.length);
  for (let i = 0; i < len; i++) {
    const a = p1[i] || 0;
    const b = p2[i] || 0;
    if (a > b) return 1;
    if (a < b) return -1;
  }
  return 0;
}
