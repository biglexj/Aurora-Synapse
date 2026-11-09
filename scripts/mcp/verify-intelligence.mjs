// Verifica el backend local real y su frontera de confirmación.
// node scripts/mcp/verify-intelligence.mjs [--chat] [--gallery]
// Si la carpeta de recepción está excluida, copiar solo la imagen de prueba a
// una carpeta compartida y usar --gallery-receive=synapse_mcp_<fecha>.png.
import { readFile, mkdtemp, copyFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { join, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';

const root = fileURLToPath(new URL('../../', import.meta.url));
const backend = resolve(root, '../Aurora---Blog/backend');
const environment = { ...process.env };
for (const path of [join(backend, 'target/debug/.env'), join(backend, 'target/debug/resources/.env'), join(process.env.APPDATA || '', 'AuroraHub/.env'), join(backend, '.env')]) {
  const contents = await readFile(path, 'utf8').catch(() => '');
  for (const line of contents.split(/\r?\n/)) {
    const match = line.match(/^\s*([A-Z][A-Z0-9_]*)=(.*)$/);
    if (match && environment[match[1]] === undefined) environment[match[1]] = match[2].trim().replace(/^(['"])(.*)\1$/, '$2');
  }
}
let key = environment.ELY_INTELLIGENCE_SHARED_SECRET?.trim() || '';
if (key.length < 32 && environment.SUPABASE_SERVICE_ROLE_KEY?.trim().length >= 32) {
  key = createHash('sha256').update(`ely-intelligence-boundary-v1|${environment.SUPABASE_SERVICE_ROLE_KEY.trim()}`).digest('hex');
}
assert.ok(key.length >= 32, 'Falta la credencial interna del backend local');
const headers = { 'Content-Type': 'application/json', 'X-Ely-Intelligence-Key': key, 'X-Aurora-Principal': 'synapse-local-verification' };
async function api(path, payload) {
  const response = await fetch(`http://127.0.0.1:5000${path}`, {
    method: payload === undefined ? 'GET' : 'POST', headers,
    body: payload === undefined ? undefined : JSON.stringify(payload), signal: AbortSignal.timeout(150_000),
  });
  assert.equal(response.status, 200, `API HTTP ${response.status}`);
  return response.json();
}
async function invoke(name, arguments_ = {}) {
  let result = await api('/api/v1/mcp/invoke', { name, arguments: arguments_ });
  if (result.status === 'pending_confirmation') {
    assert.ok(result.confirmation_id, 'Falta el identificador de confirmación');
    // Esta sonda solo muta sus propias imágenes de prueba, a petición del operador.
    result = await api('/api/v1/mcp/confirm', { confirmation_id: result.confirmation_id, approved: true });
  }
  assert.equal(result.status, 'executed', JSON.stringify(result));
  assert.equal(result.result.ok, true, JSON.stringify(result.result));
  return result.result.result;
}

const connection = await api('/api/v1/mcp/synapse/status');
console.log('Intelligence → MCP:', JSON.stringify(connection));
assert.equal(connection.connected, true);
const tools = await api('/api/v1/mcp/tools');
assert.equal(tools.filter(tool => tool.name.startsWith('synapse.gallery_')).length, 5);
const gallery = await invoke('synapse.gallery_status');
console.log('Teléfono:', JSON.stringify({ configured: gallery.configured, reachable: gallery.reachable, paired: gallery.paired }));

const receiveFixture = process.argv.find(arg => arg.startsWith('--gallery-receive='))?.split('=')[1];
if (process.argv.includes('--gallery') || receiveFixture) {
  if (receiveFixture) assert.match(receiveFixture, /^synapse_mcp_\d+\.png$/);
  assert.equal(gallery.paired, true, 'Introduce el código del teléfono en Synapse antes de esta prueba');
  const directory = await mkdtemp(join(tmpdir(), 'synapse-intelligence-gallery-'));
  const input = join(directory, receiveFixture || `synapse_mcp_${Date.now()}.png`);
  await copyFile(join(root, 'src-tauri/icons/32x32.png'), input);
  const uploaded = receiveFixture ? null : await invoke('synapse.gallery_send', { file_path: input });
  if (uploaded) assert.equal(uploaded.completed, true);
  const name = uploaded?.phone?.file_name || input.split(/[\\/]/).at(-1);
  console.log('Imagen de prueba:', JSON.stringify({ name, bytes: uploaded?.bytes, sha256: uploaded?.sha256 }));
  let media;
  for (let attempt = 0; attempt < 20; attempt++) {
    const catalog = await invoke('synapse.gallery_list', { query: name, media_type: 'image', limit: 10 });
    if (attempt === 0) console.log('Consulta de la imagen de prueba:', JSON.stringify(catalog));
    media = catalog.items.find(item => item.name === name);
    if (media) break;
    await new Promise(done => setTimeout(done, 500));
  }
  assert.ok(media, 'La imagen de prueba no aparece en el catálogo compartido; comprueba indexación y exclusiones de la carpeta receptora');
  const output = join(directory, 'recibida.png');
  const downloaded = await invoke('synapse.gallery_download', { media_id: media.id, output_path: output });
  assert.equal(downloaded.completed, true);
  assert.deepEqual(await readFile(output), await readFile(input));
  assert.equal(createHash('sha256').update(await readFile(input)).digest('hex'), downloaded.sha256);
  const invalid = await api('/api/v1/mcp/invoke', { name: 'synapse.gallery_download', arguments: { media_id: media.id, output_path: output } });
  const rejected = invalid.status === 'pending_confirmation'
    ? await api('/api/v1/mcp/confirm', { confirmation_id: invalid.confirmation_id, approved: true }) : invalid;
  assert.equal(rejected.status, 'error', 'Un rechazo de la app debe aparecer como error en Intelligence');
  console.log(receiveFixture ? 'Descarga y rechazo de sobrescritura comprobados:' : 'Transferencia bidireccional comprobada:', JSON.stringify({ bytes: downloaded.bytes, sha256: downloaded.sha256, output }));
}

if (process.argv.includes('--chat')) {
  const response = await fetch('http://127.0.0.1:5000/api/v1/ai/chat', {
    method: 'POST', headers, signal: AbortSignal.timeout(150_000),
    body: JSON.stringify({ model: 'gemini-3.8-flash', stream: true, message: 'Consulta el estado actual de Super Galería usando synapse_gallery_status. No listes archivos ni ejecutes transferencias. Responde con una frase basada en el resultado real.' }),
  });
  assert.equal(response.status, 200, `Chat HTTP ${response.status}`);
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let pending = '';
  const events = [];
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    pending += decoder.decode(value, { stream: true });
    const frames = pending.split(/\r?\n\r?\n/);
    pending = frames.pop();
    for (const frame of frames) {
      const data = frame.split(/\r?\n/).filter(line => line.startsWith('data:')).map(line => line.slice(5).trim()).join('\n');
      if (data && data !== '[DONE]') {
        try { const event = JSON.parse(data); events.push(event); console.log('Chat:', JSON.stringify(event)); }
        catch { /* solo interesan los eventos JSON */ }
      }
    }
  }
  assert.ok(events.some(event => ['synapse_gallery_status', 'synapse.gallery_status'].includes(event.tool)
    && event.status === 'executed' && event.result?.ok === true), 'No hay evidencia de ejecución correcta de gallery_status');
}
