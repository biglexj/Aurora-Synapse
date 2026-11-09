// Conexión real de solo lectura: HTTP, stdio, catálogo común y sesiones.
// No descarga, escala, instala ni abre apps. Requiere cargo build reciente.
//
// AURORA_SYNAPSE_BIN permite apuntar a otro binario (por ejemplo una
// compilación en un directorio de destino propio) sin tocar este guion.
//
// `--slow` añade la caducidad real de la ventana de inactividad (unos 5 min):
// espera más de 300 s, comprueba que el sondeo de estado no renueva, que una
// petición aceptada sí renueva y que initialize recupera la sesión tras 404.
import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { createServer } from 'node:net';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';
import { randomBytes } from 'node:crypto';

const root = fileURLToPath(new URL('../../', import.meta.url));
const binary = process.env.AURORA_SYNAPSE_BIN
  ? resolve(process.env.AURORA_SYNAPSE_BIN)
  : resolve(root, 'src-tauri/target/debug/aurora-synapse.exe');
const allocator = createServer();
await new Promise(done => allocator.listen(0, '127.0.0.1', done));
const port = allocator.address().port;
await new Promise(done => allocator.close(done));
const endpoint = `http://127.0.0.1:${port}/mcp`;
const key = randomBytes(32).toString('hex');
const child = spawn(binary, ['--mcp-http'], {
  env: { ...process.env, AURORA_SYNAPSE_MCP_ADDR: `127.0.0.1:${port}`, AURORA_SYNAPSE_MCP_KEY: key },
  windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'],
});
let failure = '';
let spawnError;
child.on('error', error => { spawnError = error; });
child.stderr.on('data', data => { failure += data.toString(); });
child.stdout.resume();
let session;
let id = 0;
const headers = (extra = {}) => ({
  'Content-Type': 'application/json', Accept: 'application/json, text/event-stream',
  Authorization: `Bearer ${key}`, ...(session ? { 'Mcp-Session-Id': session } : {}), ...extra,
});
/** POST en bruto: devuelve estado, cabeceras y cuerpo sin interpretar. */
async function post(body, extra = {}) {
  const response = await fetch(endpoint, {
    method: 'POST', headers: headers(extra),
    body: JSON.stringify(body),
    signal: AbortSignal.timeout(15_000),
  });
  const text = await response.text();
  let parsed = null;
  try { parsed = JSON.parse(text); } catch { /* respuesta sin cuerpo JSON */ }
  return { status: response.status, headers: response.headers, body: parsed };
}
async function rpc(method, params = {}) {
  const requestId = ++id;
  const response = await post({ jsonrpc: '2.0', id: requestId, method, params });
  assert.equal(response.status, 200, JSON.stringify(response.body));
  session = response.headers.get('mcp-session-id') ?? session;
  assert.equal(response.body.id, requestId);
  assert.equal(response.body.error, undefined, JSON.stringify(response.body.error));
  return response.body.result;
}
const wait = ms => new Promise(done => setTimeout(done, ms));
const uri = 'aurora-synapse://capabilities';
try {
  let ready = false;
  for (let attempt = 0; attempt < 60; attempt++) {
    if (spawnError || child.exitCode !== null) throw Error(spawnError?.message || failure || 'Puente terminado');
    try { ready = (await fetch(`${endpoint}/status`, { signal: AbortSignal.timeout(500) })).ok; } catch {}
    if (ready) break;
    await new Promise(done => setTimeout(done, 100));
  }
  assert.ok(ready, 'El puente no arrancó');
  const denied = await fetch(endpoint, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ jsonrpc: '2.0', id: 0, method: 'resources/list' }) });
  assert.equal(denied.status, 401);
  // Una clave incorrecta se rechaza igual que una clave ausente.
  const wrongKey = await fetch(endpoint, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${'0'.repeat(64)}` },
    body: JSON.stringify({ jsonrpc: '2.0', id: 0, method: 'resources/list' }),
  });
  assert.equal(wrongKey.status, 401);
  // Un origen web ajeno se rechaza antes de tocar el catálogo.
  const foreignOrigin = await fetch(endpoint, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Origin: 'https://example.com' },
    body: JSON.stringify({ jsonrpc: '2.0', id: 0, method: 'resources/list' }),
  });
  assert.equal(foreignOrigin.status, 403);
  const init = await rpc('initialize', { protocolVersion: '2025-06-18', capabilities: {}, clientInfo: { name: 'Codex-foundation-verification', version: '1' } });
  assert.equal(init.serverInfo.name, 'aurora-synapse');
  assert.ok(init.capabilities.resources);
  assert.ok(session);
  const notification = await fetch(endpoint, { method: 'POST', headers: headers(), body: JSON.stringify({ jsonrpc: '2.0', method: 'notifications/initialized' }) });
  assert.equal(notification.status, 202);
  await rpc('ping');
  const listed = await rpc('resources/list');
  assert.ok(listed.resources.some(resource => resource.uri === uri));
  const read = await rpc('resources/read', { uri });
  const catalog = JSON.parse(read.contents[0].text);
  const tools = await rpc('tools/list');
  assert.equal(catalog.tool_count, tools.tools.length);
  assert.equal(catalog.session_idle_timeout_seconds, 300);
  assert.ok(catalog.apps.every(app => app.availability === 'not_probed'));
  const classified = await rpc('tools/call', { name: 'classify_intent', arguments: { input: 'https://www.youtube.com/watch?v=synapse_probe' } });
  assert.equal(classified.isError, false);
  assert.ok(classified.structuredContent.recommended_app_id);
  const stdioRequests = [
    { jsonrpc: '2.0', id: 1, method: 'initialize', params: { protocolVersion: '2025-06-18', capabilities: {}, clientInfo: { name: 'Codex-stdio-verification', version: '1' } } },
    { jsonrpc: '2.0', method: 'notifications/initialized' },
    { jsonrpc: '2.0', id: 2, method: 'resources/read', params: { uri } },
    { jsonrpc: '2.0', id: 3, method: 'tools/list' },
  ];
  const stdio = spawnSync(binary, ['--mcp-stdio'], {
    input: stdioRequests.map(value => JSON.stringify(value)).join('\n') + '\n',
    encoding: 'utf8', windowsHide: true, timeout: 20_000, maxBuffer: 4 * 1024 * 1024,
  });
  assert.equal(stdio.status, 0, stdio.error?.message || stdio.stderr);
  const replies = stdio.stdout.trim().split(/\r?\n/).map(line => JSON.parse(line));
  assert.deepEqual(JSON.parse(replies.find(reply => reply.id === 2).result.contents[0].text), catalog);
  assert.deepEqual(replies.find(reply => reply.id === 3).result.tools, tools.tools);
  const closed = await fetch(endpoint, { method: 'DELETE', headers: headers() });
  assert.equal(closed.status, 204);
  const expired = await post({ jsonrpc: '2.0', id: ++id, method: 'ping' });
  assert.equal(expired.status, 404);
  assert.equal(expired.body.error, 'session_not_found');
  assert.match(expired.body.detail, /initialize/);
  session = undefined;

  // ── Recuperación tras el 404 ────────────────────────────────────────────
  // Un cliente que conserva la cabecera caducada debe poder reinitialize.
  const stale = 'syn-mcp-caducada-verificacion';
  const staleRejected = await post({ jsonrpc: '2.0', id: ++id, method: 'ping' }, { 'Mcp-Session-Id': stale });
  assert.equal(staleRejected.status, 404);
  const recovered = await post({
    jsonrpc: '2.0', id: ++id, method: 'initialize',
    params: { protocolVersion: '2025-06-18', capabilities: {}, clientInfo: { name: 'verificacion-reconexion', version: '1' } },
  }, { 'Mcp-Session-Id': stale });
  assert.equal(recovered.status, 200);
  const recoveredSession = recovered.headers.get('mcp-session-id');
  assert.ok(recoveredSession, 'initialize debe devolver un Mcp-Session-Id nuevo');
  assert.notEqual(recoveredSession, stale);
  const afterRecovery = await post({ jsonrpc: '2.0', id: ++id, method: 'tools/list' }, { 'Mcp-Session-Id': recoveredSession });
  assert.equal(afterRecovery.status, 200);
  await fetch(endpoint, { method: 'DELETE', headers: headers({ 'Mcp-Session-Id': recoveredSession }) });

  // ── Aislamiento entre clientes ──────────────────────────────────────────
  const first = await post({
    jsonrpc: '2.0', id: ++id, method: 'initialize',
    params: { protocolVersion: '2025-06-18', capabilities: {}, clientInfo: { name: 'verificacion-cliente-a', version: '1' } },
  });
  const firstSession = first.headers.get('mcp-session-id');
  const second = await post({
    jsonrpc: '2.0', id: ++id, method: 'initialize',
    params: { protocolVersion: '2025-06-18', capabilities: {}, clientInfo: { name: 'verificacion-cliente-b', version: '1' } },
  });
  const secondSession = second.headers.get('mcp-session-id');
  assert.ok(firstSession && secondSession);
  assert.notEqual(firstSession, secondSession, 'dos clientes no pueden compartir sesión');
  const status = await (await fetch(`${endpoint}/status`)).json();
  assert.ok(status.sessions >= 2, `se esperaban al menos 2 sesiones, hay ${status.sessions}`);

  // Cerrar una sesión no afecta a la otra.
  const closedSecond = await fetch(endpoint, { method: 'DELETE', headers: headers({ 'Mcp-Session-Id': secondSession }) });
  assert.equal(closedSecond.status, 204);
  const stillAlive = await post({ jsonrpc: '2.0', id: ++id, method: 'ping' }, { 'Mcp-Session-Id': firstSession });
  assert.equal(stillAlive.status, 200);

  // ── El recurso publica el contrato de sesión ───────────────────────────
  const readContract = await post({ jsonrpc: '2.0', id: ++id, method: 'resources/read', params: { uri } }, { 'Mcp-Session-Id': firstSession });
  const contract = JSON.parse(readContract.body.result.contents[0].text);
  assert.equal(contract.session.header, 'Mcp-Session-Id');
  assert.equal(contract.session.idle_timeout_seconds, 300);
  assert.match(contract.session.recovery, /initialize/);
  assert.match(contract.session.authorization, /Bearer/);

  await fetch(endpoint, { method: 'DELETE', headers: headers({ 'Mcp-Session-Id': firstSession }) });
  session = undefined;

  // ── Caducidad real (opcional, lenta) ─────────────────────────────────────
  // Sin atajos de reloj: la ventana de inactividad del servidor son 300 s.
  // A no recibe peticiones; B se renueva cerca del límite. El sondeo del
  // panel se ejecuta durante toda la espera para demostrar que no renueva.
  let sessionTtl = null;
  if (process.argv.includes('--slow')) {
    const waitSecs = Number(process.env.SYNAPSE_TTL_WAIT_SECS ?? 312);
    assert.ok(waitSecs > 300, 'la espera debe superar la ventana de inactividad de 300 s');

    const idle = await (await fetch(`${endpoint}/status`)).json();
    assert.equal(idle.sessions, 0, 'la prueba lenta necesita un servidor sin sesiones previas');

    const create = async name => {
      const response = await post({
        jsonrpc: '2.0', id: ++id, method: 'initialize',
        params: { protocolVersion: '2025-06-18', capabilities: {}, clientInfo: { name, version: '1' } },
      });
      assert.equal(response.status, 200);
      const assigned = response.headers.get('mcp-session-id');
      assert.ok(assigned);
      return assigned;
    };
    const sessionA = await create('ttl-sin-renovar');
    const sessionB = await create('ttl-renovado');
    const started = Date.now();
    let renewed = false;
    let observed = 0;

    while ((Date.now() - started) / 1000 < waitSecs) {
      const status = await (await fetch(`${endpoint}/status`)).json();
      observed = Math.max(observed, status.sessions);
      assert.ok(status.sessions <= 2, `el panel no debe inventar sesiones: ${status.sessions}`);
      const elapsed = (Date.now() - started) / 1000;
      // Renovar con margen suficiente: el bucle duerme 15 s, así que hay que
      // hacerlo bastante antes del límite de 300 s o la propia espera caduca B.
      if (!renewed && elapsed >= waitSecs - 45) {
        const pong = await post({ jsonrpc: '2.0', id: ++id, method: 'ping' }, { 'Mcp-Session-Id': sessionB });
        assert.equal(pong.status, 200, 'la sesión renovada debe seguir viva');
        renewed = true;
      }
      await wait(15_000);
    }
    assert.ok(renewed, 'no se llegó a renovar la sesión B dentro de la ventana');

    // A quedó inactiva más de 300 s aunque el panel se consultó repetidamente.
    const expiredIdle = await post({ jsonrpc: '2.0', id: ++id, method: 'ping' }, { 'Mcp-Session-Id': sessionA });
    assert.equal(expiredIdle.status, 404, 'la sesión sin renovar debe haber caducado');
    assert.equal(expiredIdle.body.error, 'session_not_found');

    // B sigue viva porque una petición aceptada renovó su actividad.
    const aliveRenewed = await post({ jsonrpc: '2.0', id: ++id, method: 'ping' }, { 'Mcp-Session-Id': sessionB });
    assert.equal(aliveRenewed.status, 200);

    // initialize con la cabecera caducada abre una sesión nueva.
    const recovered = await post({
      jsonrpc: '2.0', id: ++id, method: 'initialize',
      params: { protocolVersion: '2025-06-18', capabilities: {}, clientInfo: { name: 'ttl-recuperado', version: '1' } },
    }, { 'Mcp-Session-Id': sessionA });
    assert.equal(recovered.status, 200);
    const sessionC = recovered.headers.get('mcp-session-id');
    assert.ok(sessionC && sessionC !== sessionA);
    assert.equal((await post({ jsonrpc: '2.0', id: ++id, method: 'tools/list' }, { 'Mcp-Session-Id': sessionC })).status, 200);

    const remaining = await (await fetch(`${endpoint}/status`)).json();
    assert.equal(remaining.sessions, 2, 'solo B y la sesión recuperada deben quedar vivas');
    await fetch(endpoint, { method: 'DELETE', headers: headers({ 'Mcp-Session-Id': sessionB }) });
    await fetch(endpoint, { method: 'DELETE', headers: headers({ 'Mcp-Session-Id': sessionC }) });

    sessionTtl = {
      wait_seconds: Math.round((Date.now() - started) / 1000),
      idle_expired: true,
      panel_polling_did_not_renew: true,
      renewed_session_alive: true,
      recovery_initialize_after_404: true,
      max_sessions_observed: observed,
    };
  }

  console.log(JSON.stringify({
    result: 'ok',
    transports: ['http_loopback', 'stdio'],
    tools: catalog.tool_count,
    resource: uri,
    authenticated: true,
    wrong_key_rejected: true,
    foreign_origin_rejected: true,
    session_closed: true,
    session_recovery_after_404: true,
    session_isolation: true,
    readonly_call: 'classify_intent',
    ...(sessionTtl ? { session_ttl: sessionTtl } : {}),
  }, null, 2));
} finally {
  if (session) await fetch(endpoint, { method: 'DELETE', headers: headers(), signal: AbortSignal.timeout(2000) }).catch(() => {});
  if (child.exitCode === null && !spawnError) {
    const exited = new Promise(done => child.once('exit', done));
    child.kill();
    await exited;
  }
}
