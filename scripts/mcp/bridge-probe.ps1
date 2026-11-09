<#
.SYNOPSIS
    Sonda del puente MCP de Aurora Synapse.

.DESCRIPTION
    Habla MCP (JSON-RPC 2.0 sobre Streamable HTTP) contra el puente de Aurora
    Synapse: abre sesión, lista las herramientas y, opcionalmente, invoca una.

    Ejemplos:
      .\bridge-probe.ps1
      .\bridge-probe.ps1 -Tool luna_status
      .\bridge-probe.ps1 -Tool apps_list -Arguments '{}'
      .\bridge-probe.ps1 -Url http://127.0.0.1:49296/mcp -Tool luna_download -Arguments '{"url":"https://example.com/video.mp4"}'

    Exit code 0 si el puente responde, 1 si no.
#>
[CmdletBinding()]
param(
    [string] $Url = 'http://127.0.0.1:49296/mcp',
    [string] $Tool,
    [string] $Arguments = '{}',
    [string] $Key = $env:AURORA_SYNAPSE_MCP_KEY,
    [ValidateRange(1, 600)] [int] $TimeoutSec = 150
)

$ErrorActionPreference = 'Stop'
$session = $null
$exitCode = 1
$script:RequestId = 0

function Invoke-Mcp {
    param(
        [Parameter(Mandatory)] [string] $Method,
        [object] $Params,
        [string] $SessionId
    )

    $script:RequestId++
    $payload = @{ jsonrpc = '2.0'; id = $script:RequestId; method = $Method }
    if ($null -ne $Params) { $payload.params = $Params }

    $headers = @{ 'Content-Type' = 'application/json'; 'Accept' = 'application/json, text/event-stream' }
    if ($SessionId) { $headers['Mcp-Session-Id'] = $SessionId }
    if ($Key) { $headers['Authorization'] = "Bearer $Key" }

    $body = $payload | ConvertTo-Json -Depth 12 -Compress
    $response = Invoke-WebRequest -Uri $Url -Method Post -Headers $headers -Body ([System.Text.Encoding]::UTF8.GetBytes($body)) -TimeoutSec $TimeoutSec -UseBasicParsing
    $parsed = $response.Content | ConvertFrom-Json
    if ($parsed.jsonrpc -ne '2.0' -or $parsed.id -ne $script:RequestId) {
        throw "Respuesta JSON-RPC inválida para $Method."
    }
    if ($null -ne $parsed.error) {
        throw "Error JSON-RPC $($parsed.error.code): $($parsed.error.message)"
    }
    if ($null -eq $parsed.result) { throw "Respuesta sin resultado para $Method." }
    [pscustomobject]@{
        Result  = $parsed
        Session = [string](@($response.Headers['Mcp-Session-Id'])[0])
    }
}

try {
    $init = Invoke-Mcp -Method 'initialize' -Params @{
        protocolVersion = '2025-06-18'
        capabilities = @{}
        clientInfo = @{ name = 'aurora-synapse-bridge-probe'; version = '1.1' }
    }
    $session = $init.Session
    if ($init.Result.result.protocolVersion -ne '2025-06-18') {
        throw 'El puente negoció una versión de protocolo no compatible con esta sonda.'
    }
    $headers = @{ 'Content-Type' = 'application/json'; 'Accept' = 'application/json, text/event-stream' }
    if ($session) { $headers['Mcp-Session-Id'] = $session }
    if ($Key) { $headers['Authorization'] = "Bearer $Key" }
    Invoke-WebRequest -Uri $Url -Method Post -Headers $headers -Body '{"jsonrpc":"2.0","method":"notifications/initialized"}' -TimeoutSec $TimeoutSec -UseBasicParsing | Out-Null

    $list = Invoke-Mcp -Method 'tools/list' -Params @{} -SessionId $session
    if ($null -eq $list.Result.result.tools) { throw 'El puente no devolvió un catálogo de herramientas.' }
    $toolList = @($list.Result.result.tools)
    Write-Host "Puente MCP activo en $Url — herramientas: $($toolList.Count)" -ForegroundColor Green
    foreach ($entry in $toolList) {
        $label = if ($entry.annotations.readOnlyHint) { ' [solo lectura]' } else { '' }
        Write-Host ("  {0}{1}" -f $entry.name, $label)
    }
    if ($Tool) {
        if (-not ($toolList | Where-Object { $_.name -eq $Tool })) { throw "La herramienta '$Tool' no está publicada." }
        $argsObject = $Arguments | ConvertFrom-Json
        if ($null -eq $argsObject -or $argsObject -isnot [pscustomobject]) { throw 'Arguments debe ser un objeto JSON.' }
        $call = Invoke-Mcp -Method 'tools/call' -Params @{ name = $Tool; arguments = $argsObject } -SessionId $session
        if ($null -eq $call.Result.result.content -or $call.Result.result.isError -isnot [bool]) {
            throw 'Resultado de herramienta incompleto.'
        }
        Write-Host ($call.Result.result | ConvertTo-Json -Depth 20)
        if ($call.Result.result.isError) { throw "La herramienta '$Tool' devolvió un error." }
    }
    $exitCode = 0
} catch {
    [Console]::Error.WriteLine("Sonda MCP fallida: $($_.Exception.Message)")
} finally {
    if ($session) {
        $closeHeaders = @{ 'Mcp-Session-Id' = $session }
        if ($Key) { $closeHeaders['Authorization'] = "Bearer $Key" }
        try {
            Invoke-WebRequest -Uri $Url -Method Delete -Headers $closeHeaders -TimeoutSec 5 -UseBasicParsing | Out-Null
        } catch {
            [Console]::Error.WriteLine("No se pudo cerrar la sesión MCP: $($_.Exception.Message)")
        }
    }
}
exit $exitCode
