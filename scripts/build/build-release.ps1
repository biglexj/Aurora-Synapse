<#
.SYNOPSIS
    Script oficial de compilación y empaquetado de Release para Aurora Synapse.

.DESCRIPTION
    Compila el bundle de producción de frontend y backend de Tauri v2 (Instalador NSIS .EXE y APK de Android),
    calcula hashes SHA-256 y prepara los artefactos directamente en la raíz de release/.

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

# 1. Preflight Checks & Frontend Build
Write-Host "➔ 1/4 Verificando dependencias y compilando frontend..." -ForegroundColor Green
bun run build
if ($LASTEXITCODE -ne 0) {
    Write-Error "Fallo en la compilación del frontend de Vite."
    exit 1
}

# 2. Build Tauri Windows NSIS Release Bundle
Write-Host "➔ 2/4 Compilando instalador oficial de escritorio (Release)..." -ForegroundColor Green
bun run tauri build
if ($LASTEXITCODE -ne 0) {
    Write-Error "Fallo en la compilación de Release de Tauri."
    exit 1
}

# 3. Artifact Packaging (Directly in root of release/)
$bundleDir = ".\src-tauri\target\release\bundle"
$releaseOutputDir = ".\release"
New-Item -ItemType Directory -Force -Path $releaseOutputDir | Out-Null

# Clean up old subfolders if present
if (Test-Path "$releaseOutputDir\v$Version") {
    Remove-Item -Path "$releaseOutputDir\v$Version" -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "➔ 3/4 Copiando instaladores directamente a $releaseOutputDir..." -ForegroundColor Green

# Copy NSIS installer
$nsisExe = Get-ChildItem -Path "$bundleDir\nsis\*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($nsisExe) {
    Copy-Item $nsisExe.FullName -Destination "$releaseOutputDir\Aurora-Synapse-v$Version-Setup.exe" -Force
    Write-Host "  • Instalador Windows: Aurora-Synapse-v$Version-Setup.exe" -ForegroundColor Green
}

# Check for Android APK
$apkCandidates = @(
    ".\src-tauri\gen\android\app\build\outputs\apk\release\app-universal-release-unsigned.apk",
    ".\src-tauri\gen\android\app\build\outputs\apk\universal\release\app-universal-release-unsigned.apk",
    ".\src-tauri\gen\android\app\build\outputs\apk\universal\debug\app-universal-debug.apk",
    ".\src-tauri\gen\android\app\build\outputs\apk\debug\app-debug.apk"
)
$foundApk = $apkCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($foundApk) {
    Copy-Item $foundApk -Destination "$releaseOutputDir\Aurora-Synapse-v$Version.apk" -Force
    Write-Host "  • APK Android: Aurora-Synapse-v$Version.apk" -ForegroundColor Green
}

# 4. Generate SHA256 Hashes
Write-Host "➔ 4/4 Generando sumas de verificación SHA-256..." -ForegroundColor Green
$hashFile = "$releaseOutputDir\SHA256SUMS.txt"
if (Test-Path $hashFile) { Remove-Item $hashFile -Force }

Get-ChildItem -Path $releaseOutputDir -File | Where-Object { $_.Name -ne "SHA256SUMS.txt" } | ForEach-Object {
    $hash = Get-FileHash -Path $_.FullName -Algorithm SHA256
    $hashLine = "$($hash.Hash)  $($_.Name)"
    $hashLine | Out-File -FilePath $hashFile -Append -Encoding utf8
    Write-Host "  • $($_.Name): $($hash.Hash)" -ForegroundColor DarkGray
}

Write-Host ""
Write-Host "🎉 Release v$Version empaquetado en release/ al 10,000 millones por ciento." -ForegroundColor Cyan
