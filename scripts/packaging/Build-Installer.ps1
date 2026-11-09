<#
.SYNOPSIS
    Compila el instalador de Windows (Inno Setup, plantilla Biglex Modern 2.0)
    de Aurora Synapse.

.DESCRIPTION
    Flujo completo:
      1. Compila la app con Tauri en modo --no-bundle.
      2. Comprueba que el ejecutable compilado NO sea mas viejo que el codigo
         fuente ni que dist/ (ver Assert-BinaryFresh mas abajo).
      3. Regenera packaging\windows\version.iss con la version real de
         package.json.
      4. Compila setup.iss con ISCC.exe.

    Uso:
      pwsh -File scripts\packaging\Build-Installer.ps1
      pwsh -File scripts\packaging\Build-Installer.ps1 -SkipAppBuild
      pwsh -File scripts\packaging\Build-Installer.ps1 -SkipAppBuild -ForceStale
#>
[CmdletBinding()]
param(
    # Reutiliza el ejecutable ya compilado en lugar de recompilar la app.
    [switch]$SkipAppBuild,
    # Empaqueta aunque el ejecutable sea mas viejo que las fuentes. Solo para
    # depurar: publicar asi reintroduce el binario obsoleto.
    [switch]$ForceStale,
    # Ruta al compilador de Inno Setup. Si se deja vacia se localiza relativa
    # al repositorio y, en su defecto, en el PATH (ver Resolve-Iscc).
    [string]$IsccPath = ''
)

$ErrorActionPreference = 'Stop'

$root      = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$exePath   = Join-Path $root 'src-tauri\target\release\aurora-synapse.exe'

function Write-Step([string]$Message) {
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Write-Fail([string]$Message) {
    Write-Host "ERROR: $Message" -ForegroundColor Red
    exit 1
}

# --- Candado anti-binario-obsoleto ------------------------------------------
# Tauri incrusta dist/ DENTRO del ejecutable. Si el .exe es mas viejo que las
# fuentes o que dist/, el instalador empaqueta un binario obsoleto aunque se
# compile "recien" y aunque dist/ este al dia. Es el fallo que produjo el
# instalador de Ely Intelligence del 22-10-2026 13:27 con un binario del 21-10.
function Get-NewestWriteTime {
    param([string[]]$Paths)
    $newest = $null
    $newestPath = $null
    foreach ($path in $Paths) {
        if (-not (Test-Path -LiteralPath $path)) { continue }
        $item = if (Test-Path -LiteralPath $path -PathType Container) {
            Get-ChildItem -LiteralPath $path -Recurse -File -Force -ErrorAction SilentlyContinue |
                Sort-Object LastWriteTime -Descending | Select-Object -First 1
        }
        else {
            Get-Item -LiteralPath $path -Force -ErrorAction SilentlyContinue
        }
        if ($item -and (-not $newest -or $item.LastWriteTime -gt $newest)) {
            $newest = $item.LastWriteTime
            $newestPath = $item.FullName
        }
    }
    return [pscustomobject]@{ Time = $newest; Path = $newestPath }
}

function Assert-BinaryFresh {
    param([string]$Exe, [switch]$Force)
    if (-not (Test-Path -LiteralPath $Exe)) {
        Write-Fail "No se encontro el ejecutable compilado: $Exe`n  Compila la app sin -SkipAppBuild."
    }
    $exeItem = Get-Item -LiteralPath $Exe
    $watch = @(
        (Join-Path $root 'src'),
        (Join-Path $root 'src-tauri\src'),
        (Join-Path $root 'dist'),
        (Join-Path $root 'src-tauri\Cargo.toml'),
        (Join-Path $root 'src-tauri\tauri.conf.json'),
        (Join-Path $root 'src-tauri\build.rs'),
        (Join-Path $root 'src-tauri\capabilities'),
        (Join-Path $root 'package.json')
    )
    $newest = Get-NewestWriteTime -Paths $watch
    if (-not $newest.Time) { return }
    if ($exeItem.LastWriteTime -ge $newest.Time) {
        Write-Step "Ejecutable al dia: $($exeItem.LastWriteTime.ToString('dd/MM HH:mm:ss'))"
        return
    }
    $rel = if ($newest.Path) { $newest.Path.Substring($root.Length).TrimStart('\') } else { '(desconocido)' }
    if ($Force) {
        Write-Host "AVISO: se empaqueta un binario OBSOLETO por -ForceStale." -ForegroundColor Yellow
        Write-Host "       exe $($exeItem.LastWriteTime.ToString('dd/MM HH:mm:ss')) < fuente $($newest.Time.ToString('dd/MM HH:mm:ss')) $rel" -ForegroundColor Yellow
        return
    }
    Write-Fail ("El ejecutable es MAS VIEJO que el codigo fuente:`n" +
                "  ejecutable : $($exeItem.LastWriteTime.ToString('dd/MM HH:mm:ss:ss'))  $Exe`n" +
                "  fuente     : $($newest.Time.ToString('dd/MM HH:mm:ss'))  $rel`n`n" +
                "Tauri incrusta dist/ dentro del ejecutable, asi que un .exe viejo publica`n" +
                "tambien un frontend viejo aunque dist/ este al dia. Vuelve a compilar la app`n" +
                "(sin -SkipAppBuild), o usa -ForceStale solo para depurar.")
}

# Localiza ISCC.exe sin rutas absolutas: primero la copia del workspace,
# subiendo desde este script; despues el PATH del sistema.
function Resolve-Iscc {
    $dir = $PSScriptRoot
    for ($i = 0; $i -lt 8; $i++) {
        $candidate = Join-Path $dir 'Scripts\tools\InnoSetup\ISCC.exe'
        if (Test-Path -LiteralPath $candidate) { return $candidate }
        $parent = Split-Path -Parent $dir
        if (-not $parent -or $parent -eq $dir) { break }
        $dir = $parent
    }
    $onPath = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    if ($onPath) { return $onPath.Source }
    return $null
}

# --- Puerta de calidad de empaquetado ------------------------------------------
# Localiza la carpeta compartida Scripts\tools\inno subiendo desde este script.
# Si la app se usa fuera de este workspace no se encuentra y la validacion se
# omite con un aviso: el builder no debe romperse por estar aislado.
function Resolve-SharedInno {
    $dir = $PSScriptRoot
    for ($i = 0; $i -lt 8; $i++) {
        $candidate = Join-Path $dir 'Scripts\tools\inno'
        if (Test-Path -LiteralPath $candidate) { return $candidate }
        $parent = Split-Path -Parent $dir
        if (-not $parent -or $parent -eq $dir) { break }
        $dir = $parent
    }
    return $null
}

# Copia la plantilla visual compartida sobre la copia local. Asi el repositorio
# refleja SIEMPRE lo que se compilo y no puede haber drift entre apps.
function Sync-SharedTemplate {
    param([string]$SharedDir, [string]$PackagingDir)
    if (-not $SharedDir) { return }
    $src = Join-Path $SharedDir 'BiglexModernUI.iss'
    $dst = Join-Path $PackagingDir 'BiglexModernUI.iss'
    if ((Test-Path -LiteralPath $src) -and (Test-Path -LiteralPath $PackagingDir)) {
        Copy-Item -LiteralPath $src -Destination $dst -Force
    }
}

# Valida las invariantes de empaquetado antes de compilar: si algo esta mal,
# no se genera un instalador con el arranque defectuoso (0xc00004bc) ni un
# nombre con el sufijo obsoleto _x64.
function Assert-PackagingInvariants {
    param([string]$SharedDir, [string[]]$SetupFiles)
    if (-not $SharedDir) {
        Write-Host 'AVISO: no se encontro la carpeta compartida Scripts\tools\inno.' -ForegroundColor Yellow
        Write-Host '       Se omite la validacion de invariantes de empaquetado.' -ForegroundColor Yellow
        return
    }
    $checker = Join-Path $SharedDir 'Check-Packaging.ps1'
    if (-not (Test-Path -LiteralPath $checker)) {
        Write-Host "AVISO: no se encontro $checker. Se omite la validacion." -ForegroundColor Yellow
        return
    }
    $pwshExe = (Get-Command pwsh -ErrorAction SilentlyContinue).Source
    if (-not $pwshExe) { $pwshExe = (Get-Command powershell -ErrorAction SilentlyContinue).Source }
    foreach ($setup in $SetupFiles) {
        & $pwshExe -NoProfile -ExecutionPolicy Bypass -File $checker -SetupIss $setup
        if ($LASTEXITCODE -ne 0) {
            Write-Host "ERROR: las invariantes de empaquetado fallan para $setup." -ForegroundColor Red
            Write-Host '       No se compila el instalador. Corrige lo anterior.' -ForegroundColor Red
            exit 1
        }
    }
}

# --- 1. Build de la app ------------------------------------------------------
if (-not $SkipAppBuild) {
    Write-Step 'Compilando Aurora Synapse con Tauri (--no-bundle)'
    Push-Location $root
    try {
        & bun run tauri build --no-bundle
        if ($LASTEXITCODE -ne 0) { Write-Fail "La compilacion de Tauri termino con codigo $LASTEXITCODE" }
    }
    finally { Pop-Location }
}
else {
    Write-Step 'Omite la compilacion de Tauri (-SkipAppBuild)'
}

# --- 2. Candado --------------------------------------------------------------
Assert-BinaryFresh -Exe $exePath -Force:$ForceStale

# --- 3. Version real ---------------------------------------------------------
$v = (Get-Content "$root\package.json" -Raw | ConvertFrom-Json).version
if ([string]::IsNullOrWhiteSpace($v)) { Write-Fail 'No se pudo leer la version de package.json' }
[IO.File]::WriteAllText("$root\packaging\windows\version.iss", ('#define MyAppVersion "' + $v + '"'), [Text.UTF8Encoding]::new($true))
Write-Step "version.iss regenerado con la version $v"

# --- 4. Compilacion del instalador -------------------------------------------
# La plantilla compartida se copia y las invariantes se validan ANTES de
# compilar: si el .iss quedo con un arranque directo o con el sufijo _x64, el
# instalador no llega a generarse.
$sharedInno = Resolve-SharedInno
Sync-SharedTemplate -SharedDir $sharedInno -PackagingDir (Join-Path $root 'packaging\windows')
Assert-PackagingInvariants -SharedDir $sharedInno -SetupFiles @((Join-Path $root 'packaging\windows\setup.iss'))

if (-not $IsccPath) { $IsccPath = Resolve-Iscc }
if (-not $IsccPath) { Write-Fail 'No se localizo ISCC.exe en Scripts\tools\InnoSetup ni en el PATH.' }
Write-Step "Compilando instalador con $IsccPath"
& $IsccPath "$root\packaging\windows\setup.iss" /Q
if ($LASTEXITCODE) { Write-Fail "ISCC termino con codigo $LASTEXITCODE" }

$installer = Join-Path $root "release\Aurora-Synapse-v$v-Setup.exe"
if (-not (Test-Path -LiteralPath $installer)) { Write-Fail "No se genero el instalador esperado: $installer" }
Write-Step "Instalador generado: $installer ($((Get-Item $installer).Length) bytes)"

# Sincronizar copia en la carpeta de releases del workspace.
# Sin rutas absolutas al disco local: la carpeta '12. Releases' se deriva de la
# raiz del repositorio (Core: command-scripts-standard, pto 4).
$workspaceRoot = Split-Path -Parent $root
$releasesGlobalDir = Join-Path $workspaceRoot '12. Releases\Aurora-Synapse'
if (Test-Path -LiteralPath (Split-Path $releasesGlobalDir -Parent)) {
    if (-not (Test-Path -LiteralPath $releasesGlobalDir)) {
        New-Item -ItemType Directory -Path $releasesGlobalDir -Force | Out-Null
    }
    $installerLeaf = Split-Path $installer -Leaf
    Get-ChildItem -LiteralPath $releasesGlobalDir -Filter "Aurora-Synapse-v*-Setup.exe" -File |
        Where-Object { $_.Name -ne $installerLeaf } |
        ForEach-Object { Remove-Item -LiteralPath $_.FullName -Force -ErrorAction SilentlyContinue }
    Copy-Item -LiteralPath $installer -Destination (Join-Path $releasesGlobalDir $installerLeaf) -Force
    Write-Step "Copia sincronizada en 12. Releases\Aurora-Synapse\$installerLeaf"
}
Write-Step 'Listo.'