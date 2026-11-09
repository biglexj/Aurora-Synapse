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

# Suma SHA-256 de un archivo.
# No se usa Get-FileHash: en Windows PowerShell 5.1 el PSModulePath puede
# apuntar antes a los modulos de PowerShell 7 (windowsapps\microsoft.powershell_7.*),
# y al cargar Microsoft.PowerShell.Utility se obtiene la version 7, que 5.1 no
# puede usar. El cmdlet queda "no reconocido" y el release se cae al final.
# Esta implementacion con .NET funciona igual en 5.1 y en 7+.
function Get-FileSha256 {
    param([Parameter(Mandatory = $true)][string]$Path)
    $stream = [System.IO.File]::OpenRead($Path)
    try {
        $sha = [System.Security.Cryptography.SHA256]::Create()
        try {
            return ([System.BitConverter]::ToString($sha.ComputeHash($stream))).Replace("-", "")
        }
        finally { $sha.Dispose() }
    }
    finally { $stream.Dispose() }
}

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

    # 2.2 Android Universal APK Build (si no se omite)
    if (-not $SkipAndroid) {
        # Sin rutas absolutas a disco local (Core: command-scripts-standard, pto 4):
        # se respeta ANDROID_HOME/ANDROID_SDK_ROOT y, si no, la carpeta estandar
        # del SDK en el perfil del usuario.
        $androidSdk = if ($env:ANDROID_HOME) { $env:ANDROID_HOME }
                      elseif ($env:ANDROID_SDK_ROOT) { $env:ANDROID_SDK_ROOT }
                      else { Join-Path $env:LOCALAPPDATA 'Android\Sdk' }
        if (Test-Path $androidSdk) {
            Write-Host "[2/5] Compilando APK autónomo de Android (ARM64 / aarch64)..." -ForegroundColor Yellow
            $env:ANDROID_HOME = $androidSdk
            $ndk = Get-ChildItem -Path "$androidSdk\ndk" -Directory -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($ndk) { $env:NDK_HOME = $ndk.FullName }

            $cargoTomlPath = ".\src-tauri\Cargo.toml"
            $cargoContent = Get-Content $cargoTomlPath -Raw
            if ($cargoContent -notmatch 'crate-type\s*=\s*\[.*"cdylib".*\]') {
                $updatedCargo = $cargoContent -replace 'crate-type\s*=\s*\[(.*?)\]', 'crate-type = ["staticlib", "cdylib", "rlib"]'
                Set-Content -Path $cargoTomlPath -Value $updatedCargo -NoNewline
            }

            $androidExit = 0
            try {
                bun run tauri android build --apk --debug --target aarch64
                $androidExit = $LASTEXITCODE
            }
            finally {
                # Restaurar crate-type para desktop
                $cargoContent = Get-Content $cargoTomlPath -Raw
                $restoredCargo = $cargoContent -replace 'crate-type\s*=\s*\[.*?"cdylib".*?\]', 'crate-type = ["staticlib", "rlib"]'
                Set-Content -Path $cargoTomlPath -Value $restoredCargo -NoNewline
            }
            # Un APK fallido no puede pasar desapercibido: el release se publica
            # en GitHub y un artefacto movil ausente es un defecto, no un detalle.
            if ($androidExit -ne 0) {
                throw ("La compilacion del APK de Android fallo con codigo $androidExit.`n" +
                       "Se detiene el release para no publicar un artefacto movil ausente o viejo.`n" +
                       "Usa -SkipAndroid para publicar solo el instalador de Windows.")
            }
        } else {
            Write-Host "[2/5] Android SDK no detectado; omitiendo compilación de APK." -ForegroundColor DarkGray
        }
    } else {
        Write-Host "[2/5] Compilación Android omitida (-SkipAndroid)." -ForegroundColor DarkGray
    }

    # 2.3 Windows Desktop Build (Tauri v2 + Inno Setup)
    # Este paso va DESPUES del de Android a proposito: tanto `tauri build` como
    # `tauri android build` ejecutan el beforeBuildCommand y regeneran dist/.
    # Si el instalador se generase antes, el candado anti-obsoleto de la
    # seccion 3 lo rechazaria por ser unos segundos mas viejo que dist/, pese
    # a contener exactamente el mismo frontend.
    Write-Host "[3/5] Compilando ejecutable e instalador oficial de Windows..." -ForegroundColor Yellow
    & "$root\scripts\packaging\Build-Installer.ps1"
    if ($LASTEXITCODE -ne 0) { throw "La compilación de escritorio falló con código $LASTEXITCODE" }
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
# El empaquetador de Inno escribe directamente en release.
if (-not (Test-Path -LiteralPath $installerTarget)) {
    throw "No se encontró el instalador Inno: $installerTarget"
}

# Candado anti-instalador-obsoleto --------------------------------------------
# Con -SkipBuild no se compila nada, pero el script aun podia publicar un
# instalador de una carrera anterior de la MISMA version. Se comprueba que el
# instalador sea mas nuevo que el codigo fuente y que dist/, igual que hace el
# empaquetador con el ejecutable. Regla de Core:
# Docs/global/architecture/command-scripts-standard.md, principio 2.
$installerItem = Get-Item -LiteralPath $installerTarget
$watch = @(
    (Join-Path $root 'src'),
    (Join-Path $root 'src-tauri\src'),
    (Join-Path $root 'dist'),
    (Join-Path $root 'src-tauri\Cargo.toml'),
    (Join-Path $root 'src-tauri\tauri.conf.json'),
    (Join-Path $root 'package.json')
)
$staleSource = $null
foreach ($p in $watch) {
    if (-not (Test-Path -LiteralPath $p)) { continue }
    $i = if (Test-Path -LiteralPath $p -PathType Container) {
        Get-ChildItem -LiteralPath $p -Recurse -File -Force -ErrorAction SilentlyContinue |
            Sort-Object LastWriteTime -Descending | Select-Object -First 1
    }
    else { Get-Item -LiteralPath $p -Force -ErrorAction SilentlyContinue }
    if ($i -and $i.LastWriteTime -gt $installerItem.LastWriteTime) { $staleSource = $i; break }
}
if ($staleSource) {
    throw ("El instalador es MAS VIEJO que el codigo fuente; no se puede publicar:`n" +
           "  instalador : $($installerItem.LastWriteTime.ToString('dd/MM HH:mm:ss'))  $installerTarget`n" +
           "  fuente     : $($staleSource.LastWriteTime.ToString('dd/MM HH:mm:ss'))  $($staleSource.FullName)`n" +
           "Vuelve a compilar el instalador con scripts\packaging\Build-Installer.ps1.")
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
    $workspaceRoot = Split-Path -Parent $root
    $releasesGlobalDir = Join-Path $workspaceRoot '12. Releases\Aurora-Synapse'
    if (Test-Path -LiteralPath $releasesGlobalDir) {
        Copy-Item -LiteralPath $apkTarget -Destination (Join-Path $releasesGlobalDir (Split-Path $apkTarget -Leaf)) -Force
        Write-Host "==> APK sincronizado en 12. Releases\Aurora-Synapse\$(Split-Path $apkTarget -Leaf)" -ForegroundColor Green
    }
}

# Generar Checksums SHA-256
Write-Host "[4/5] Generando sumas de verificación criptográficas SHA-256..." -ForegroundColor Yellow
$hashFile = Join-Path $releaseDir "SHA256SUMS.txt"
if (Test-Path $hashFile) { Remove-Item $hashFile -Force }

# Solo se resumen los artefactos que se van a publicar en la release. La
# carpeta release/ puede conservar artefactos de versiones anteriores, que no
# se suben: incluirlos aqui generaba entradas huerfanas que el usuario no
# podria verificar porque los ficheros no existirian en la release.
$publishedAssets = @($installerTarget)
if (Test-Path -LiteralPath $apkTarget) { $publishedAssets += $apkTarget }

# Sin BOM: sha256sum -c y las herramientas POSIX fallan con la primera linea
# si el fichero arranca con BOM UTF-8. Se escribe con UTF-8 sin BOM.
$hashLines = New-Object System.Collections.Generic.List[string]
foreach ($asset in $publishedAssets) {
    $hash = Get-FileSha256 -Path $asset
    $hashLines.Add("$hash  $([System.IO.Path]::GetFileName($asset))")
    Write-Host "    $([System.IO.Path]::GetFileName($asset)) -> $hash" -ForegroundColor DarkGray
}
[System.IO.File]::WriteAllText(
    $hashFile,
    (($hashLines -join "`n") + "`n"),
    [System.Text.UTF8Encoding]::new($false)
)
$workspaceRoot = Split-Path -Parent $root
$releasesGlobalDir = Join-Path $workspaceRoot '12. Releases\Aurora-Synapse'
if (Test-Path -LiteralPath $releasesGlobalDir) {
    Copy-Item -LiteralPath $hashFile -Destination (Join-Path $releasesGlobalDir (Split-Path $hashFile -Leaf)) -Force
    Write-Host "==> SHA256SUMS.txt sincronizado en 12. Releases\Aurora-Synapse" -ForegroundColor Green
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

# Se sube exactamente lo que se resumio en el paso 4, mas el fichero de
# sumas. $publishedAssets ya excluye los artefactos historicos de release/.
$assetsToUpload = @($publishedAssets) + $hashFile

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

    # Sin rutas absolutas al disco local (Core: command-scripts-standard, pto 4):
    # el repositorio de Aurora Blog es hermano de este en el workspace.
    $workspaceRoot = Split-Path -Parent $root
    $auroraEnvPath = Join-Path $workspaceRoot 'Aurora---Blog\frontend\.env'
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
                $sha256Hash = (Get-FileSha256 -Path $installerTarget).ToLower()
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
