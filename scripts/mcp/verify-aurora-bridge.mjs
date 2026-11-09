// Verificacion del enlace Aurora -> Synapse (puente `synapse_bridge`).
//
// Levanta DOS procesos propios y los para al terminar:
//   1. un puente MCP de Aurora Synapse en un puerto loopback temporal,
//      con una clave efimera;
//   2. el backend Rust de Aurora, en loopback (APP_HOST=127.0.0.1) y en un
//      puerto propio, apuntando AURORA_SYNAPSE_MCP_URL al puente anterior.
//
// Comprueba que el backend de Aurora:
//   - descubre el puente (tools/list con prefijo `synapse.`),
//   - lo marca `connected` tras un refresh exitoso,
//   - y puede invocar una herramienta de solo lectura a traves de el.
//
// No toca el MCP abierto por la app instalada del usuario, no descarga nada,
// no muta apps y no imprime credenciales.
//
// Uso: bun scripts/mcp/verify-aurora-bridge.mjs
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createServer } from 'node:net';
import { readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { randomBytes } from 'node:crypto';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const blog = resolve(root, '../Aurora---Blog/backend');
const synapseBinary = process.env.AURORA_SYNAPSE_BIN
  ? resolve(process.env.AURORA_SYNAPSE_BIN)
  : resolve(root, 'src-tauri/target/debug/aurora-synapse.exe');
const backendBinary = join(blog, 'target/debug/aurora-backend.exe');

// ── Credenciales: se leen de los .env del backend y nunca se imprimen.
const environment = { ...process.env };
for (const path of [join(blog, '.env'), join(blog, 'target/debug/.env')]) {
  const contents = await readFile(path, 'utf8').catch(() => '');
  for (const line of contents.split(/\r?\n/)) {
    const match = line.match(/^\s*([A-Z][A-Z0-9_]*)=(.*)$/);
    if (match && environment[match[1]] === undefined) {
      environment[match[1]] = match[2].trim().replace(/^(['"])(.*)\1$/, '$2');
    }
  }
}
const technicalKey = (environment.ELY_INTELLIGENCE_SHARED_SECRET ?? '').trim();
assert.ok(technicalKey.length >= 32, 'Falta la credencial tecnica local del backend de Aurora');

/** Puerto libre en loopback. */
async function freePort() {
  const server = createServer();
  await new Promise(done => server.listen(0, '127.0.0.1', done));
  const port = server.address().port;
  await new Promise(done => server.close(done));
  return port;
}

const sleep = ms => new Promise(done => setTimeout(done, ms));
const children = [];

function stopAll() {
  for (const child of children) {
    if (child.exitCode === null) {
      try { child.kill(); } catch { /* ya terminado */ }
    }
  }
}

try {
  // ── 1. Puente MCP de Synapse, propio y temporal ────────────────────────
  const synapsePort = await freePort();
  const synapseKey = randomBytes(32).toString('hex');
  const synapse = spawn(synapseBinary, ['--mcp-http'], {
    env: {
      ...process.env,
      AURORA_SYNAPSE_MCP_ADDR: `127.0.0.1:${synapsePort}`,
      AURORA_SYNAPSE_MCP_KEY: synapseKey,
    },
    windowsHide: true,
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  children.push(synapse);
  let synapseLog = '';
  synapse.stderr.on('data', data => { synapseLog += data.toString(); });
  synapse.stdout.resume();

  let bridgeReady = false;
  for (let attempt = 0; attempt < 100; attempt++) {
    try {
      const response = await fetch(`http://127.0.0.1:${synapsePort}/status`, {
        signal: AbortSignal.timeout(500),
      });
      if (response.ok) { bridgeReady = true; break; }
    } catch { /* aun arrancando */ }
    await sleep(100);
  }
  assert.ok(bridgeReady, `El puente MCP no arranco: ${synapseLog.slice(0, 300)}`);

  // ── 2. Backend de Aurora en loopback, apuntando a ese puente ────────────
  const backendPort = await freePort();
  const backend = spawn(backendBinary, [], {
    env: {
      ...process.env,
      APP_HOST: '127.0.0.1',
      PORT: String(backendPort),
      AURORA_SYNAPSE_MCP_URL: `http://127.0.0.1:${synapsePort}/mcp`,
      AURORA_SYNAPSE_MCP_KEY: synapseKey,
    },
    cwd: blog,
    windowsHide: true,
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  children.push(backend);
  let backendLog = '';
  backend.stderr.on('data', data => { backendLog += data.toString(); });
  backend.stdout.resume();

  const headers = {
    'x-ely-intelligence-key': technicalKey,
    'X-Aurora-Principal': 'synapse-verificacion-puente',
    'Content-Type': 'application/json',
  };
  const base = `http://127.0.0.1:${backendPort}/api/v1/mcp`;

  let backendReady = false;
  for (let attempt = 0; attempt < 200; attempt++) {
    if (backend.exitCode !== null) break;
    try {
      const response = await fetch(`${base}/synapse/status`, {
        headers,
        signal: AbortSignal.timeout(1000),
      });
      if (response.ok) { backendReady = true; break; }
    } catch { /* aun arrancando */ }
    await sleep(250);
  }
  assert.ok(backendReady, `El backend de Aurora no respondio: ${backendLog.slice(0, 300)}`);

  // ── 3. El puente se descubre y se marca conectado ───────────────────────
  let status = null;
  for (let attempt = 0; attempt < 40; attempt++) {
    const response = await fetch(`${base}/synapse/status`, { headers, signal: AbortSignal.timeout(5000) });
    status = await response.json();
    if (status.configured && status.connected) break;
    await sleep(1000);
  }
  assert.equal(status.configured, true, 'el puente debe estar configurado');
  assert.equal(status.connected, true, `el puente debe quedar conectado: ${JSON.stringify(status)}`);
  assert.equal(status.tools, 20, `se esperaban 20 herramientas del puente: ${status.tools}`);

  const tools = await (await fetch(`${base}/tools`, { headers, signal: AbortSignal.timeout(5000) })).json();
  const synapseTools = tools.filter(tool => tool.name.startsWith('synapse.'));
  assert.ok(synapseTools.length >= 20, `el control plane debe publicar el catalogo del puente: ${synapseTools.length}`);

  // ── 4. Invocacion de solo lectura a traves del puente ───────────────────
  const invoked = await (await fetch(`${base}/invoke`, {
    method: 'POST',
    headers,
    body: JSON.stringify({ name: 'synapse.apps_list', arguments: {} }),
    signal: AbortSignal.timeout(30_000),
  })).json();
  assert.equal(invoked.status, 'executed', `la invocacion no se ejecuto: ${JSON.stringify(invoked).slice(0, 300)}`);
  assert.equal(invoked.result?.ok, true, 'la invocacion devolvio un error');

  // ── 5. Estado del Host (Aurora Hub) visto por el backend ───────────────
  const hosts = await (await fetch(`${base}/hosts/status`, {
    headers,
    signal: AbortSignal.timeout(5000),
  })).json();

  console.log(JSON.stringify({
    result: 'ok',
    enlace: 'Aurora (backend local, loopback) -> Synapse (puente propio, loopback)',
    synapse_bridge: {
      configured: status.configured,
      connected: status.connected,
      tools: status.tools,
      url: status.url,
      last_error: status.last_error ?? null,
    },
    catalogo_control_plane: {
      total: tools.length,
      con_prefijo_synapse: synapseTools.length,
    },
    invocacion: { name: 'synapse.apps_list', status: invoked.status },
    aurora_hub: { connected: hosts.connected, ready: hosts.ready },
  }, null, 2));
} finally {
  stopAll();
}
