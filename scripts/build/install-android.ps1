<#
.SYNOPSIS
    Compila e instala el APK autónomo de Aurora Synapse en un dispositivo Android físico vía ADB.

.DESCRIPTION
    Verifica el entorno Android SDK/NDK, compila el frontend estático y el APK completo
    de Android para que funcione 100% independiente y sin pantalla en blanco.
#>

[CmdletBinding()]
param(
    [switch]$Dev = $false
)

$ErrorActionPreference = "Stop"

Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  📱 Aurora Synapse — Android Standalone ADB Deploy Pipeline   " -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan

# 1. Check Connected Devices
$devices = adb devices
Write-Host $devices

$connectedLine = $devices | Where-Object { $_ -match "\bdevice\b" -and $_ -notmatch "List of" } | Select-Object -First 1
if (-not $connectedLine) {
    Write-Error "No se detectó ningún dispositivo Android conectado con ADB. Activa la depuración por Wi-Fi o conecta por USB."
    exit 1
}

$deviceId = ($connectedLine.Trim() -split "\s+")[0]
Write-Host "➔ Dispositivo seleccionado: $deviceId" -ForegroundColor Green

# 2. Check Android SDK & NDK
$androidSdk = "C:\Users\biglexj\AppData\Local\Android\Sdk"
$env:ANDROID_HOME = $androidSdk

$ndkPath = Get-ChildItem -Path "$androidSdk\ndk" -Directory -ErrorAction SilentlyContinue | Select-Object -First 1
if ($ndkPath) {
    $env:NDK_HOME = $ndkPath.FullName
    Write-Host "➔ Android NDK detectado en: $env:NDK_HOME" -ForegroundColor Green
} else {
    Write-Warning "No se encontró NDK en $androidSdk\ndk."
}

if ($Dev) {
    Write-Host "➔ Modo Dev interactivo con recarga en caliente..." -ForegroundColor Cyan
    bun run tauri android dev
    exit 0
}

# 3. Build Standalone Production APK
Write-Host "➔ 1/3 Compilando bundle frontend estático..." -ForegroundColor Green
bun run build
if ($LASTEXITCODE -ne 0) {
    Write-Error "Error compilando frontend Vite."
    exit 1
}

# Ensure cdylib is enabled for Android JNI
$cargoTomlPath = ".\src-tauri\Cargo.toml"
$cargoContent = Get-Content $cargoTomlPath -Raw
if ($cargoContent -notmatch 'crate-type\s*=\s*\[.*"cdylib".*\]') {
    $updatedCargo = $cargoContent -replace 'crate-type\s*=\s*\[(.*?)\]', 'crate-type = ["staticlib", "cdylib", "rlib"]'
    Set-Content -Path $cargoTomlPath -Value $updatedCargo -NoNewline
}

Write-Host "➔ 2/3 Compilando APK autónomo con Tauri CLI..." -ForegroundColor Green
bun run tauri android build --apk --debug
$buildSuccess = ($LASTEXITCODE -eq 0)

# Restore desktop-friendly crate-type for Windows GNU linker
$cargoContent = Get-Content $cargoTomlPath -Raw
$restoredCargo = $cargoContent -replace 'crate-type\s*=\s*\[.*?"cdylib".*?\]', 'crate-type = ["staticlib", "rlib"]'
Set-Content -Path $cargoTomlPath -Value $restoredCargo -NoNewline

if (-not $buildSuccess) {
    Write-Error "Fallo en la compilación del APK con Tauri Android."
    exit 1
}

# 4. Install via ADB with explicit target device ID
$apkCandidates = @(
    ".\src-tauri\gen\android\app\build\outputs\apk\universal\debug\app-universal-debug.apk",
    ".\src-tauri\gen\android\app\build\outputs\apk\arm64\debug\app-arm64-debug.apk",
    ".\src-tauri\gen\android\app\build\outputs\apk\debug\app-debug.apk"
)

$apkPath = $apkCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $apkPath) {
    Write-Error "No se encontró el APK generado en gen\android\app\build\outputs\apk\"
    exit 1
}

Write-Host "➔ 3/3 Instalando APK autónomo en el teléfono ($apkPath)..." -ForegroundColor Green
adb -s $deviceId install -r $apkPath
if ($LASTEXITCODE -eq 0) {
    Write-Host "➔ Lanzando Aurora Synapse en el dispositivo ($deviceId)..." -ForegroundColor Green
    adb -s $deviceId shell am start -n com.biglexj.aurorasynapse/.MainActivity | Out-Null
    Write-Host ""
    Write-Host "🎉 APK instalado y ejecutándose de forma 100% autónoma al 10,000 millones por ciento." -ForegroundColor Cyan
} else {
    Write-Error "Fallo en la instalación por ADB."
}
