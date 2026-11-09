# Sync & Convert Icons from Source to Aurora-Synapse
# La biblioteca canonica se movio a 03_Proyectos/02_Development/Icons/Applications.
#
# Sin rutas absolutas al disco local (Core: command-scripts-standard, pto 4):
# la raiz del proyecto y la carpeta que la contiene se derivan de la ubicacion
# de este script. La biblioteca de iconos, que vive fuera del workspace, se
# puede fijar con AURORA_ICON_LIBRARY; si no, se busca en la unidad del
# workspace.
$projectRoot = Split-Path -Parent $PSScriptRoot
$workspaceRoot = Split-Path -Parent $projectRoot
$driveRoot = (Split-Path -Qualifier $workspaceRoot) + '\'

$sourceDir = if ($env:AURORA_ICON_LIBRARY) {
    $env:AURORA_ICON_LIBRARY
} else {
    Join-Path $driveRoot 'Imágenes\03_Proyectos\02_Development\Icons\Applications'
}

$targetPublicDir = Join-Path $projectRoot 'public\assets\icons'
$targetAssetsDir = Join-Path $projectRoot 'assets\icons'

$mappings = @{
    "DaVinciFlow"      = "davinci-flow"
    "Ely-Tesia"        = "ely-tesia"
    "ElyIntelligence"  = "ely-intelligence"
    "Gallery-DL-GUI"   = "gallery-dl-gui"
    "Lienzo"           = "lienzo-gallery"
    "LunaYTDLP"        = "luna-fetch"
    "LyraFlow"         = "lyraflow"
    "MouziFlow"        = "mouziflow"
    "Prisma"           = "prisma"
    "WinTTS"           = "wintts"
}

foreach ($srcFolder in $mappings.Keys) {
    $slug = $mappings[$srcFolder]
    $fullSrcPath = Join-Path $sourceDir $srcFolder
    $destPublic = Join-Path $targetPublicDir $slug
    $destAssets = Join-Path $targetAssetsDir $slug

    New-Item -ItemType Directory -Force -Path $destPublic | Out-Null
    New-Item -ItemType Directory -Force -Path $destAssets | Out-Null

    # 1. Main Icon (Standard with background)
    $mainCandidates = @(
        "$fullSrcPath\$srcFolder.webp",
        "$fullSrcPath\$srcFolder.png",
        "$fullSrcPath\icon.webp",
        "$fullSrcPath\icon.png",
        "$fullSrcPath\1080x1080.png",
        "$fullSrcPath\300x300.png"
    )
    $mainSrc = $mainCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1

    if ($mainSrc) {
        Write-Host "Converting Standard Icon for $slug ($mainSrc)..."
        if ($mainSrc.EndsWith(".webp")) {
            Copy-Item "$mainSrc" "$destPublic\icon.webp" -Force
            Copy-Item "$mainSrc" "$destAssets\icon.webp" -Force
            $companionPng = [System.IO.Path]::ChangeExtension($mainSrc, ".png")
            if (Test-Path $companionPng) {
                Copy-Item "$companionPng" "$destPublic\icon.png" -Force
                Copy-Item "$companionPng" "$destAssets\icon.png" -Force
            } else {
                & magick "$mainSrc" "$destPublic\icon.png"
                & magick "$mainSrc" "$destAssets\icon.png"
            }
        } else {
            # Convert to WebP using ImageMagick
            & magick "$mainSrc" -quality 92 "$destPublic\icon.webp"
            & magick "$mainSrc" -quality 92 "$destAssets\icon.webp"
            Copy-Item "$mainSrc" "$destPublic\icon.png" -Force
            Copy-Item "$mainSrc" "$destAssets\icon.png" -Force
        }
    }

    # 2. Transparent Icon (Emblem variant)
    $transCandidates = @(
        "$fullSrcPath\$srcFolder-transparent.webp",
        "$fullSrcPath\$srcFolder-transparent.png",
        "$fullSrcPath\icon-transparent.webp",
        "$fullSrcPath\icon-transparent.png",
        "$fullSrcPath\LyraFlow-transparent.webp",
        "$fullSrcPath\LyraFlow-transparent.png",
        "$fullSrcPath\WinTTS-transparent.webp",
        "$fullSrcPath\WinTTS-transparent.png",
        "$fullSrcPath\MouziFlow-transparent.webp",
        "$fullSrcPath\MouziFlow-transparent.png",
        "$fullSrcPath\Prisma-transparent.webp",
        "$fullSrcPath\Prisma-transparent.png",
        "$fullSrcPath\Gallery-DL-GUI-transparent.webp",
        "$fullSrcPath\Gallery-DL-GUI-transparent.png",
        "$fullSrcPath\LunaYTDLP-transparent.webp",
        "$fullSrcPath\LunaYTDLP-transparent.png"
    )
    $transSrc = $transCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1

    if ($transSrc) {
        Write-Host "Converting Transparent Icon for $slug ($transSrc)..."
        if ($transSrc.EndsWith(".webp")) {
            Copy-Item "$transSrc" "$destPublic\icon-transparent.webp" -Force
            Copy-Item "$transSrc" "$destAssets\icon-transparent.webp" -Force
            $companionPng = [System.IO.Path]::ChangeExtension($transSrc, ".png")
            if (Test-Path $companionPng) {
                Copy-Item "$companionPng" "$destPublic\icon-transparent.png" -Force
                Copy-Item "$companionPng" "$destAssets\icon-transparent.png" -Force
            } else {
                & magick "$transSrc" "$destPublic\icon-transparent.png"
                & magick "$transSrc" "$destAssets\icon-transparent.png"
            }
        } else {
            & magick "$transSrc" -quality 92 "$destPublic\icon-transparent.webp"
            & magick "$transSrc" -quality 92 "$destAssets\icon-transparent.webp"
            Copy-Item "$transSrc" "$destPublic\icon-transparent.png" -Force
            Copy-Item "$transSrc" "$destAssets\icon-transparent.png" -Force
        }
    }
}

# 3. Pixi Store & Ely Intelligence
if (Test-Path "$targetAssetsDir\pixi-store\biglexstore.png") {
    & magick "$targetAssetsDir\pixi-store\biglexstore.png" -quality 92 "$targetPublicDir\pixi-store\icon.webp"
    & magick "$targetAssetsDir\pixi-store\biglexstore.png" -quality 92 "$targetAssetsDir\pixi-store\icon.webp"
}

if (Test-Path "$targetAssetsDir\ely-intelligence\ely-intelligence.webp") {
    Copy-Item "$targetAssetsDir\ely-intelligence\ely-intelligence.webp" "$targetPublicDir\ely-intelligence\icon.webp" -Force
    Copy-Item "$targetAssetsDir\ely-intelligence\ely-intelligence.webp" "$targetAssetsDir\ely-intelligence\icon.webp" -Force
}

# 4. Aurora Synapse Branding Icons
$auroraSrcAppDir = Join-Path $sourceDir "Aurora-Synapse"
$brandingSrcDir = Join-Path $projectRoot 'assets\branding\icons'
$brandingDestDir = Join-Path $projectRoot 'public\assets\branding\icons'
New-Item -ItemType Directory -Force -Path $brandingSrcDir | Out-Null
New-Item -ItemType Directory -Force -Path $brandingDestDir | Out-Null

if (Test-Path $auroraSrcAppDir) {
    $mainWebp = Join-Path $auroraSrcAppDir "Aurora-Synapse.webp"
    $transWebp = Join-Path $auroraSrcAppDir "Aurora-Synapse-transparent.webp"
    if (Test-Path $mainWebp) {
        Copy-Item $mainWebp "$brandingSrcDir\icon.webp" -Force
        Copy-Item $mainWebp "$brandingDestDir\icon.webp" -Force
        & magick $mainWebp "$brandingSrcDir\icon.png"
        & magick $mainWebp "$brandingDestDir\icon.png"
    }
    if (Test-Path $transWebp) {
        Copy-Item $transWebp "$brandingSrcDir\icon-transparent.webp" -Force
        Copy-Item $transWebp "$brandingDestDir\icon-transparent.webp" -Force
        & magick $transWebp "$brandingSrcDir\icon-transparent.png"
        & magick $transWebp "$brandingDestDir\icon-transparent.png"
    }
} elseif (Test-Path $brandingSrcDir) {
    Copy-Item "$brandingSrcDir\*" "$brandingDestDir\" -Force
}

# 4b. Iconos de la app para Tauri (escritorio + launcher de Android) ----------
# Sin este paso, NADA de esta carpeta llegaba al icono del telefono: el set de
# src-tauri\icons (que es lo que alimenta al launcher de Android) se generaba
# aparte y el artwork de telefono se veia recortado.
#
# Android usa iconos adaptativos de 108x108dp y la mascara del launcher solo
# deja ver el circulo interior de 66dp. El artwork debe caber AHI dentro o los
# brazos del emblema se cortan. Por eso la capa foreground se genera con el
# artwork reducido al 48% del lienzo (1000px -> 480px = 51.8dp) y fondo
# transparente; la capa background aporta el color.
$projectDir = $projectRoot
$tauriIconDir = Join-Path $brandingSrcDir "tauri"
New-Item -ItemType Directory -Force -Path $tauriIconDir | Out-Null

# Arte maxima dentro del lienzo de 1000px (48% -> 51.8dp, dentro del circulo
# seguro de 66dp) y porcentaje del lienzo para el icono legacy sin adaptativo.
$androidFgContentPx = 480
$androidFgScale = 72

if (Test-Path $mainWebp) {
    & magick $mainWebp -resize 1024x1024 "$tauriIconDir\icon.png"
}
if (Test-Path $transWebp) {
    # -trim quita el margen transparente del webp y -resize deja el artwork
    # dentro de la zona segura; -extent lo centra en un lienzo 1000x1000.
    & magick $transWebp -trim +repage -resize "${androidFgContentPx}x${androidFgContentPx}>" `
        -gravity center -background none -extent 1000x1000 "$tauriIconDir\icon-android-fg.png"
}
& magick -size 1000x1000 xc:white "$tauriIconDir\icon-android-bg.png"

$iconManifest = [ordered]@{
    default          = "icon.png"
    bg_color         = "#ffffff"
    android_fg       = "icon-android-fg.png"
    android_bg       = "icon-android-bg.png"
    android_fg_scale = $androidFgScale
}
$iconManifestPath = Join-Path $tauriIconDir "icon-manifest.json"
# Sin BOM: serde_json no acepta BOM al inicio del fichero.
[System.IO.File]::WriteAllText(
    $iconManifestPath,
    ($iconManifest | ConvertTo-Json),
    [System.Text.UTF8Encoding]::new($false)
)

Write-Host "Regenerando iconos de Tauri (escritorio + launcher de Android)..." -ForegroundColor Green
$tauriIconsOut = Join-Path $projectDir "src-tauri\icons"
Push-Location $projectDir
try {
    & bun run tauri icon "$iconManifestPath" --output "$tauriIconsOut"
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Fallo la generacion de iconos de Tauri."
        exit 1
    }
}
finally { Pop-Location }

# El proyecto no tiene target iOS, asi que tauri icon deja un directorio ios
# huerfano dentro de src-tauri\icons. Se elimina para no ensuciar el repo.
$strayIosDir = Join-Path $tauriIconsOut "ios"
if (Test-Path $strayIosDir) {
    Remove-Item $strayIosDir -Recurse -Force
}
# 5. Iconos del ecosistema desde los repos del proyecto ---------------------
# La biblioteca central de iconos (biblioteca canonica, ver AURORA_ICON_LIBRARY)
# ya no cubre estas apps, asi que su fuente canonica es el propio repositorio de cada una.
# Se generan aqui para que el catalogo de Aurora Synapse sea reproducible.
$ecosystemRoots = @{
    "aurora"          = Join-Path $workspaceRoot 'Aurora---Blog\AuroraHub\src-tauri\icons\128x128.png'
    "prisma-upscaler" = Join-Path $workspaceRoot 'Prisma-Upscaler\assets\branding\icons\icon.png'
    "nubix"           = Join-Path $workspaceRoot 'Nubix\src-tauri\icons\128x128.png'
}
$ecosystemTransparent = @{
    "prisma-upscaler" = Join-Path $workspaceRoot 'Prisma-Upscaler\assets\branding\icons\icon-transparent.png'
    "nubix"           = Join-Path $workspaceRoot 'Nubix\assets\branding\icons\icon-transparent.png'
}

foreach ($slug in $ecosystemRoots.Keys) {
    $src = $ecosystemRoots[$slug]
    if (-not (Test-Path -LiteralPath $src)) {
        Write-Host "OMITIDO ${slug}: no existe $src" -ForegroundColor Yellow
        continue
    }
    $dest = Join-Path $targetPublicDir $slug
    New-Item -ItemType Directory -Force -Path $dest | Out-Null
    & magick "$src" -quality 92 "$dest\icon.webp"
    & magick "$src" "$dest\icon.png"
    Write-Host "Icono del ecosistema generado: $slug" -ForegroundColor Green
}

foreach ($slug in $ecosystemTransparent.Keys) {
    $src = $ecosystemTransparent[$slug]
    if (-not (Test-Path -LiteralPath $src)) { continue }
    $dest = Join-Path $targetPublicDir $slug
    New-Item -ItemType Directory -Force -Path $dest | Out-Null
    & magick "$src" -quality 92 "$dest\icon-transparent.webp"
}

Write-Host "✅ Icons synchronized and converted to WebP successfully!"
