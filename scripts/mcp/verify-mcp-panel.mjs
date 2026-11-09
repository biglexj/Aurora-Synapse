// Verificacion del panel de estado del MCP (frontera de la interfaz).
//
// Comprueba el contrato de datos que consume el panel: normalizacion del
// payload del backend, agrupacion de herramientas por aplicacion, la regla que
// prohibe afirmar "agente conectado" sin evidencia y la degradacion clara
// cuando el comando nativo aun no existe.
//
// Ejecutar: bun scripts/mcp/verify-mcp-panel.mjs
import assert from 'node:assert/strict';
import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';

import {
  describeAgentState,
  describeStatusFailure,
  groupToolsByApp,
  normalizeStatus,
} from '../../src/features/mcp/mcpService.ts';
import {
  describeConnection,
  readAppConnections,
} from '../../src/features/mcp/appConnections.ts';
import { McpStatusPanelBody } from '../../src/features/mcp/McpStatusPanel.tsx';
import { inferAppId } from '../../src/features/mcp/types.ts';

let checks = 0;
function check(name, fn) {
  fn();
  checks += 1;
  console.log('OK  -', name);
}

// ── Normalizacion del payload ───────────────────────────────────────────────
const full = normalizeStatus({
  running: true,
  transport: 'http',
  endpoint: 'http://127.0.0.1:49296/mcp',
  host: '127.0.0.1',
  port: 49296,
  protocol_version: '2025-06-18',
  api_key_required: false,
  tool_count: 3,
  tools: [
    { name: 'gallery_list', description: 'Lista la galeria', annotations: { readOnlyHint: true } },
    { name: 'luna_status' },
    { name: 'upscaler_scale', annotations: { destructiveHint: true } },
  ],
  apps: [
    { id: 'super_gallery', name: 'Super Galeria', supported: true, reachable: true },
    { id: 'luna_fetch', name: 'Luna Fetch', supported: true, reachable: false },
    { id: 'prisma', name: 'Prisma' },
  ],
  agents: [{ client: 'Ely Intelligence', connected: true, protocol_version: '2025-06-18' }],
  agent_connected: true,
  started_at: '2026-10-23T10:00:00Z',
});

check('conserva running y endpoint', () => {
  assert.equal(full.running, true);
  assert.equal(full.endpoint, 'http://127.0.0.1:49296/mcp');
  assert.equal(full.port, 49296);
});

check('lee readOnlyHint y destructiveHint de annotations', () => {
  assert.equal(full.tools[0].read_only, true);
  assert.equal(full.tools[2].destructive, true);
});

check('infiere la aplicacion de la herramienta por prefijo', () => {
  assert.equal(full.tools[0].app_id, 'super_gallery');
  assert.equal(full.tools[1].app_id, 'luna_fetch');
  assert.equal(full.tools[2].app_id, 'prisma_upscaler');
  assert.equal(inferAppId('prisma_play'), 'prisma');
  assert.equal(inferAppId('davinciflow_status'), 'davinci_flow');
  assert.equal(inferAppId('dispatch'), 'aurora_synapse');
});

check('tri-estado de disponibilidad de aplicaciones', () => {
  assert.equal(full.apps[0].availability, 'available');
  assert.equal(full.apps[1].availability, 'unavailable');
  assert.equal(full.apps[2].availability, 'unknown');
});

check('un payload vacio no se promueve a servidor activo', () => {
  const empty = normalizeStatus({});
  assert.equal(empty.running, false);
  assert.equal(empty.transport, 'none');
  assert.equal(empty.tool_count, 0);
  assert.equal(empty.agent_connected, null);
  assert.deepEqual(empty.agents, []);
});

// ── Agrupacion por aplicacion ───────────────────────────────────────────────
check('agrupa las herramientas por aplicacion', () => {
  const groups = groupToolsByApp(full);
  const gallery = groups.find((group) => group.id === 'super_gallery');
  const luna = groups.find((group) => group.id === 'luna_fetch');
  assert.equal(gallery.tools.length, 1);
  assert.equal(gallery.availability, 'available');
  assert.equal(luna.tools.length, 1);
  assert.equal(luna.availability, 'unavailable');
});

// ── Regla del agente conectado ──────────────────────────────────────────────
check('servidor activo sin sesiones no afirma agente conectado', () => {
  const summary = describeAgentState(normalizeStatus({ running: true }));
  assert.equal(summary.tone, 'unknown');
  assert.notEqual(summary.label, 'Conectado');
});

check('sin payload el estado del agente es desconocido', () => {
  const summary = describeAgentState(null);
  assert.equal(summary.tone, 'unknown');
  assert.match(summary.note, /no se puede afirmar/i);
});

check('agent_connected false se muestra como ninguno', () => {
  const summary = describeAgentState(normalizeStatus({ running: true, agent_connected: false }));
  assert.equal(summary.tone, 'disconnected');
  assert.equal(summary.label, 'Ninguno');
});

check('un agente listado con connected true se muestra conectado', () => {
  const summary = describeAgentState(
    normalizeStatus({ running: true, agents: [{ client: 'Ely Intelligence', connected: true }] }),
  );
  assert.equal(summary.tone, 'connected');
  assert.equal(summary.entries.length, 1);
});

check('un agente listado desconectado no se muestra como conectado', () => {
  const summary = describeAgentState(
    normalizeStatus({ running: true, agents: [{ client: 'Agente de terminal', connected: false }] }),
  );
  assert.equal(summary.entries[0].connection, 'disconnected');
  assert.notEqual(summary.label, 'Conectado');
});

// ── Actividad reciente de la sesion ─────────────────────────────────────────
check('el agente expone la marca de su ultima peticion', () => {
  const status = normalizeStatus({
    running: true,
    agents: [{
      client: 'Ely Intelligence',
      connected: true,
      connected_at: '2026-10-27T09:00:00.000Z',
      last_activity_at: '2026-10-27T09:04:30.000Z',
      idle_seconds: 12,
    }],
  });
  assert.equal(status.agents[0].last_activity_at, '2026-10-27T09:04:30.000Z');
  assert.equal(status.agents[0].idle_seconds, 12);
});

check('un agente sin datos de actividad los deja como desconocidos', () => {
  const status = normalizeStatus({
    running: true,
    agents: [{ client: 'Agente de terminal', connected: true }],
  });
  assert.equal(status.agents[0].idle_seconds, undefined);
  assert.equal(status.agents[0].last_activity_at, undefined);
});

check('la nota de conectado aclara que es actividad reciente', () => {
  const summary = describeAgentState(
    normalizeStatus({ running: true, agents: [{ client: 'Ely Intelligence', connected: true }] }),
  );
  assert.match(summary.note, /actividad|reciente/i);
});

// ── Degradacion sin backend ──────────────────────────────────────────────────
check('comando inexistente degrada a pendiente de backend', () => {
  const failure = describeStatusFailure('Command mcp_server_status not found');
  assert.equal(failure.server_state, 'unavailable');
  assert.match(failure.message, /Pendiente de backend/i);
});

check('sin runtime de Tauri degrada a no disponible', () => {
  const failure = describeStatusFailure(new Error('window is not defined'));
  assert.equal(failure.server_state, 'unavailable');
  assert.match(failure.message, /Sin runtime de Tauri/i);
});

check('un fallo inesperado se informa como error', () => {
  const failure = describeStatusFailure(new Error('timeout de red'));
  assert.equal(failure.server_state, 'error');
  assert.match(failure.message, /timeout de red/);
});

// ── Presentacion con datos reales ───────────────────────────────────────────
const populated = normalizeStatus({
  running: true,
  transport: 'http',
  endpoint: 'http://127.0.0.1:49296/mcp',
  protocol_version: '2025-06-18',
  tool_count: 20,
  tools: [
    { name: 'gallery_list', description: 'Lista la galeria', annotations: { readOnlyHint: true } },
    { name: 'luna_status' },
    { name: 'upscaler_scale', annotations: { destructiveHint: true } },
  ],
  apps: [
    { id: 'super_gallery', name: 'Super Galeria', supported: true, reachable: true },
    { id: 'luna_fetch', name: 'Luna Fetch', supported: true, reachable: false },
  ],
  agents: [],
  agent_connected: null,
});

const html = renderToStaticMarkup(
  createElement(McpStatusPanelBody, {
    view: {
      server_state: 'active',
      message: 'El puente MCP esta escuchando y responde peticiones.',
      status: populated,
    },
    isMobile: true,
    isRefreshing: false,
    onRefresh: () => {},
    onClose: () => {},
  }),
);

check('la vista muestra la direccion del servidor', () => {
  assert.match(html, /127\.0\.0\.1:49296/);
});

check('la vista agrupa las herramientas por aplicacion', () => {
  assert.match(html, /Super Galeria/);
  assert.match(html, /Luna Fetch/);
  assert.match(html, /gallery_list/);
});

check('la vista no nombra a ningun agente sin evidencia', () => {
  assert.doesNotMatch(html, /Ely Intelligence/);
  assert.match(html, /Desconocido/);
});

// ── Presentacion de la actividad del agente ─────────────────────────────────
const conAgente = renderToStaticMarkup(
  createElement(McpStatusPanelBody, {
    view: {
      server_state: 'active',
      message: 'El puente MCP esta escuchando y responde peticiones.',
      status: normalizeStatus({
        running: true,
        agents: [{
          client: 'Ely Intelligence',
          connected: true,
          connected_at: '2026-10-27T09:00:00.000Z',
          idle_seconds: 125,
        }],
        agent_connected: true,
      }),
    },
    isMobile: false,
    isRefreshing: false,
    onRefresh: () => {},
    onClose: () => {},
  }),
);

check('la vista nombra al agente con sesion viva', () => {
  assert.match(conAgente, /Ely Intelligence/);
});

check('la vista informa de hace cuanto hablo el agente', () => {
  assert.match(conAgente, /ltima petici/);
  assert.match(conAgente, /hace 2 min/);
});

// ── Conexion en vivo de las apps del catalogo ───────────────────────────────
const connected = normalizeStatus({
  running: true,
  tools: [
    { name: 'gallery_list', annotations: { readOnlyHint: true } },
    { name: 'luna_status' },
  ],
  apps: [
    { id: 'super_gallery', name: 'Super Galeria', supported: true, reachable: true, availability: 'available' },
    { id: 'luna_fetch', name: 'Luna Fetch', supported: true, reachable: true, availability: 'available' },
    { id: 'prisma', name: 'Prisma', supported: true, reachable: false, availability: 'offline' },
    { id: 'gallery_dl', name: 'Gallery-DL GUI', supported: false, reachable: true, availability: 'available' },
  ],
});

const connections = readAppConnections(connected);

check('el identificador del catalogo se relaciona con el del puente', () => {
  assert.equal(connections.lienzo.connected, true);
  assert.equal(connections.luna.connected, true);
  assert.equal(connections['prisma-upscaler']?.connected, undefined);
});

check('una app conectada cuenta las herramientas MCP que publica', () => {
  assert.equal(connections.lienzo.mcpTools, true);
  assert.equal(connections.lienzo.toolCount, 1);
  assert.equal(connections.luna.mcpTools, true);
});

check('una app conectada sin herramientas MCP no se marca como MCP', () => {
  assert.equal(connections.gallerydl.connected, true);
  assert.equal(connections.gallerydl.mcpTools, false);
  assert.equal(connections.gallerydl.toolCount, 0);
});

check('una app que no responde no se marca como conectada', () => {
  assert.equal(connections.prisma.connected, false);
});

check('sin payload no hay conexiones que pintar', () => {
  assert.deepEqual(readAppConnections(null), {});
});

check('el texto del badge distingue capacidad MCP de simple conexion', () => {
  assert.match(describeConnection(connections.lienzo), /publica 1 herramienta/);
  assert.match(describeConnection(connections.gallerydl), /todavía no publica herramientas MCP/);
});

console.log(`\n${checks} comprobaciones correctas en total.`);
