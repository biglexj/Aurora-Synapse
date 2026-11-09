/**
 * Contrato de datos del panel de estado del MCP.
 *
 * El panel distingue tres conceptos que NO son intercambiables:
 *
 * 1. **Servidor MCP activo**: el puente de Aurora Synapse está escuchando y
 *    responde peticiones MCP. Es una propiedad del proceso local.
 * 2. **Aplicación disponible**: la aplicación satélite existe, está instalada
 *    o el adaptador MCP la soporta. Que el servidor esté encendido NO implica
 *    que la aplicación esté disponible.
 * 3. **Agente conectado**: hay un cliente de agente (Ely Intelligence, agente
 *    de terminal, ...) con una sesión MCP viva. Que el servidor esté encendido
 *    NO implica que haya un agente conectado.
 *
 * El payload lo produce el comando nativo `mcp_server_status`. El frontend
 * solo lo consume: nunca lo inventa ni lo simula.
 */

/** Estado del servidor MCP (el puente). */
export type McpServerState =
  | "active"     // escuchando y respondiendo
  | "inactive"   // el backend informa que no hay puente en marcha
  | "loading"    // consulta en curso
  | "error"      // la consulta falló
  | "unavailable"; // el backend todavía no expone el comando

/** Disponibilidad de una aplicación frente al puente MCP. */
export type McpAppAvailability =
  | "available"    // instalada y alcanzable ahora mismo
  | "unavailable"  // el adaptador la soporta pero no responde
  | "unknown";     // sin datos: no se puede afirmar nada

/** Estado de conexión de un cliente de agente. */
export type McpAgentConnection =
  | "connected"
  | "disconnected"
  | "unknown";

/** Herramienta publicada por el puente MCP. */
export interface McpTool {
  name: string;
  description?: string;
  /** Identificador de la aplicación a la que pertenece la herramienta. */
  app_id: string;
  read_only: boolean;
  destructive: boolean;
}

/** Aplicación satélite vista desde el puente MCP. */
export interface McpApp {
  id: string;
  name: string;
  availability: McpAppAvailability;
  /** Detalle legible del por qué (por ejemplo: "CLI instalada, sin servicio HTTP"). */
  detail?: string;
}

/** Cliente de agente con una sesión MCP viva. */
export interface McpAgent {
  client: string;
  connection: McpAgentConnection;
  connected_at?: string;
  /**
   * Marca de tiempo de la última petición aceptada de la sesión.
   *
   * "Conectado" significa actividad reciente, no un socket permanente: este
   * dato es el que permite distinguir a un cliente que acaba de hablar de uno
   * que lleva minutos sin hacerlo.
   */
  last_activity_at?: string;
  /** Segundos desde la última petición aceptada. */
  idle_seconds?: number;
  protocol_version?: string;
}

/**
 * Payload esperado del comando Tauri `mcp_server_status`.
 *
 * Todos los campos son opcionales a efectos de consumo: el frontend normaliza
 * lo que llega y trata lo que falta como "desconocido", nunca como "activo".
 */
export interface McpServerStatus {
  /** El puente está escuchando y responde. */
  running: boolean;
  /** "http" | "stdio" | "none". */
  transport: string;
  /** Dirección completa del puente, p. ej. "http://127.0.0.1:49296/mcp". */
  endpoint: string | null;
  host: string | null;
  port: number | null;
  /** Versión del protocolo MCP negociada, p. ej. "2025-06-18". */
  protocol_version: string | null;
  /** El puente exige clave de API para aceptar conexiones. */
  api_key_required: boolean;
  /** Número de herramientas publicadas. */
  tool_count: number;
  /** Herramientas publicadas, con su aplicación de destino. */
  tools: McpTool[];
  /** Aplicaciones y su disponibilidad real. */
  apps: McpApp[];
  /** Clientes de agente conectados. */
  agents: McpAgent[];
  /**
   * Estado de conexión de agente resumido.
   * `null` significa "sin evidencia": el backend no sabe, no "desconectado".
   */
  agent_connected: boolean | null;
  /** Marca de tiempo ISO 8601 del arranque del puente. */
  started_at: string | null;
  /** Descripción del último error conocido del puente. */
  error: string | null;
}

/** Vista normalizada que consume el panel. */
export interface McpStatusView {
  server_state: McpServerState;
  /** Descripción corta del estado actual, lista para mostrar. */
  message: string;
  /** Payload normalizado; `null` cuando el backend aún no expone el comando. */
  status: McpServerStatus | null;
}

/** Identificadores de aplicación canónicos del puente MCP. */
export const MCP_APP_IDS = {
  auroraSynapse: "aurora_synapse",
  superGallery: "super_gallery",
  lunaFetch: "luna_fetch",
  prisma: "prisma",
  prismaUpscaler: "prisma_upscaler",
  davinciFlow: "davinci_flow",
} as const;

/** Etiquetas legibles para los identificadores de aplicación conocidos. */
export const MCP_APP_LABELS: Record<string, string> = {
  [MCP_APP_IDS.auroraSynapse]: "Aurora Synapse",
  [MCP_APP_IDS.superGallery]: "Super Galería",
  [MCP_APP_IDS.lunaFetch]: "Luna Fetch",
  [MCP_APP_IDS.prisma]: "Prisma",
  [MCP_APP_IDS.prismaUpscaler]: "Prisma Upscaler",
  [MCP_APP_IDS.davinciFlow]: "DaVinci Flow",
};

/**
 * Deduce la aplicación de una herramienta por su prefijo.
 *
 * Es una derivación del catálogo real de herramientas del puente (no un dato
 * inventado) y solo se usa cuando el backend omite `app_id`.
 */
export function inferAppId(toolName: string): string {
  if (toolName.startsWith("gallery_")) return MCP_APP_IDS.superGallery;
  if (toolName.startsWith("luna_")) return MCP_APP_IDS.lunaFetch;
  if (toolName.startsWith("upscaler_")) return MCP_APP_IDS.prismaUpscaler;
  if (toolName.startsWith("prisma_")) return MCP_APP_IDS.prisma;
  if (toolName.startsWith("davinciflow_")) return MCP_APP_IDS.davinciFlow;
  return MCP_APP_IDS.auroraSynapse;
}
