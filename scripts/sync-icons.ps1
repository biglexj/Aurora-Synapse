# Sync & Convert Icons from Source to Aurora-Synapse
$sourceDir = "D:\Imágenes\Proyectos\DEV\Icon"
$targetPublicDir = "D:\Proyectos\biglexj\Aurora-Synapse\public\assets\icons"
$targetAssetsDir = "D:\Proyectos\biglexj\Aurora-Synapse\Assets\icons"

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
    "codex-go"         = "codex-go"
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
        "$fullSrcPath\$srcFolder.png",
        "$fullSrcPath\icon.png",
        "$fullSrcPath\1080x1080.png",
        "$fullSrcPath\300x300.png"
    )
    $mainSrc = $mainCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1

    if ($mainSrc) {
        Write-Host "Converting Standard Icon for $slug ($mainSrc)..."
        # Convert to WebP using ImageMagick
        & magick "$mainSrc" -quality 92 "$destPublic\icon.webp"
        & magick "$mainSrc" -quality 92 "$destAssets\icon.webp"
        Copy-Item "$mainSrc" "$destPublic\icon.png" -Force
        Copy-Item "$mainSrc" "$destAssets\icon.png" -Force
    }

    # 2. Transparent Icon (Emblem variant)
    $transCandidates = @(
        "$fullSrcPath\$srcFolder-transparent.png",
        "$fullSrcPath\icon-transparent.png",
        "$fullSrcPath\LyraFlow-transparent.png",
        "$fullSrcPath\WinTTS-transparent.png",
        "$fullSrcPath\MouziFlow-transparent.png",
        "$fullSrcPath\Prisma-transparent.png",
        "$fullSrcPath\Gallery-DL-GUI-transparent.png",
        "$fullSrcPath\LunaYTDLP-transparent.png"
    )
    $transSrc = $transCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1

    if ($transSrc) {
        Write-Host "Converting Transparent Icon for $slug ($transSrc)..."
        & magick "$transSrc" -quality 92 "$destPublic\icon-transparent.webp"
        & magick "$transSrc" -quality 92 "$destAssets\icon-transparent.webp"
        Copy-Item "$transSrc" "$destPublic\icon-transparent.png" -Force
        Copy-Item "$transSrc" "$destAssets\icon-transparent.png" -Force
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

Write-Host "✅ Icons synchronized and converted to WebP successfully!"
