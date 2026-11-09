//! Superficies **reales** de invocación de cada app del ecosistema.
//!
//! Estas constantes no provienen del protocolo documentado, sino del código que
//! hoy ejecuta cada aplicación. El protocolo Aurora Synapse (named pipes,
//! envoltorio de acciones genérico) está documentado pero **no implementado**
//! en la mayoría de las apps, así que el puente habla el dialecto que cada app
//! entiende de verdad:
//!
//! | App | Transporte real | Verificado en |
//! | :--- | :--- | :--- |
//! | Luna Fetch | HTTP `49288` + línea TCP `49282` | `SynapseLanServer.kt`, `SynapseDispatcherServer.kt` |
//! | Prisma | HTTP `49290` + línea TCP `49290` | `features/synapse/server.rs` |
//! | Prisma Upscaler | HTTP `8085` (multipart) o CLI `--local` | `server/src/main.rs`, `cli/src/main.rs` |
//! | DaVinci Flow | Servidor MCP nativo JSON-RPC stdio (`davinci_flow.mcp_server`) | `src/davinci_flow/mcp_server.py` |
//!
//! WinTTS, Ely-Tesia, LyraFlow y MouziFlow **no exponen ninguna superficie
//! externa** hoy, así que el puente no publica herramientas para ellas.

use std::path::PathBuf;

/// Puerto del servidor LAN de Luna Fetch (`SynapseLanServer`).
pub const LUNA_LAN_PORT: u16 = 49288;

/// Puerto del despachador de una línea de Luna Fetch (`SynapseDispatcherServer`).
pub const LUNA_DISPATCH_PORT: u16 = 49282;

/// Puerto del servidor Synapse de Prisma.
pub const PRISMA_HTTP_PORT: u16 = 49290;

/// Puerto del microservicio de Prisma Upscaler.
pub const UPSCALER_HTTP_PORT: u16 = 8085;

/// Endpoints de Luna Fetch.
pub const LUNA_PING: &str = "/api/v1/synapse/ping";
pub const LUNA_STATUS: &str = "/api/v1/synapse/status";
pub const LUNA_PUSH_DOWNLOAD: &str = "/api/v1/synapse/push-download";

/// Endpoints de Prisma.
pub const PRISMA_STATUS: &str = "/api/v1/synapse/status";
pub const PRISMA_PLAYBACK: &str = "/api/v1/synapse/playback";
pub const PRISMA_HANDOFF: &str = "/api/v1/synapse/handoff";
pub const PRISMA_REMOTE: &str = "/api/v1/synapse/remote";

/// Endpoints de Prisma Upscaler.
pub const UPSCALER_HEALTH: &str = "/health";
pub const UPSCALER_MODELS: &str = "/api/v1/models";
pub const UPSCALER_UPSCALE: &str = "/api/v1/upscale";

/// Verbos de control de reproducción que Prisma acepta en `POST /remote`.
///
/// Fuente: `features/synapse/server.rs` y su resolución en el frontend.
pub const PRISMA_COMMANDS: &[&str] = &[
    "play_pause",
    "seek_backward",
    "seek_forward",
    "previous",
    "next",
    "shuffle",
    "volume_up",
    "volume_down",
    "toggle_mute",
    "toggle_subtitles",
    "toggle_audio_track",
    "fullscreen",
    "escape",
    "quick_look",
    "lyrics",
    "queue",
    "previa",
    "search",
];

/// Acciones que el despachador de Luna Fetch entiende (`SynapseUriParser`).
pub const LUNA_ACTIONS: &[&str] = &["enqueue_download", "analyze_url", "open_folder", "focus"];

/// Ruta base de los datos locales de la aplicación.
fn local_app_data() -> PathBuf {
    std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// Ejecutable de Luna Fetch.
pub fn luna_exe() -> PathBuf {
    local_app_data()
        .join("Programs")
        .join("LunaFetch")
        .join("LunaFetch.exe")
}

/// Ejecutable de Prisma.
pub fn prisma_exe() -> PathBuf {
    local_app_data().join("Prisma").join("prisma.exe")
}

/// Ejecutable de escritorio de Prisma Upscaler (incluye motor embebido).
pub fn upscaler_desktop_exe() -> PathBuf {
    local_app_data()
        .join("Prisma Upscaler")
        .join("prisma-upscaler-desktop.exe")
}

/// Ejecutable de la CLI de Prisma Upscaler.
pub fn upscaler_cli_exe() -> PathBuf {
    local_app_data()
        .join("Programs")
        .join("PrismaUpscalerCLI")
        .join("upscale.exe")
}

/// Comprueba si un ejecutable existe en disco.
pub fn executable_exists(path: &PathBuf) -> bool {
    path.is_file()
}

/// Carpeta de DaVinci Flow en desarrollo o producción.
pub fn davinci_repo_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("DAVINCIFLOW_DIR") {
        return PathBuf::from(dir);
    }
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let sibling = current_dir
        .parent()
        .map(|p| p.join("DaVinciFlow"))
        .unwrap_or_else(|| PathBuf::from("../DaVinciFlow"));
    if sibling.is_dir() {
        sibling
    } else {
        PathBuf::from("D:\\Proyectos\\biglexj\\DaVinciFlow")
    }
}

/// Intérprete Python configurado para el motor de DaVinci Flow.
pub fn davinci_python_exe() -> PathBuf {
    if let Ok(py) = std::env::var("DAVINCIFLOW_PYTHON") {
        return PathBuf::from(py);
    }
    let repo = davinci_repo_dir();
    #[cfg(windows)]
    let venv_py = repo.join(".venv").join("Scripts").join("python.exe");
    #[cfg(not(windows))]
    let venv_py = repo.join(".venv").join("bin").join("python");

    if venv_py.is_file() {
        return venv_py;
    }

    PathBuf::from("python")
}

/// Apps del ecosistema con su superficie real de invocación.
///
/// `tools` lista las herramientas MCP que el puente puede usar contra esa app.
pub const BRIDGE_TARGETS: &[(&str, &str, &str, &[&str])] = &[
    ("supergallery", "Super Galería", "HTTP 49288 del teléfono; beacons 49289 compartidos con Prisma y autorización mediante código temporal",
        &["gallery_devices", "gallery_status", "gallery_list", "gallery_download", "gallery_send"]),
    (
        "luna",
        "Luna Fetch",
        "HTTP en 127.0.0.1:49288 y despachador de una línea en 127.0.0.1:49282",
        &[
            "luna_status",
            "luna_download",
            "luna_analyze",
            "luna_open_folder",
            "luna_focus",
        ],
    ),
    (
        "prisma",
        "Prisma",
        "HTTP en 127.0.0.1:49290 (control de reproducción y handoff)",
        &[
            "prisma_status",
            "prisma_playback",
            "prisma_play",
            "prisma_control",
        ],
    ),
    (
        "prisma-upscaler",
        "Prisma Upscaler",
        "HTTP en 127.0.0.1:8085 o CLI local instalada; \
         la GUI no ejecuta órdenes completas de escalado",
        &["upscaler_status", "upscaler_models", "upscaler_scale"],
    ),
    (
        "wintts",
        "WinTTS",
        "Sin superficie externa: solo comandos Tauri internos de su propio WebView",
        &[],
    ),
    (
        "elytesia",
        "Ely-Tesia",
        "Sin superficie externa: el esquema elytesia:// solo existe en el manifest de Android",
        &[],
    ),
    (
        "lyraflow",
        "LyraFlow",
        "Sin superficie externa: el puerto 49281 solo trae la ventana al frente, no acepta acciones",
        &[],
    ),
    (
        "mouzi",
        "MouziFlow",
        "Sin superficie externa: solo se puede lanzar el ejecutable",
        &[],
    ),
    (
        "davinciflow",
        "DaVinci Flow",
        "Servidor MCP JSON-RPC 2.0 stdio (motor Python local y puente con DaVinci Resolve)",
        &[
            "davinciflow_status",
            "davinciflow_templates",
            "davinciflow_capture",
            "davinciflow_fetch_media",
            "davinciflow_create_fusion_effect",
            "davinciflow_insert_native_title",
            "davinciflow_write_subtitles",
        ],
    ),
];
