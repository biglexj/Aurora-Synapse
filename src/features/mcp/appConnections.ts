/**
 * Conexión en vivo de las aplicaciones del catálogo con el puente MCP.
 *
 * El catálogo de Sinapse usa identificadores propios (`lienzo`, `luna`,
 * `prisma`, `prisma-upscaler`, `gallerydl`) y el puente MCP usa los suyos
 * (`super_gallery`, `luna_fetch`, `prisma`, `prisma_upscaler`, `gallery_dl`).
 * Este módulo es el único sitio donde se relacionan ambas listas.
 *
 * Los datos vienen del comando nativo `mcp_server_status`. Si el backend aún no
 * lo expone, no hay conexión que pintar: la interfaz no inventa estados.
 */

import { fetchMcpStatus } from "./mcpService";
import type { McpServerStatus } from "./types";

/** Identificador del catálogo de Sinapse → identificador en el puente MCP. */
export const CATALOG_TO_BRIDGE_APP: Record<string, string> = {
  lienzo: "super_gallery",
  luna: "luna_fetch",
  prisma: "prisma",
  "prisma-upscaler": "prisma_upscaler",
  gallerydl: "gallery_dl",
  "davinci-flow": "davinci_flow",
  davinciflow: "davinci_flow",
};

/** Estado de conexión de una aplicación del catálogo. */
export interface AppConnection {
  /** La aplicación responde ahora mismo. */
  connected: boolean;
  /** El puente publica herramientas MCP para ella. */
  mcpTools: boolean;
  /** Cuántas herramientas MCP publica el puente para ella. */
  toolCount: number;
  /** Detalle legible del backend, si lo hay. */
  detail?: string;
}

/**
 * Traduce el payload del puente al estado de conexión por aplicación.
 *
 * `connected` solo es `true` con evidencia directa (`availability: "available"`),
 * que el backend emite tras sondear la app de verdad. `mcpTools` se cuenta a
 * partir de las herramientas publicadas, no se supone: una app puede estar
 * conectada y aun así no exponer ninguna herramienta.
 */
export function readAppConnections(status: McpServerStatus | null): Record<string, AppConnection> {
  const connections: Record<string, AppConnection> = {};
  if (!status) return connections;

  for (const [catalogId, bridgeId] of Object.entries(CATALOG_TO_BRIDGE_APP)) {
    const app = status.apps.find((candidate) => candidate.id === bridgeId);
    if (!app) continue;

    const toolCount = status.tools.filter((tool) => tool.app_id === bridgeId).length;
    connections[catalogId] = {
      connected: app.availability === "available",
      mcpTools: toolCount > 0,
      toolCount,
      detail: app.detail,
    };
  }

  return connections;
}

/** Consulta el puente y devuelve el estado de conexión por aplicación. */
export async function fetchAppConnections(): Promise<Record<string, AppConnection>> {
  const view = await fetchMcpStatus();
  return readAppConnections(view.status);
}

/**
 * Texto explicativo del badge.
 *
 * Distingue las dos situaciones para no prometer una capacidad que no existe:
 * una app conectada con herramientas MCP puede operarse con un agente; una
 * app conectada sin herramientas solo está enlazada con Synapse.
 */
export function describeConnection(connection: AppConnection): string {
  if (connection.mcpTools) {
    const tools = connection.toolCount === 1 ? "1 herramienta" : `${connection.toolCount} herramientas`;
    return `Conectada ahora mismo. El puente MCP publica ${tools} para esta aplicación.`;
  }
  return "Conectada ahora mismo con Aurora Synapse. El puente todavía no publica herramientas MCP para esta aplicación.";
}
