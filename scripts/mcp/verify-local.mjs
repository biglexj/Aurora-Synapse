// Prueba del binario real por HTTP y stdio. No abre ni reinicia la GUI.
// Usa Prisma Upscaler CLI instalada y FSR1 para crear un PNG nuevo en %TEMP%.
// Ejecutar después de cargo build: bun scripts/mcp/verify-local.mjs
import { spawn, spawnSync } from 'node:child_process';
import { mkdtemp, readFile, stat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'node:net';
import assert from 'node:assert/strict';

const root = fileURLToPath(new URL('../../', import.meta.url));
const binary = resolve(root, 'src-tauri/target/debug/aurora-synapse.exe');
const outputDir = await mkdtemp(join(tmpdir(), 'aurora-synapse-mcp-'));
const allocator = createServer();
await new Promise((done) => allocator.listen(0, '127.0.0.1', done));
const port = allocator.address().port;
await new Promise((done) => allocator.close(done));
const endpoint = `http://127.0.0.1:${port}/mcp`;
const env = { ...process.env, AURORA_SYNAPSE_MCP_ADDR: `127.0.0.1:${port}`, AURORA_SYNAPSE_MCP_KEY: '' };
const child = spawn(binary, ['--mcp-http'], { env, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
let failure = '';
let spawnError;
child.on('error', error => { spawnError = error; });
child.stderr.on('data', data => { failure += data.toString(); });
let session;
let requestId = 0;
async function rpc(method, params = {}) {
  const id = ++requestId;
  const response = await fetch(endpoint, {
    method: 'POST', headers: { 'Content-Type': 'application/json', 'Accept': 'application/json, text/event-stream', ...(session ? { 'Mcp-Session-Id': session } : {}) },
    body: JSON.stringify({ jsonrpc: '2.0', id, method, params }), signal: AbortSignal.timeout(140_000),
  });
  assert.equal(response.status, 200);
  session = response.headers.get('mcp-session-id') ?? session;
  const body = await response.json();
  assert.equal(body.id, id);
  assert.equal(body.error, undefined, JSON.stringify(body.error));
  return body.result;
}
async function call(name, arguments_ = {}) {
  return rpc('tools/call', { name, arguments: arguments_ });
}
try {
  let ready = false;
  for (let attempt = 0; attempt < 50; attempt++) {
    if (spawnError || child.exitCode !== null) throw Error(spawnError?.message || failure || `servidor terminado: ${child.exitCode}`);
    try {
      const status = await fetch(`${endpoint}/status`, { signal: AbortSignal.timeout(500) });
      ready = status.ok;
    } catch { /* arranque */ }
    if (ready) break;
    await new Promise(done => setTimeout(done, 100));
  }
  assert.ok(ready, 'El servidor no arrancó');
  const init = await rpc('initialize', { protocolVersion: '2025-06-18', capabilities: {}, clientInfo: { name: 'synapse-local-verification', version: '1' } });
  assert.equal(init.serverInfo.name, 'aurora-synapse');
  const notification = await fetch(endpoint, { method: 'POST', headers: { 'Content-Type': 'application/json', 'Mcp-Session-Id': session }, body: JSON.stringify({ jsonrpc: '2.0', method: 'notifications/initialized' }) });
  assert.equal(notification.status, 202);
  const catalog = await rpc('tools/list');
  assert.equal(catalog.tools.length, 20);
  const apps = await call('apps_list');
  assert.equal(apps.isError, false);
  for (const app of apps.structuredContent.capacidades_del_puente) {
    assert.equal(typeof app.supported, 'boolean');
    assert.equal(typeof app.reachable, 'boolean');
    if (!app.supported) assert.equal(app.reachable, false);
    if (app.availability === 'offline' || app.availability === 'cli_available') assert.equal(app.reachable, false);
  }
  console.log('Disponibilidad:', JSON.stringify(apps.structuredContent.capacidades_del_puente.map(({ id, supported, reachable, availability }) => ({ id, supported, reachable, availability }))));
  const prisma = await call('prisma_status');
  console.log('Prisma real:', JSON.stringify(prisma));
  assert.equal((await call('dispatch', { target_app: 'luna', action: 'enqueue_download', payload: {} })).isError, true);
  const output = join(outputDir, 'resultado FSR1.png');
  const input = resolve(root, 'src-tauri/icons/32x32.png');
  const scaled = await call('upscaler_scale', { input_path: input, output_path: output, engine: 'fsr1', scale: 2, sharpness: 0.5 });
  assert.equal(scaled.isError, false, JSON.stringify(scaled));
  assert.equal(scaled.structuredContent.completed, true);
  const bytes = await readFile(output);
  assert.equal(bytes.subarray(0, 8).toString('hex'), '89504e470d0a1a0a');
  assert.equal(bytes.readUInt32BE(16), 64);
  assert.equal(bytes.readUInt32BE(20), 64);
  assert.equal((await stat(output)).size, scaled.structuredContent.result.bytes);
  const repeated = await call('upscaler_scale', { input_path: input, output_path: output, engine: 'fsr1', scale: 2 });
  assert.equal(repeated.isError, true);
  assert.deepEqual(await readFile(output), bytes);
  console.log('Escalado real:', JSON.stringify(scaled.structuredContent));

  const before = await (await fetch(`${endpoint}/status`)).json();
  for (const shell of ['pwsh', 'powershell.exe']) {
    const probe = spawnSync(shell, ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', resolve(root, 'scripts/mcp/bridge-probe.ps1'), '-Url', endpoint, '-Tool', 'classify_intent', '-Arguments', '{"input":"https://example.com/video.mp4"}'], { env, windowsHide: true, encoding: 'utf8', timeout: 30_000 });
    assert.equal(probe.status, 0, `${shell}: ${probe.stderr}\n${probe.stdout}`);
    const rejected = spawnSync(shell, ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', resolve(root, 'scripts/mcp/bridge-probe.ps1'), '-Url', endpoint, '-Tool', 'dispatch', '-Arguments', '{"target_app":"luna","action":"enqueue_download","payload":{}}'], { env, windowsHide: true, encoding: 'utf8', timeout: 30_000 });
    assert.equal(rejected.status, 1, `${shell}: se esperaba fallo de herramienta`);
    console.log(`Sonda ${shell}: éxito, error y cierre de sesión comprobados`);
  }
  const after = await (await fetch(`${endpoint}/status`)).json();
  assert.equal(after.sessions, before.sessions, 'La sonda dejó sesiones abiertas');
  const stdio = spawnSync(binary, ['--mcp-stdio'], { env, windowsHide: true, encoding: 'utf8', timeout: 15_000, input: [
    { jsonrpc: '2.0', id: 1, method: 'initialize', params: { protocolVersion: '2025-06-18', clientInfo: { name: 'stdio-verification' } } },
    { jsonrpc: '2.0', method: 'notifications/initialized' },
    { jsonrpc: '2.0', id: 2, method: 'tools/list' },
  ].map(value => JSON.stringify(value)).join('\n') + '\n' });
  assert.equal(stdio.status, 0, stdio.stderr);
  const replies = stdio.stdout.trim().split(/\r?\n/).map(line => JSON.parse(line));
  assert.equal(replies.length, 2);
  assert.equal(replies[1].result.tools.length, 20);
  console.log(`HTTP y stdio comprobados. Artefacto: ${output}`);
} finally {
  if (session) await fetch(endpoint, { method: 'DELETE', headers: { 'Mcp-Session-Id': session }, signal: AbortSignal.timeout(2000) }).catch(() => {});
  const closed = child.exitCode === null ? new Promise(done => child.once('exit', done)) : Promise.resolve();
  child.kill();
  await closed;
}
