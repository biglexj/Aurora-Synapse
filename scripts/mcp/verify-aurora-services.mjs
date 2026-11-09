// Diagnostico separado de los cuatro servicios MCP de Aurora.
//
//   1. MCP directo de Aurora  -> backend Rust local (http://127.0.0.1:5000)
//   2. Conector publico       -> https://api.biglexj.com/api/v1/mcp
//   3. Aurora Hub             -> app de escritorio; cliente Host, no servidor
//   4. Aurora -> Synapse      -> puente `synapse_bridge` del backend
//
// Solo lectura: initialize, tools/list y consultas de estado. No invoca
// herramientas mutantes, no modifica despliegues y no imprime secretos.
//
// Política de credenciales: la credencial del backend se envia SOLO a
// localhost. Los endpoints publicos se sondean sin credencial, salvo que se
// pase `--auth-publica`, que es una decision explicita del operador.
//
// Ejecutar: bun scripts/mcp/verify-aurora-services.mjs [--auth-publica]
import { readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const blog = resolve(root, '../Aurora---Blog/backend');
const AUTH_PUBLICA = process.argv.includes('--auth-publica');

// ── Entorno: se cargan los .env del backend SOLO para leer nombres y valores
// de credencial. Ningun valor se imprime.
const environment = { ...process.env };
const envFiles = [
  join(blog, 'target/debug/.env'),
  join(blog, 'target/debug/resources/.env'),
  join(blog, '.env'),
  join(process.env.APPDATA || '', 'AuroraHub/.env'),
];
for (const path of envFiles) {
  const contents = await readFile(path, 'utf8').catch(() => '');
  for (const line of contents.split(/\r?\n/)) {
    const match = line.match(/^\s*([A-Z][A-Z0-9_]*)=(.*)$/);
    if (match && environment[match[1]] === undefined) {
      environment[match[1]] = match[2].trim().replace(/^(['"])(.*)\1$/, '$2');
    }
  }
}

const LOCAL = 'http://127.0.0.1:5000';
const PUBLIC = 'https://api.biglexj.com/api/v1/mcp';
const PUBLIC_PROXY = 'https://www.biglexj.com/api/v1/mcp';
const TIMEOUT = 20_000;

/** Credenciales disponibles, sin revelar sus valores. */
function credentials() {
  const candidates = [
    ['MCP_API_KEY', environment.MCP_API_KEY],
    ['ELY_INTELLIGENCE_SHARED_SECRET', environment.ELY_INTELLIGENCE_SHARED_SECRET],
  ].filter(([, value]) => typeof value === 'string' && value.trim().length > 0);
  return candidates.map(([name, value]) => ({ name, value: value.trim() }));
}

const available = credentials();

/**
 * Cabeceras de autenticación.
 *
 * `loopback` decide si la credencial puede enviarse: a `127.0.0.1` sí; a un
 * host público solo con `--auth-publica`, porque enviar el secreto del
 * backend a internet es una decisión del operador, no unaroutine.
 */
function authHeaders(scope) {
  const headers = { 'Content-Type': 'application/json', Accept: 'application/json, text/event-stream' };
  const allowed = scope === 'loopback' || AUTH_PUBLICA;
  if (!allowed || !available.length) return headers;
  headers['x-mcp-key'] = available[0].value;
  headers.Authorization = `Bearer ${available[0].value}`;
  return headers;
}

function scopeOf(url) {
  return url.startsWith('http://127.0.0.1') || url.startsWith('http://localhost') ? 'loopback' : 'publica';
}

async function rpc(url, method, params = {}, extraHeaders = {}) {
  const response = await fetch(url, {
    method: 'POST',
    headers: { ...authHeaders(scopeOf(url)), ...extraHeaders },
    body: JSON.stringify({ jsonrpc: '2.0', id: 1, method, params }),
    signal: AbortSignal.timeout(TIMEOUT),
  });
  const text = await response.text();
  let parsed = null;
  try { parsed = JSON.parse(text); } catch { /* cuerpo no JSON */ }
  return { status: response.status, body: parsed, raw: text.slice(0, 400) };
}

/** Describe un fallo de red sin filtrar nada sensible. */
function describeError(error) {
  const cause = error?.cause?.code ?? error?.cause?.message ?? error?.message ?? error?.name;
  return `sin respuesta: ${cause}`;
}

/**
 * Sondea un endpoint MCP y describe el resultado sin filtrar credenciales.
 */
async function probeMcp(label, url) {
  const record = {
    servicio: label,
    url,
    credencial_enviada: scopeOf(url) === 'loopback' || AUTH_PUBLICA,
    reachable: false,
    autenticado: false,
    detalle: '',
  };
  let init;
  try {
    init = await rpc(url, 'initialize', {
      protocolVersion: '2025-06-18',
      capabilities: {},
      clientInfo: { name: 'synapse-verificacion-aurora', version: '1' },
    });
  } catch (error) {
    record.detalle = describeError(error);
    return record;
  }
  record.reachable = true;
  record.http_status = init.status;
  if (init.status === 401 || init.status === 403) {
    record.detalle = 'responde pero rechaza la credencial';
    return record;
  }
  if (init.body?.error) {
    record.detalle = `error JSON-RPC ${init.body.error.code}: ${init.body.error.message}`;
    return record;
  }
  record.autenticado = true;
  record.server = init.body?.result?.serverInfo?.name ?? null;
  record.protocol = init.body?.result?.protocolVersion ?? null;
  const listed = await rpc(url, 'tools/list');
  record.herramientas = listed.body?.result?.tools?.length ?? 0;
  record.detalle = 'initialize y tools/list correctos';
  return record;
}

/** Consulta el estado del puente Aurora -> Synapse. */
async function probeSynapseBridge() {
  const record = {
    servicio: 'Aurora -> Synapse (synapse_bridge)',
    url: `${LOCAL}/api/v1/mcp/synapse/status`,
    reachable: false,
    configurado: false,
    conectado: false,
    detalle: '',
  };
  try {
    const response = await fetch(record.url, {
      method: 'GET',
      headers: authHeaders('loopback'),
      signal: AbortSignal.timeout(TIMEOUT),
    });
    record.reachable = true;
    record.http_status = response.status;
    if (response.status !== 200) {
      record.detalle = `HTTP ${response.status}`;
      return record;
    }
    const body = await response.json();
    record.configurado = body.configured === true;
    record.conectado = body.connected === true;
    record.url_configurado = body.url ?? null;
    record.ultimo_error = body.last_error ?? null;
    record.herramientas_publicadas = body.tools ?? null;
    record.detalle = record.configurado
      ? (record.conectado ? 'puente configurado y conectado' : 'configurado pero sin conexión')
      : 'puente sin configurar en este runtime';
  } catch (error) {
    record.detalle = describeError(error);
  }
  return record;
}

/** Estado del Host de Aurora Hub visto por el backend. */
async function probeAuroraHub() {
  const record = {
    servicio: 'Aurora Hub (host MCP del backend)',
    url: `${LOCAL}/api/v1/mcp/hosts/status`,
    reachable: false,
    detalle: '',
  };
  try {
    const response = await fetch(record.url, {
      method: 'GET',
      headers: authHeaders('loopback'),
      signal: AbortSignal.timeout(TIMEOUT),
    });
    record.reachable = true;
    record.http_status = response.status;
    record.detalle = `HTTP ${response.status}`;
    const text = await response.text();
    try {
      const body = JSON.parse(text);
      record.hosts = body.hosts ?? body;
    } catch { /* cuerpo no JSON */ }
  } catch (error) {
    record.detalle = describeError(error);
  }
  return record;
}

/** `mcp.status` del backend: host_bridge, invocation_audit y synapse_bridge. */
async function probeMcpStatus() {
  const record = { servicio: 'mcp.status (control plane)', detalle: '' };
  try {
    const response = await fetch(`${LOCAL}/api/v1/mcp/tools`, {
      method: 'GET',
      headers: authHeaders('loopback'),
      signal: AbortSignal.timeout(TIMEOUT),
    });
    record.http_status = response.status;
    if (response.status !== 200) {
      record.detalle = `HTTP ${response.status}`;
      return record;
    }
    const status = await rpc(`${LOCAL}/api/v1/mcp`, 'tools/call', {
      name: 'mcp.status',
      arguments: {},
    });
    const result = status.body?.result?.structuredContent ?? status.body?.result;
    if (result) {
      record.host_bridge = result.host_bridge ?? null;
      record.invocation_audit = result.invocation_audit ?? null;
      record.synapse_bridge = result.synapse_bridge ?? null;
    }
    record.detalle = result ? 'mcp.status respondido' : 'mcp.status no respondio con datos';
  } catch (error) {
    record.detalle = describeError(error);
  }
  return record;
}

/**
 * Lee el estado del control plane de Aurora con `mcp.status`.
 *
 * Es una herramienta de solo lectura: informa de `host_bridge` (Aurora Hub),
 * `invocation_audit` y `synapse_bridge` (Aurora -> Synapse) sin mutar nada.
 * Se intenta en el endpoint local y en el público.
 */
async function probeControlPlane(label, url) {
  const record = { servicio: label, url, detalle: '' };
  try {
    const listed = await rpc(url, 'tools/list');
    const names = (listed.body?.result?.tools ?? []).map(tool => tool.name);
    record.herramientas = names.length;
    const statusName = names.find(name => name === 'mcp.status' || name === 'mcp_status');
    if (!statusName) {
      record.detalle = `el catalogo no publica ${names.includes('mcp_status') ? 'mcp_status' : 'mcp.status'}`;
      return record;
    }
    const called = await rpc(url, 'tools/call', { name: statusName, arguments: {} });
    if (called.body?.error) {
      record.detalle = `error JSON-RPC ${called.body.error.code}: ${called.body.error.message}`;
      return record;
    }
    // El resultado puede venir como `structuredContent` o como texto JSON en
    // el primer bloque de contenido.
    const raw = called.body?.result?.structuredContent
      ?? called.body?.result
      ?? null;
    const text = called.body?.result?.content?.[0]?.text;
    let result = raw;
    if (typeof text === 'string') {
      try { result = JSON.parse(text); } catch { result = raw; }
    }
    if (result) {
      record.host_bridge = result.host_bridge ?? null;
      record.invocation_audit = result.invocation_audit ?? null;
      record.synapse_bridge = result.synapse_bridge ?? null;
      record.catalog_tools = result.catalog_tools ?? null;
      record.autoridad = result.authority ?? null;
      record.detalle = 'mcp.status respondido';
    } else {
      record.detalle = 'mcp.status sin datos estructurados';
    }
  } catch (error) {
    record.detalle = describeError(error);
  }
  return record;
}

const results = [];

results.push(await probeMcp('MCP directo de Aurora (backend local)', `${LOCAL}/api/v1/mcp`));
results.push(await probeMcp('Conector publico Aurora MCP', PUBLIC));
results.push(await probeMcp('Proxy publico del sitio (Astro)', PUBLIC_PROXY));
results.push(await probeSynapseBridge());
results.push(await probeAuroraHub());
results.push(await probeMcpStatus());
results.push(await probeControlPlane('mcp.status en el conector publico', PUBLIC));

const summary = {
  resultado: 'ok',
  credenciales_disponibles: available.map(entry => entry.name),
  credencial_enviada_a_publicos: AUTH_PUBLICA,
  servicios: results,
};

console.log(JSON.stringify(summary, null, 2));
