// Cliente de agente REAL contra el puente MCP de Aurora Synapse.
//
// A diferencia de verify-foundation.mjs (que habla JSON-RPC con su propia
// sonda), aquí el servidor MCP se registra como cliente nativo de un agente de
// verdad (Claude Code) por stdio: las herramientas aparecen en el chat como
// herramientas MCP, no como texto.
//
// Requisitos:
//   - binario compilado (cargo build --bin aurora-synapse)
//   - CLI `claude` disponible y con credenciales
//
// Uso:
//   bun scripts/mcp/verify-agent-client.mjs
//   AURORA_SYNAPSE_BIN=<ruta> bun scripts/mcp/verify-agent-client.mjs
//
// Solo lectura: la lista de herramientas permitidas no incluye ninguna
// herramienta que despierte una acción sobre las apps.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const binary = process.env.AURORA_SYNAPSE_BIN
  ? resolve(process.env.AURORA_SYNAPSE_BIN)
  : resolve(root, 'src-tauri/target/debug/aurora-synapse.exe');

const workspace = mkdtempSync(join(tmpdir(), 'synapse-agente-real-'));
const configPath = join(workspace, 'mcp.json');
writeFileSync(
  configPath,
  JSON.stringify(
    {
      mcpServers: {
        'aurora-synapse': {
          command: binary.replace(/\\/g, '/'),
          args: ['--mcp-stdio'],
        },
      },
    },
    null,
    2,
  ),
  'utf8',
);

const prompt = [
  "Conéctate al servidor MCP 'aurora-synapse' y haz SOLO esto, sin usar ninguna otra herramienta:",
  "1) lista los recursos MCP del servidor con la herramienta de listar recursos;",
  "2) lee el recurso 'aurora-synapse://capabilities';",
  "3) llama a la herramienta 'classify_intent' con el argumento",
  "   input='https://www.youtube.com/watch?v=synapse_real_client'.",
  "Termina con una línea 'EVIDENCIA:' seguida de (a) el número de herramientas del catálogo,",
  "(b) el valor de session.idle_timeout_seconds leído del recurso y",
  "(c) el recommended_app_id devuelto.",
].join(' ');

const allowed = [
  'mcp__aurora-synapse__apps_list',
  'mcp__aurora-synapse__classify_intent',
  'mcp__aurora-synapse__ListMcpResourcesTool',
  'mcp__aurora-synapse__ReadMcpResourceTool',
];

let run;
try {
  run = spawnSync(
    'claude',
    [
      '-p', prompt,
      '--mcp-config', configPath,
      '--strict-mcp-config',
      '--permission-mode', 'dontAsk',
      '--allowedTools', ...allowed,
      '--output-format', 'stream-json',
      '--verbose',
    ],
    { encoding: 'utf8', windowsHide: true, timeout: 300_000, maxBuffer: 16 * 1024 * 1024 },
  );
  assert.ok(run.stdout, `El cliente no produjo salida: ${run.stderr}`);
} finally {
  rmSync(workspace, { recursive: true, force: true });
}

const lines = run.stdout.trim().split(/\r?\n/);
const events = lines
  .map(line => { try { return JSON.parse(line); } catch { return null; } })
  .filter(event => event !== null);

const toolUses = [];
for (const event of events) {
  const content = event.message?.content;
  if (!Array.isArray(content)) continue;
  for (const block of content) {
    if (block.type === 'tool_use') toolUses.push({ name: block.name, input: block.input });
  }
}
const result = events.find(event => event.type === 'result');
const answer = result?.result ?? '';

const names = toolUses.map(use => use.name);
assert.ok(
  names.includes('mcp__aurora-synapse__classify_intent'),
  `El agente no llamó a la herramienta nativa del puente: ${names.join(', ')}`,
);
assert.ok(
  names.includes('ReadMcpResourceTool'),
  `El agente no leyó el recurso de capacidades de forma nativa: ${names.join(', ')}`,
);
assert.match(answer, /EVIDENCIA/);
assert.match(answer, /300/, 'La respuesta debe incluir session.idle_timeout_seconds');

console.log(
  JSON.stringify(
    {
      result: 'ok',
      cliente: 'claude (agente real, transporte stdio)',
      herramientas_nativas_usadas: names,
      recurso_leido: toolUses.find(use => use.name === 'ReadMcpResourceTool')?.input?.uri,
      evidencia: answer.split('\n').filter(line => line.startsWith('EVIDENCIA')).join(' '),
    },
    null,
    2,
  ),
);
