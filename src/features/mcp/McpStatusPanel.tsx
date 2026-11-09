import { useCallback, useEffect, useMemo, useState } from "react";
import { describeAgentState, fetchMcpStatus, groupToolsByApp } from "./mcpService";
import {
  McpAppAvailability,
  McpAgentConnection,
  McpServerState,
  McpStatusView,
} from "./types";

interface McpStatusPanelProps {
  isOpen: boolean;
  isMobile: boolean;
  onClose: () => void;
}

interface McpStatusPanelBodyProps {
  view: McpStatusView;
  isMobile: boolean;
  isRefreshing: boolean;
  onRefresh: () => void;
  onClose: () => void;
}

const SERVER_STATE_LABEL: Record<McpServerState, string> = {
  active: "Activo",
  inactive: "Inactivo",
  loading: "Consultando…",
  error: "Error",
  unavailable: "No disponible",
};

const AVAILABILITY_LABEL: Record<McpAppAvailability, string> = {
  available: "Disponible",
  unavailable: "No disponible",
  unknown: "Desconocido",
};

const AVAILABILITY_HINT: Record<McpAppAvailability, string> = {
  available: "Respondiendo ahora mismo a través del puente.",
  unavailable: "No disponible ahora mismo: no responde o el adaptador no la soporta.",
  unknown: "Sin datos suficientes para afirmar nada.",
};

const AGENT_CONNECTION_LABEL: Record<McpAgentConnection, string> = {
  connected: "Conectado",
  disconnected: "Desconectado",
  unknown: "Desconocido",
};

/**
 * Describe el tiempo desde la última petición aceptada del cliente.
 *
 * Sin este dato, "Conectado" parecería un socket permanente; con él, el
 * usuario ve si el cliente acaba de hablar o si lleva minutos en silencio.
 */
function describeIdle(idleSeconds: number | undefined): string | null {
  if (idleSeconds === undefined) return null;
  if (idleSeconds < 60) return `hace ${idleSeconds} s`;
  const minutes = Math.floor(idleSeconds / 60);
  if (minutes < 60) return `hace ${minutes} min`;
  const hours = Math.floor(minutes / 60);
  return `hace ${hours} h`;
}

/**
 * Cuerpo presentacional del panel.
 *
 * Se separa del contenedor para poder verificar los estados con datos reales
 * sin depender del runtime de Tauri.
 */
export function McpStatusPanelBody({
  view,
  isMobile,
  isRefreshing,
  onRefresh,
  onClose,
}: McpStatusPanelBodyProps) {
  const status = view.status;
  const groups = useMemo(() => (status ? groupToolsByApp(status) : []), [status]);
  const agents = useMemo(() => describeAgentState(status), [status]);

  const serverActive = view.server_state === "active";
  const serverNote = (() => {
    switch (view.server_state) {
      case "loading":
        return "Consultando…";
      case "unavailable":
        return "Sin datos del backend: no se puede afirmar que haya un agente conectado.";
      case "error":
        return "No se pudo consultar el estado del servidor MCP.";
      default:
        return "Servidor MCP inactivo: no puede haber un agente conectado.";
    }
  })();

  const agentsSection = () => {
    if (agents.entries.length === 0) {
      return <p className="mcp-empty-note">{agents.note}</p>;
    }
    return (
      <>
        <ul className="mcp-agent-list">
          {agents.entries.map((agent, index) => {
            const idle = describeIdle(agent.idle_seconds);
            return (
              <li key={`${agent.client}-${index}`} className="mcp-agent-row">
                <span className="mcp-agent-name">{agent.client}</span>
                <span className={`mcp-chip mcp-chip-agent-${agent.connection}`}>
                  {AGENT_CONNECTION_LABEL[agent.connection]}
                </span>
                {agent.protocol_version && (
                  <span className="mcp-agent-meta">protocolo {agent.protocol_version}</span>
                )}
                {idle && <span className="mcp-agent-meta">última petición {idle}</span>}
                {agent.connected_at && (
                  <span className="mcp-agent-meta">desde {agent.connected_at}</span>
                )}
              </li>
            );
          })}
        </ul>
        {agents.note && <p className="mcp-empty-note">{agents.note}</p>}
      </>
    );
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div
        className={`modal-content mcp-panel ${isMobile ? "mcp-panel-mobile" : ""}`}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <div className="modal-header-title">
            <span className="modal-icon">🔌</span>
            <h2>Estado del MCP</h2>
          </div>
          <div className="mcp-header-actions">
            <button
              className="mcp-refresh-btn"
              onClick={onRefresh}
              disabled={isRefreshing}
              title="Volver a consultar el puente MCP"
            >
              {isRefreshing ? "Actualizando…" : "Actualizar"}
            </button>
            <button className="modal-close-btn" onClick={onClose} aria-label="Cerrar">
              ✕
            </button>
          </div>
        </div>

        <div className="modal-body">
          <p className="mcp-intro">
            Tres conceptos distintos: que el <strong>servidor</strong> esté encendido no implica que una{" "}
            <strong>aplicación</strong> esté disponible, ni que haya un <strong>agente</strong> conectado.
          </p>

          {/* 1. Servidor MCP */}
          <section className="mcp-block">
            <div className="mcp-block-head">
              <h3 className="mcp-block-title">Servidor MCP</h3>
              <span className={`mcp-chip mcp-chip-server-${view.server_state}`}>
                {SERVER_STATE_LABEL[view.server_state]}
              </span>
            </div>
            <p className="mcp-block-note">{view.message}</p>

            <dl className="mcp-facts">
              <div className="mcp-fact">
                <dt>Dirección</dt>
                <dd>{status?.endpoint || (status?.host ? `${status.host}:${status.port ?? "?"}` : "—")}</dd>
              </div>
              <div className="mcp-fact">
                <dt>Transporte</dt>
                <dd>{status?.transport || "—"}</dd>
              </div>
              <div className="mcp-fact">
                <dt>Herramientas</dt>
                <dd>{status ? status.tool_count : "—"}</dd>
              </div>
              <div className="mcp-fact">
                <dt>Protocolo</dt>
                <dd>{status?.protocol_version || "—"}</dd>
              </div>
              {status?.started_at && (
                <div className="mcp-fact">
                  <dt>Arranque</dt>
                  <dd>{status.started_at}</dd>
                </div>
              )}
            </dl>
          </section>

          {/* 2. Aplicaciones y herramientas */}
          <section className="mcp-block">
            <div className="mcp-block-head">
              <h3 className="mcp-block-title">Aplicaciones y herramientas</h3>
              <span className="mcp-block-count">{groups.length} grupos</span>
            </div>

            {!status ? (
              <p className="mcp-empty-note">
                Sin datos del backend. No se puede afirmar qué aplicaciones están disponibles.
              </p>
            ) : groups.length === 0 ? (
              <p className="mcp-empty-note">El puente no publica herramientas.</p>
            ) : (
              <ul className="mcp-app-list">
                {groups.map((group) => (
                  <li key={group.id} className="mcp-app-row">
                    <div className="mcp-app-head">
                      <span className="mcp-app-name">{group.name}</span>
                      <span className={`mcp-chip mcp-chip-app-${group.availability}`}>
                        {AVAILABILITY_LABEL[group.availability]}
                      </span>
                    </div>
                    <p className="mcp-app-detail">
                      {group.detail || AVAILABILITY_HINT[group.availability]}
                    </p>
                    {group.tools.length > 0 && (
                      <ul className="mcp-tool-list">
                        {group.tools.map((tool) => (
                          <li key={tool.name} className="mcp-tool-row">
                            <code className="mcp-tool-name">{tool.name}</code>
                            <span className="mcp-tool-tags">
                              {tool.read_only && <span className="mcp-tag">solo lectura</span>}
                              {tool.destructive && <span className="mcp-tag mcp-tag-destructive">destructiva</span>}
                            </span>
                            {tool.description && (
                              <span className="mcp-tool-desc">{tool.description}</span>
                            )}
                          </li>
                        ))}
                      </ul>
                    )}
                  </li>
                ))}
              </ul>
            )}
          </section>

          {/* 3. Agentes conectados */}
          <section className="mcp-block">
            <div className="mcp-block-head">
              <h3 className="mcp-block-title">Agentes conectados</h3>
              <span className={`mcp-chip mcp-chip-agent-${agents.tone}`}>{agents.label}</span>
            </div>
            {serverActive ? (
              agentsSection()
            ) : (
              <p className="mcp-empty-note">{serverNote}</p>
            )}
          </section>
        </div>
      </div>
    </div>
  );
}

export function McpStatusPanel({ isOpen, isMobile, onClose }: McpStatusPanelProps) {
  const [view, setView] = useState<McpStatusView>({
    server_state: "loading",
    message: "Consultando el puente MCP…",
    status: null,
  });
  const [isRefreshing, setIsRefreshing] = useState<boolean>(false);

  const refresh = useCallback(async () => {
    setIsRefreshing(true);
    setView({ server_state: "loading", message: "Consultando el puente MCP…", status: null });
    const next = await fetchMcpStatus();
    setView(next);
    setIsRefreshing(false);
  }, []);

  useEffect(() => {
    if (!isOpen) return;
    void refresh();
  }, [isOpen, refresh]);

  if (!isOpen) return null;

  return (
    <McpStatusPanelBody
      view={view}
      isMobile={isMobile}
      isRefreshing={isRefreshing}
      onRefresh={() => void refresh()}
      onClose={onClose}
    />
  );
}
