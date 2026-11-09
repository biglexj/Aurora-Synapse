import { invoke } from "@tauri-apps/api/core";
import {
  MCP_APP_LABELS,
  McpAgent,
  McpApp,
  McpAppAvailability,
  McpAgentConnection,
  McpServerStatus,
  McpServerState,
  McpStatusView,
  McpTool,
  inferAppId,
} from "./types";

/**
 * Comando nativo que debe exponer el backend para alimentar el panel.
 *
 * El backend todavía no lo registra (ver `docs/mcp/PLAN.md`, etapa 2). El
 * adaptador degrada con claridad en ese caso: el panel muestra "no
 * disponible / pendiente de backend" en lugar de inventar datos.
 */
export const MCP_STATUS_COMMAND = "mcp_server_status";

const VIEW_PENDING_BACKEND =
  "Pendiente de backend: el comando nativo `mcp_server_status` aún no está registrado.";

const VIEW_NO_TAURI =
  "Sin runtime de Tauri: el estado del MCP solo puede consultarse dentro de la aplicación de escritorio.";

/** Detecta si el error viene de "el comando no existe" o de "no estoy en Tauri". */
function classifyInvokeError(error: unknown): "pending" | "environment" | "unknown" {
  const message = String(
    error instanceof Error ? error.message : (error as { message?: string })?.message ?? error ?? ""
  ).toLowerCase();

  if (
    message.includes("__tauri_internals__") ||
    message.includes("is not a function") ||
    message.includes("window is not defined") ||
    message.includes("window is undefined")
  ) {
    return "environment";
  }
  if (
    message.includes("not found") ||
    message.includes("unknown command") ||
    message.includes("no such command") ||
    message.includes("command mcp_server_status")
  ) {
    return "pending";
  }
  return "unknown";
}

/**
 * Traduce un fallo de la consulta a un estado visible y honesto.
 *
 * "El comando no existe" no es un error de la interfaz: significa que el
 * backend todavía no expone el dato. Se muestra como "no disponible", nunca
 * como datos simulados.
 */
export function describeStatusFailure(error: unknown): { server_state: McpServerState; message: string } {
  const kind = classifyInvokeError(error);
  if (kind === "pending") {
    return { server_state: "unavailable", message: VIEW_PENDING_BACKEND };
  }
  if (kind === "environment") {
    return { server_state: "unavailable", message: VIEW_NO_TAURI };
  }
  const message = error instanceof Error ? error.message : String(error);
  return {
    server_state: "error",
    message: `No se pudo consultar el estado del MCP: ${message}`,
  };
}

function asBoolean(value: unknown): boolean {
  return value === true || value === "true" || value === 1;
}

function asString(value: unknown): string | null {
  if (typeof value === "string" && value.trim().length > 0) return value.trim();
  if (typeof value === "number") return String(value);
  return null;
}

function asNumber(value: unknown): number | null {
  if (typeof value === "number" && Number.isFinite(value)) return value;
  if (typeof value === "string" && value.trim() !== "") {
    const parsed = Number(value);
    if (Number.isFinite(parsed)) return parsed;
  }
  return null;
}

/**
 * Convierte el estado crudo de una aplicación en un tri-estado honesto.
 *
 * `reachable === true` es evidencia directa de que la aplicación responde.
 * `reachable === false` o `supported === false` son evidencia directa de que
 * no está disponible ahora mismo. Sin ninguno de esos datos, el estado queda
 * como "desconocido": la ausencia de datos no se convierte en un "no".
 */
function normalizeAvailability(raw: Record<string, unknown>): McpAppAvailability {
  const supported = asBoolean(raw.supported);
  const reachable = raw.reachable === true || raw.reachable === "true";
  const explicit = asString(raw.availability)?.toLowerCase();

  if (explicit === "available" || explicit === "online" || explicit === "ready") return "available";
  if (explicit === "unavailable" || explicit === "offline" || explicit === "not_installed") {
    return "unavailable";
  }
  if (reachable) return "available";
  if (raw.reachable === false || raw.reachable === "false") return "unavailable";
  if (raw.supported !== undefined && !supported) return "unavailable";
  return "unknown";
}

function normalizeTool(raw: unknown): McpTool | null {
  if (!raw || typeof raw !== "object") return null;
  const record = raw as Record<string, unknown>;
  const name = asString(record.name);
  if (!name) return null;

  const annotations =
    record.annotations && typeof record.annotations === "object"
      ? (record.annotations as Record<string, unknown>)
      : {};

  return {
    name,
    description: asString(record.description) ?? undefined,
    app_id: asString(record.app_id) ?? asString(record.appId) ?? inferAppId(name),
    read_only:
      record.read_only === undefined && annotations.readOnlyHint === undefined
        ? false
        : asBoolean(record.read_only ?? annotations.readOnlyHint),
    destructive: asBoolean(record.destructive ?? annotations.destructiveHint),
  };
}

function normalizeApp(raw: unknown): McpApp | null {
  if (!raw || typeof raw !== "object") return null;
  const record = raw as Record<string, unknown>;
  const id = asString(record.id) ?? asString(record.app_id);
  if (!id) return null;

  return {
    id,
    name: asString(record.name) ?? id,
    availability: normalizeAvailability(record),
    detail: asString(record.detail) ?? asString(record.status) ?? undefined,
  };
}

function normalizeAgent(raw: unknown): McpAgent | null {
  if (!raw || typeof raw !== "object") return null;
  const record = raw as Record<string, unknown>;
  const client = asString(record.client) ?? asString(record.name);
  if (!client) return null;

  let connection: McpAgentConnection = "unknown";
  if (record.connected === true) connection = "connected";
  else if (record.connected === false) connection = "disconnected";

  return {
    client,
    connection,
    connected_at: asString(record.connected_at) ?? undefined,
    last_activity_at:
      asString(record.last_activity_at) ?? asString(record.last_activity_utc) ?? undefined,
    idle_seconds: asNumber(record.idle_seconds) ?? undefined,
    protocol_version: asString(record.protocol_version) ?? undefined,
  };
}

/**
 * Normaliza el payload del backend sin promover lo desconocido a "activo".
 *
 * Si el payload no trae `running`, el servidor se considera inactivo: ausencia
 * de evidencia de escucha no es evidencia de escucha.
 */
export function normalizeStatus(raw: unknown): McpServerStatus {
  const record = (raw && typeof raw === "object" ? raw : {}) as Record<string, unknown>;

  const tools = Array.isArray(record.tools)
    ? record.tools.map(normalizeTool).filter((tool): tool is McpTool => tool !== null)
    : [];

  const apps = Array.isArray(record.apps)
    ? record.apps.map(normalizeApp).filter((app): app is McpApp => app !== null)
    : [];

  const agents = Array.isArray(record.agents)
    ? record.agents.map(normalizeAgent).filter((agent): agent is McpAgent => agent !== null)
    : [];

  const agentConnectedRaw = record.agent_connected;
  const agentConnected =
    agentConnectedRaw === null || agentConnectedRaw === undefined
      ? null
      : asBoolean(agentConnectedRaw);

  const running = asBoolean(record.running);

  return {
    running,
    transport: asString(record.transport) ?? (running ? "http" : "none"),
    endpoint: asString(record.endpoint),
    host: asString(record.host),
    port: asNumber(record.port),
    protocol_version: asString(record.protocol_version),
    api_key_required: asBoolean(record.api_key_required),
    tool_count: asNumber(record.tool_count) ?? tools.length,
    tools,
    apps,
    agents,
    agent_connected: agentConnected,
    started_at: asString(record.started_at),
    error: asString(record.error),
  };
}

/** Agrupa de herramientas por aplicación, para la presentación del panel. */
export interface McpToolGroup {
  id: string;
  name: string;
  availability: McpAppAvailability;
  detail?: string;
  tools: McpTool[];
}

/**
 * Agrupa las herramientas publicadas por la aplicación a la que pertenecen.
 *
 * Las aplicaciones que el backend declara sin herramientas se conservan: su
 * disponibilidad sigue siendo informacion relevante aunque no publiquen nada.
 */
export function groupToolsByApp(status: McpServerStatus): McpToolGroup[] {
  const map = new Map<string, McpToolGroup>();

  for (const app of status.apps) {
    map.set(app.id, {
      id: app.id,
      name: app.name || MCP_APP_LABELS[app.id] || app.id,
      availability: app.availability,
      detail: app.detail,
      tools: [],
    });
  }

  for (const tool of status.tools) {
    let group = map.get(tool.app_id);
    if (!group) {
      group = {
        id: tool.app_id,
        name: MCP_APP_LABELS[tool.app_id] || tool.app_id,
        availability: "unknown",
        tools: [],
      };
      map.set(tool.app_id, group);
    }
    group.tools.push(tool);
  }

  return Array.from(map.values());
}

/** Resumen del estado de conexión de los agentes. */
export interface McpAgentSummary {
  tone: McpAgentConnection;
  label: string;
  entries: McpAgent[];
  note: string;
}

/**
 * Resume el estado de los clientes de agente.
 *
 * Regla central: que el servidor esté encendido NO significa que haya un
 * agente conectado. Sin evidencia explícita en el payload, el estado es
 * "desconocido", nunca "conectado".
 */
export function describeAgentState(status: McpServerStatus | null): McpAgentSummary {
  if (!status) {
    return {
      tone: "unknown",
      label: "Desconocido",
      entries: [],
      note: "Desconocido. Sin el dato del backend no se puede afirmar que haya un agente conectado.",
    };
  }

  if (status.agents.length > 0) {
    const connected = status.agents.some((agent) => agent.connection === "connected");
    return {
      tone: connected ? "connected" : "unknown",
      label: connected ? "Conectado" : "Sin confirmar",
      entries: status.agents,
      note: connected
        ? "«Conectado» significa que el cliente hizo una petición MCP aceptada dentro de la ventana de inactividad; no que el socket siga abierto."
        : "",
    };
  }

  if (status.agent_connected === false) {
    return {
      tone: "disconnected",
      label: "Ninguno",
      entries: [],
      note: "No hay ningún cliente de agente conectado al puente.",
    };
  }

  if (status.agent_connected === true) {
    return {
      tone: "connected",
      label: "Conectado",
      entries: [],
      note: "El backend informa de un agente conectado, pero no detalla cuál.",
    };
  }

  return {
    tone: "unknown",
    label: "Desconocido",
    entries: [],
    note: "Sin evidencia de agente conectado: el backend no informa de sesiones MCP vivas.",
  };
}

/**
 * Consulta el estado del puente MCP.
 *
 * Devuelve siempre una vista con estado explícito; nunca datos simulados.
 */
export async function fetchMcpStatus(): Promise<McpStatusView> {
  try {
    const raw = await invoke<unknown>(MCP_STATUS_COMMAND);
    const status = normalizeStatus(raw);

    if (status.error && !status.running) {
      return {
        server_state: "error",
        message: status.error,
        status,
      };
    }

    return {
      server_state: status.running ? "active" : "inactive",
      message: status.running
        ? "El puente MCP está escuchando y responde peticiones."
        : "El puente MCP no está en marcha.",
      status,
    };
  } catch (error) {
    const failure = describeStatusFailure(error);
    return {
      server_state: failure.server_state,
      message: failure.message,
      status: null,
    };
  }
}
