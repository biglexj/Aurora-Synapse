<#
.SYNOPSIS
    Compila e instala Aurora Synapse en un dispositivo Android físico vía ADB inalámbrico o USB.

.DESCRIPTION
    Verifica el entorno Android SDK/NDK, inicializa el target si es necesario y despliega
    el APK directamente en el teléfono conectado.
#>

[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"

Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  📱 Aurora Synapse — Android Wireless ADB Deploy Pipeline     " -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan

# 1. Check Connected Devices
$devices = adb devices
Write-Host $devices

$connected = $devices | Where-Object { $_ -match "\bdevice\b" -and $_ -notmatch "List of" }
if (-not $connected) {
    Write-Error "No se detectó ningún dispositivo Android conectado con ADB. Activa la depuración por Wi-Fi o conecta por USB."
    exit 1
}

Write-Host "➔ Dispositivo detectado: $connected" -ForegroundColor Green

# 2. Check Android SDK & NDK
$androidSdk = "C:\Users\biglexj\AppData\Local\Android\Sdk"
$env:ANDROID_HOME = $androidSdk

$ndkPath = Get-ChildItem -Path "$androidSdk\ndk" -Directory -ErrorAction SilentlyContinue | Select-Object -First 1
if ($ndkPath) {
    $env:NDK_HOME = $ndkPath.FullName
    Write-Host "➔ Android NDK detectado en: $env:NDK_HOME" -ForegroundColor Green
} else {
    Write-Warning "No se encontró NDK en $androidSdk\ndk. Instálalo desde Android Studio (SDK Tools -> NDK Side by side)."
}

# 3. Deploy App
Write-Host "➔ Desplegando en Android vía Tauri..." -ForegroundColor Cyan
bun run tauri android dev
