<#
.SYNOPSIS
    Script oficial de compilación y empaquetado de Release para Aurora Synapse.

.DESCRIPTION
    Compila el bundle de producción de frontend y backend de Tauri v2 (MSI / EXE / NSIS),
    calcula hashes SHA-256 y prepara los artefactos de distribución.

.PARAMETER Version
    Versión del release (e.g. "0.1.0").

.EXAMPLE
    .\scripts\build\build-release.ps1 -Version "0.1.0"
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $false)]
    [string]$Version = "0.1.0"
)

$ErrorActionPreference = "Stop"

Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  ⚡ Aurora Synapse — Official Release Build Pipeline          " -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "Versión de Compilación: $Version" -ForegroundColor Yellow
Write-Host ""

# 1. Preflight Checks
Write-Host "➔ 1/4 Verificando dependencias y sincronización de iconos..." -ForegroundColor Green
bun run build
if ($LASTEXITCODE -ne 0) {
    Write-Error "Fallo en la compilación del frontend de Vite."
    exit 1
}

# 2. Build Tauri Release Bundle
Write-Host "➔ 2/4 Compilando binarios de escritorio con Tauri v2 (Release)..." -ForegroundColor Green
bun run tauri build
if ($LASTEXITCODE -ne 0) {
    Write-Error "Fallo en la compilación de Release de Tauri."
    exit 1
}

# 3. Artifact Packaging
$bundleDir = ".\src-tauri\target\release\bundle"
$releaseOutputDir = ".\release\v$Version"
New-Item -ItemType Directory -Force -Path $releaseOutputDir | Out-Null

Write-Host "➔ 3/4 Copiando instalador oficial de escritorio (.exe) a $releaseOutputDir..." -ForegroundColor Green
if (Test-Path "$bundleDir\nsis") {
    Copy-Item "$bundleDir\nsis\*.exe" -Destination $releaseOutputDir -Force
}
$androidApk = ".\src-tauri\gen\android\app\build\outputs\apk\release\app-universal-release-unsigned.apk"
if (Test-Path $androidApk) {
    Copy-Item $androidApk -Destination "$releaseOutputDir\Aurora-Synapse-v$Version.apk" -Force
}

# 4. Generate SHA256 Hashes
Write-Host "➔ 4/4 Generando sumas de verificación SHA-256..." -ForegroundColor Green
Get-ChildItem -Path $releaseOutputDir -File | ForEach-Object {
    $hash = Get-FileHash -Path $_.FullName -Algorithm SHA256
    $hashLine = "$($hash.Hash)  $($_.Name)"
    $hashLine | Out-File -FilePath "$releaseOutputDir\SHA256SUMS.txt" -Append -Encoding utf8
    Write-Host "  • $($_.Name): $($hash.Hash)" -ForegroundColor DarkGray
}

Write-Host ""
Write-Host "🎉 Release v$Version generado exitosamente al 10,000 millones por ciento." -ForegroundColor Cyan
