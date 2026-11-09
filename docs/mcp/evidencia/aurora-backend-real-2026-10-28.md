# Evidencia: runtime real de Aurora y enlace Aurora → Synapse

Fecha: 2026-10-28. Proceso: [SYN-110-MCP](../../../process/active/2026-10-27_sesiones-y-conexion-mcp/PLAN.md).

Diagnóstico de los cuatro servicios de Aurora **por separado**, con el backend
real en marcha. No se imprime ninguna credencial: la del backend se usa solo
contra `127.0.0.1` y nunca sale del equipo.

## Entorno

- El backend Rust de Aurora **no estaba en ejecución** al empezar: nada
  escuchaba en el puerto 5000.
- Para el diagnóstico se arrancó el binario ya compilado del propio proyecto,
  `Aurora---Blog/backend/target/debug/aurora-backend.exe`, con
  `APP_HOST=127.0.0.1`. Su MCP quedó **solo en loopback**: en ningún momento
  se expuso a la LAN.
- Se detuvo al terminar la comprobación; el puerto 5000 quedó libre.

## 1. MCP directo de Aurora — `http://127.0.0.1:5000/api/v1/mcp`

| Comprobación | Resultado observado |
| --- | --- |
| `initialize` | `200` — `serverInfo: {name: "aurora-mcp", version: "1.0.0"}`, protocolo `2025-06-18` |
| `tools/list` | `200` — 17 herramientas del catálogo remoto de Aurora |
| `GET /api/v1/mcp/tools` (control plane) | `200` — 59 herramientas, de las cuales **20 con prefijo `synapse.`** |
| `tools/call` de `mcp_status` por el MCP remoto | «Operación restringida: herramienta no disponible en el catálogo remoto» |
| `POST /api/v1/mcp/invoke` con `mcp_status` | `status: executed` |

**Causa del fallo directo registrado antes:** el servidor MCP de Aurora no
estaba levantado en el entorno. No era un problema de credencial del cliente.
Además `mcp.status` pertenece al **control plane**, no al catálogo remoto:
invocarla por `tools/call` del MCP remoto es un error de ruta, no un fallo de
autenticación.

## 2. Conector público — `https://api.biglexj.com/api/v1/mcp`

| Comprobación | Resultado observado |
| --- | --- |
| `initialize` sin credencial | `200` — `serverInfo: {name: "aurora-mcp", version: "1.0.0"}` |
| `tools/list` sin credencial | `200` — catálogo remoto de Aurora |
| `aurora_site_context` | `200` — contexto de la plataforma |
| `mcp_status` por llamada directa sin credencial | `{"error":"invalid_token"}` — el endpoint directo exige credencial |

La sonda del frente (`verify-aurora-services.mjs`) sí lee el estado del
runtime publicado: `host_bridge: disconnected`, `invocation_audit:
unavailable`, `synapse_bridge: {configured: false, reason: "AURORA_SYNAPSE_MCP_URL
no está definido"}`, `catalog_tools: 39`, 8 herramientas públicas.

**Límite:** ese estado es del runtime **publicado** y no acredita control del
PC. En esta ejecución no se envió ninguna credencial local a un host remoto.

## 3. Aurora Hub

Aurora Hub es una app de escritorio Tauri que actúa como **cliente Host**, no
como servidor MCP: no expone puerto ni endpoint MCP propio. Su estado se ve
desde el backend, en la herramienta `mcp.status`.

| Comprobación | Resultado observado |
| --- | --- |
| `GET /api/v1/mcp/hosts/status` | `{"connected":false,"ready":false,"connectionId":null,"hostId":null,"capabilities":[]}` |
| `mcp.status` → `host_bridge` | `"disconnected"` |
| `mcp.status` → `invocation_audit` | `"unavailable"` (constante del backend) |
| Proceso Aurora Hub | No en ejecución |

Causa concreta: **Aurora Hub no está en marcha en este equipo**. No es un
fallo del puente.

## 4. Aurora → Synapse — lectura real del puente en este PC

| Comprobación | Resultado observado |
| --- | --- |
| `GET /api/v1/mcp/synapse/status` | `{"configured":true,"connected":true,"last_error":null,"tools":20,"url":"http://127.0.0.1:49296/mcp"}` |
| `mcp.status` → `synapse_bridge` | `configured: true`, `connected: true`, `tools: 20`, `last_error: null` |
| Log del backend | «🌉 [MCP] Catálogo del puente Aurora Synapse actualizado — count=20» |

Configuración en el runtime del backend (`Aurora---Blog/backend/.env`):

```
AURORA_SYNAPSE_MCP_URL=http://127.0.0.1:49296/mcp
AURORA_SYNAPSE_TIMEOUT_MS=150000
```

El runtime de Aurora, en este mismo equipo, refrescó el catálogo del puente
Synapse y lo publica como `synapse.*`. Es control de lectura verificado, no
una inferencia a partir de una respuesta pública.

**Alcance:** la URL configurada es loopback. Solo funciona si el backend de
Aurora y Synapse están en la misma máquina. Un runtime remoto de Aurora no
alcanza este puente por esa vía: el canal remoto autenticado sigue pendiente
y no se sustituye por la lectura pública.

## 5. Instancia de Synapse ya en ejecución

El puente que el usuario tenía abierto en `127.0.0.1:49296` respondió al 404
de sesión con `{"error":"session_not_found"}` **sin** `detail`: es el binario
anterior a los cambios de este frente. Esa instancia se diagnostica; no se
reinició. Para que recoja el contrato nuevo hace falta reiniciar la
aplicación, que requiere autorización.
