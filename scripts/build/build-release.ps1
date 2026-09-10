<#
.SYNOPSIS
    Script oficial de lanzamiento de Release para Aurora Synapse.

.DESCRIPTION
    Implementa el estándar universal de Core-Docs:
    - Compilación de frontend y binario Tauri v2 para Windows (Setup .exe único con icono NSIS).
    - Compilación opcional de APK universal autónomo para Android.
    - Generación de sumas criptográficas de integridad SHA256SUMS.txt.
    - Registro en Git (commit, tag vX.Y.Z).
    - Publicación atómica en GitHub Releases vía GitHub CLI (gh).
    - Notificación de metadata de la release a Aurora Blog (biglexj.com).

.PARAMETER Version
    Versión semántica del lanzamiento (e.g. "1.0.0"). Si se omite, se extrae de package.json.

.PARAMETER ReleaseNotes
    Notas opcionales del lanzamiento. Si se omite, se lee RELEASE_MESSAGE.md.

.PARAMETER LocalOnly
    Omite la publicación en GitHub y la notificación web, completando solo el empaquetado local.

.PARAMETER SkipBuild
    Omite las etapas de compilación de código fuente si los artefactos ya fueron construidos.

.PARAMETER SkipAndroid
    Omite la compilación del APK de Android.

.PARAMETER SkipAuroraUpload
    Omite la notificación HTTP de metadata a biglexj.com.
#>

[CmdletBinding()]
param(
    [string]$Version,
    [string]$ReleaseNotes,
    [switch]$LocalOnly,
    [switch]$SkipBuild,
    [switch]$SkipAndroid,
    [switch]$SkipAuroraUpload
)

$ErrorActionPreference = "Stop"

$root = if (Test-Path (Join-Path $PSScriptRoot "package.json")) {
    $PSScriptRoot
} elseif (Test-Path (Join-Path $PSScriptRoot "..\package.json")) {
    (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
} else {
    (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
}

$releaseDir = Join-Path $root "release"
$packageFile = Join-Path $root "package.json"

# 1. Determinar Versión Canónica
if (-not $Version) {
    if (Test-Path $packageFile) {
        $packageJson = Get-Content $packageFile -Raw -Encoding UTF8 | ConvertFrom-Json
        $Version = $packageJson.version
    }
    if (-not $Version) {
        $Version = "1.0.0"
    }
}

Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  ⚡ Aurora Synapse — Official Release Pipeline v$Version      " -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

# 2. Compilación de Artefactos
if (-not $SkipBuild) {
    Set-Location $root

    # 2.1 Frontend Build
    Write-Host "[1/5] Compilando frontend estático (Vite + TS)..." -ForegroundColor Yellow
    bun run build
    if ($LASTEXITCODE -ne 0) { throw "Error compilando frontend estático." }

    # 2.2 Windows Desktop Build (Tauri v2 + NSIS)
    Write-Host "[2/5] Compilando ejecutable e instalador oficial de Windows..." -ForegroundColor Yellow
    bun run tauri build
    if ($LASTEXITCODE -ne 0) { throw "La compilación de escritorio falló con código $LASTEXITCODE" }

    # 2.3 Android Universal APK Build (si no se omite)
    if (-not $SkipAndroid) {
        $androidSdk = "C:\Users\biglexj\AppData\Local\Android\Sdk"
        if (Test-Path $androidSdk) {
            Write-Host "[3/5] Compilando APK universal autónomo de Android..." -ForegroundColor Yellow
            $env:ANDROID_HOME = $androidSdk
            $ndk = Get-ChildItem -Path "$androidSdk\ndk" -Directory -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($ndk) { $env:NDK_HOME = $ndk.FullName }

            $cargoTomlPath = ".\src-tauri\Cargo.toml"
            $cargoContent = Get-Content $cargoTomlPath -Raw
            if ($cargoContent -notmatch 'crate-type\s*=\s*\[.*"cdylib".*\]') {
                $updatedCargo = $cargoContent -replace 'crate-type\s*=\s*\[(.*?)\]', 'crate-type = ["staticlib", "cdylib", "rlib"]'
                Set-Content -Path $cargoTomlPath -Value $updatedCargo -NoNewline
            }

            bun run tauri android build --apk --debug

            # Restaurar crate-type para desktop
            $cargoContent = Get-Content $cargoTomlPath -Raw
            $restoredCargo = $cargoContent -replace 'crate-type\s*=\s*\[.*?"cdylib".*?\]', 'crate-type = ["staticlib", "rlib"]'
            Set-Content -Path $cargoTomlPath -Value $restoredCargo -NoNewline
        } else {
            Write-Host "[3/5] Android SDK no detectado; omitiendo compilación de APK." -ForegroundColor DarkGray
        }
    } else {
        Write-Host "[3/5] Compilación Android omitida (-SkipAndroid)." -ForegroundColor DarkGray
    }
} else {
    Write-Host "[1-3/5] Compilación omitida (-SkipBuild)..." -ForegroundColor DarkGray
}

# 3. Empaquetado en Directorio de Distribución release/
if (-not (Test-Path $releaseDir)) {
    New-Item -ItemType Directory -Force -Path $releaseDir | Out-Null
}

$bundleDir = Join-Path $root "src-tauri\target\release\bundle"
$installerTarget = Join-Path $releaseDir "Aurora-Synapse-v$Version-Setup.exe"
$apkTarget = Join-Path $releaseDir "Aurora-Synapse-v$Version.apk"

# Copiar instalador Windows
$nsisFound = Get-ChildItem -Path "$bundleDir\nsis\*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($nsisFound) {
    Copy-Item $nsisFound.FullName -Destination $installerTarget -Force
    Write-Host "  • Instalador Windows: $installerTarget" -ForegroundColor Green
} elseif (-not (Test-Path $installerTarget)) {
    throw "No se encontró el instalador generado en $bundleDir\nsis ni en $installerTarget"
}

# Copiar APK Android si existe
$apkCandidates = @(
    (Join-Path $root "src-tauri\gen\android\app\build\outputs\apk\universal\debug\app-universal-debug.apk"),
    (Join-Path $root "src-tauri\gen\android\app\build\outputs\apk\universal\release\app-universal-release-unsigned.apk"),
    (Join-Path $root "src-tauri\gen\android\app\build\outputs\apk\debug\app-debug.apk")
)
$foundApk = $apkCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($foundApk) {
    Copy-Item $foundApk -Destination $apkTarget -Force
    Write-Host "  • APK Android: $apkTarget" -ForegroundColor Green
}

# Generar Checksums SHA-256
Write-Host "[4/5] Generando sumas de verificación criptográficas SHA-256..." -ForegroundColor Yellow
$hashFile = Join-Path $releaseDir "SHA256SUMS.txt"
if (Test-Path $hashFile) { Remove-Item $hashFile -Force }

Get-ChildItem -Path $releaseDir -File | Where-Object { $_.Name -ne "SHA256SUMS.txt" } | ForEach-Object {
    $hash = Get-FileHash -Path $_.FullName -Algorithm SHA256
    "$($hash.Hash)  $($_.Name)" | Out-File -FilePath $hashFile -Append -Encoding utf8
    Write-Host "    $($_.Name) -> $($hash.Hash)" -ForegroundColor DarkGray
}

if ($LocalOnly) {
    Write-Host ""
    Write-Host "✅ Empaquetado local completado (-LocalOnly)." -ForegroundColor Green
    exit 0
}

# 4. Registro en Git (Commit y Tag Atómico)
Write-Host "[5/5] Sincronizando Git y publicando release oficial..." -ForegroundColor Yellow
Set-Location $root

$currentBranch = (& git branch --show-current).Trim()
if (-not $currentBranch) { $currentBranch = "master" }

git add .
$hasStagedChanges = (& git status --porcelain)
if ($hasStagedChanges) {
    git commit -m "release: v$Version - Publicación oficial Aurora Synapse"
}
git tag -a "v$Version" -m "Release v$Version" -f

$hasRemote = (& git remote)
if ($hasRemote) {
    git push origin "$currentBranch" --tags --force
}

# 5. Publicación en GitHub Releases
$releaseNotesFile = Join-Path $root "RELEASE_MESSAGE.md"
try { gh release delete "v$Version" --yes 2>$null } catch {}

$assetsToUpload = @($installerTarget, $hashFile)
if (Test-Path $apkTarget) { $assetsToUpload += $apkTarget }

if (-not $ReleaseNotes) {
    if (Test-Path $releaseNotesFile) {
        gh release create "v$Version" @assetsToUpload --title "Aurora Synapse v$Version" -F $releaseNotesFile
    } else {
        gh release create "v$Version" @assetsToUpload --title "Aurora Synapse v$Version" --notes "Lanzamiento oficial de Aurora Synapse v$Version"
    }
} else {
    gh release create "v$Version" @assetsToUpload --title "Aurora Synapse v$Version" --notes $ReleaseNotes
}

$githubAssetUrl = "https://github.com/biglexj/Aurora-Synapse/releases/download/v$Version/Aurora-Synapse-v$Version-Setup.exe"

Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "  🎉 ¡Release v$Version publicado en GitHub con éxito!        " -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Green

# 6. Notificar a Aurora Blog (biglexj.com) — SOLO METADATA (Estándar Core)
if ($SkipAuroraUpload) {
    Write-Host "Notificación a Aurora Blog omitida (-SkipAuroraUpload)." -ForegroundColor DarkGray
} else {
    Write-Host "Notificando metadata a Aurora Blog (biglexj.com)..." -ForegroundColor Yellow

    $auroraEnvPath = "D:\Proyectos\biglexj\Aurora---Blog\frontend\.env"
    if (Test-Path $auroraEnvPath) {
        $auroraEnv = Get-Content $auroraEnvPath -Raw -Encoding UTF8
        $serviceKey = [regex]::Match($auroraEnv, '(?m)^SUPABASE_SERVICE_ROLE_KEY=(.+)$').Groups[1].Value.Trim()

        if ($serviceKey) {
            $baseUrl = "https://www.biglexj.com"
            $slug = "aurora-synapse"

            $releaseMsg = if (Test-Path $releaseNotesFile) {
                [System.IO.File]::ReadAllText($releaseNotesFile, [System.Text.Encoding]::UTF8)
            } else { "Aurora Synapse v$Version" }

            try {
                $sha256Hash = (Get-FileHash -Path $installerTarget -Algorithm SHA256).Hash.ToLower()
                $vParts = $Version.Split('.')
                $calcVersionCode = if ($vParts.Length -ge 3) {
                    [int]$vParts[0] * 10000 + [int]$vParts[1] * 100 + [int]$vParts[2]
                } else { 10000 }

                $releaseBody = @{
                    slug           = $slug
                    downloadUrl    = $githubAssetUrl
                    versionName    = $Version
                    versionCode    = $calcVersionCode
                    releaseNotes   = $releaseMsg
                    sha256Checksum = $sha256Hash
                } | ConvertTo-Json -Depth 5

                Invoke-RestMethod -Uri "$baseUrl/api/admin/developer-apps" -Method PUT `
                                  -Headers @{
                                      "Content-Type" = "application/json"
                                      Authorization  = "Bearer $serviceKey"
                                  } `
                                  -Body $releaseBody | Out-Null

                Write-Host "  ✅ Aurora Synapse v$Version registrado exitosamente en biglexj.com" -ForegroundColor Cyan
            } catch {
                Write-Host "  ⚠️ Error durante la notificación a biglexj.com: $_" -ForegroundColor DarkYellow
            }
        }
    }
}
