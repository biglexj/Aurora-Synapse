use serde::{Deserialize, Serialize};
use tauri::Manager;

pub mod mcp;
mod peer;

#[tauri::command]
fn create_pc_invitation() -> Result<serde_json::Value, String> { peer::create() }
#[tauri::command]
fn cancel_pc_invitation() { peer::cancel(); }
#[tauri::command]
fn pc_pairing_status() -> serde_json::Value { peer::status() }
#[tauri::command]
async fn scan_pc_qr(#[allow(unused)] app: tauri::AppHandle) -> Result<String, String> {
    #[cfg(target_os = "android")]
    {
        let launcher = app.try_state::<tauri::plugin::PluginHandle<tauri::Wry>>().ok_or("Escáner no disponible")?;
        let result = launcher.run_mobile_plugin_async::<serde_json::Value>("scanPcQr", serde_json::json!({})).await.map_err(|_| "No se pudo escanear el QR; comprueba Google Play Services o vuelve a intentarlo")?;
        return result["qr"].as_str().map(str::to_owned).ok_or("QR vacío".into());
    }
    #[cfg(not(target_os = "android"))]
    Err("Escanea el QR desde Synapse del teléfono".into())
}
#[tauri::command]
async fn connect_pc_from_qr(#[allow(unused)] qr: String) -> Result<serde_json::Value, String> {
    #[cfg(target_os = "android")]
    { return peer::connect(&qr).await; }
    #[cfg(not(target_os = "android"))]
    Err("La conexión por QR se inicia desde el teléfono".into())
}
#[tauri::command]
async fn paired_pc_status() -> Result<serde_json::Value, String> {
    #[cfg(target_os = "android")]
    { return peer::connected_status().await; }
    #[cfg(not(target_os = "android"))]
    Err("Consulta disponible en el teléfono".into())
}

#[cfg(desktop)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(desktop)]
use std::sync::Arc;
#[cfg(desktop)]
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
#[cfg(desktop)]
use tauri::image::Image;
#[cfg(desktop)]
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
#[cfg(desktop)]
use tauri::Emitter;
#[cfg(desktop)]
use tauri_plugin_autostart::ManagerExt;

#[cfg(desktop)]
static MINIMIZE_TO_TRAY: AtomicBool = AtomicBool::new(true);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppTarget {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub icon_path: String,
    pub icon_emoji: Option<String>,
    pub accent_color: String,
    pub uri_scheme: String,
    pub web_url: Option<String>,
    pub port: u16,
    pub supported_domains: Vec<String>,
    pub is_web_app: bool,
    pub status: String,
    pub platforms: Vec<String>,
    pub package_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceNode {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub ip: String,
    pub is_local: bool,
    pub os: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub recommended_app_id: String,
    pub reason: String,
    pub detected_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SynapseSettings {
    pub autostart: bool,
    pub minimize_to_tray: bool,
    pub lan_discovery: bool,
}

#[tauri::command]
fn get_registered_apps() -> Vec<AppTarget> {
    app_catalog()
}

/// Catálogo completo de aplicaciones del ecosistema.
///
/// La lógica vive fuera del comando Tauri para que el puente MCP pueda
/// reutilizarla sin depender de la interfaz.
pub(crate) fn app_catalog() -> Vec<AppTarget> {
    vec![
        // ─── APLICACIONES NATIVAS (DESKTOP & MOBILE) ───
        AppTarget {
            id: "gallerydl".into(),
            name: "Gallery-DL GUI".into(),
            category: "Multimedia".into(),
            description: "Todo tu arte favorito, a un clic.".into(),
            icon_path: "/assets/icons/gallery-dl-gui/icon.webp".into(),
            icon_emoji: Some("🖼️".into()),
            accent_color: "#38bdf8".into(),
            uri_scheme: "gallerydl://download?url=".into(),
            web_url: Some("https://www.biglexj.com/apps/gallerydl".into()),
            port: 18274,
            supported_domains: vec![
                "deviantart.com".into(),
                "pixiv.net".into(),
                "artstation.com".into(),
                "danbooru.donmai.us".into(),
                "gelbooru.com".into(),
                "safebooru.org".into(),
                "e-hentai.org".into(),
                "pinterest.com".into(),
                "imgur.com".into(),
            ],
            is_web_app: false,
            status: "ACTUALIZADA".into(),
            platforms: vec!["windows".into(), "linux".into()],
            package_name: None,
        },
        AppTarget {
            id: "luna".into(),
            name: "Luna Fetch".into(),
            category: "Multimedia".into(),
            description: "Descarga en un instante, en máxima calidad.".into(),
            icon_path: "/assets/icons/luna-fetch/icon.webp".into(),
            icon_emoji: Some("🌙".into()),
            accent_color: "#818cf8".into(),
            uri_scheme: "luna://download?url=".into(),
            web_url: Some("https://www.biglexj.com/apps/luna-fetch".into()),
            port: 49288,
            supported_domains: vec![
                "youtube.com".into(),
                "youtu.be".into(),
                "tiktok.com".into(),
                "instagram.com".into(),
                "twitter.com".into(),
                "x.com".into(),
                "soundcloud.com".into(),
                "bilibili.com".into(),
                "twitch.tv".into(),
            ],
            is_web_app: false,
            status: "ACTUALIZADA".into(),
            platforms: vec!["windows".into(), "linux".into(), "android".into()],
            package_name: Some("com.biglexj.lunafetch".into()),
        },
        AppTarget {
            id: "prisma".into(),
            name: "Prisma".into(),
            category: "Multimedia".into(),
            description: "Tus fotos, vídeos y audios, con la máxima calidad y control total.".into(),
            icon_path: "/assets/icons/prisma/icon.webp".into(),
            icon_emoji: Some("✨".into()),
            accent_color: "#60a5fa".into(),
            uri_scheme: "prisma://open?path=".into(),
            web_url: Some("https://www.biglexj.com/apps/prisma".into()),
            port: 49290,
            supported_domains: vec![
                ".mp4".into(),
                ".mkv".into(),
                ".mp3".into(),
                ".flac".into(),
                ".wav".into(),
                ".ogg".into(),
                ".webm".into(),
            ],
            is_web_app: false,
            status: "NUEVA".into(),
            platforms: vec!["windows".into(), "linux".into(), "macos".into()],
            package_name: None,
        },
        AppTarget {
            id: "prisma-upscaler".into(),
            name: "Prisma Upscaler".into(),
            category: "Multimedia".into(),
            description: "Cada imagen, en alta resolución.".into(),
            icon_path: "/assets/icons/prisma-upscaler/icon.webp".into(),
            icon_emoji: Some("🔍".into()),
            accent_color: "#22d3ee".into(),
            uri_scheme: "prisma-upscaler://upscale?path=".into(),
            web_url: Some("https://github.com/biglexj/Prisma-Upscaler".into()),
            port: 8085,
            supported_domains: vec![
                ".png".into(),
                ".jpg".into(),
                ".jpeg".into(),
                ".webp".into(),
            ],
            is_web_app: false,
            status: "ACTUALIZADA".into(),
            platforms: vec!["windows".into(), "linux".into()],
            package_name: None,
        },
        AppTarget {
            id: "elytesia".into(),
            name: "Ely-Tesia".into(),
            category: "Multimedia".into(),
            description: "Tus melodías, a la vista y al tacto.".into(),
            icon_path: "/assets/icons/ely-tesia/icon.webp".into(),
            icon_emoji: Some("🎹".into()),
            accent_color: "#f472b6".into(),
            uri_scheme: "elytesia://play?file=".into(),
            web_url: Some("https://www.biglexj.com/apps/ely-tesia".into()),
            port: 49291,
            supported_domains: vec![
                ".mid".into(),
                ".midi".into(),
                ".kar".into(),
                ".elytheme.json".into(),
            ],
            is_web_app: false,
            status: "ACTUALIZADA".into(),
            platforms: vec!["windows".into(), "android".into()],
            package_name: Some("com.biglexj.elytesia".into()),
        },
        AppTarget {
            id: "wintts".into(),
            name: "WinTTS".into(),
            category: "IA".into(),
            description: "Tu texto, narrado en tu pantalla.".into(),
            icon_path: "/assets/icons/wintts/icon.webp".into(),
            icon_emoji: Some("🗣️".into()),
            accent_color: "#34d399".into(),
            uri_scheme: "wintts://speak?text=".into(),
            web_url: Some("https://www.biglexj.com/apps/wintts".into()),
            port: 49285,
            supported_domains: vec![
                ".txt".into(),
                ".md".into(),
                ".epub".into(),
            ],
            is_web_app: false,
            status: "ACTUALIZADA".into(),
            platforms: vec!["windows".into()],
            package_name: None,
        },
        AppTarget {
            id: "lyraflow".into(),
            name: "LyraFlow".into(),
            category: "IA".into(),
            description: "Habla. Ya está escrito.".into(),
            icon_path: "/assets/icons/lyraflow/icon.webp".into(),
            icon_emoji: Some("🎙️".into()),
            accent_color: "#fbbf24".into(),
            uri_scheme: "lyraflow://listen".into(),
            web_url: Some("https://www.biglexj.com/apps/lyraflow".into()),
            port: 49292,
            supported_domains: vec![
                ".m4a".into(),
            ],
            is_web_app: false,
            status: "ACTUALIZADA".into(),
            platforms: vec!["windows".into()],
            package_name: None,
        },
        AppTarget {
            id: "davinciflow".into(),
            name: "DaVinci Flow".into(),
            category: "Multimedia".into(),
            description: "Tus subtítulos, solos en la línea de tiempo.".into(),
            icon_path: "/assets/icons/davinci-flow/icon.webp".into(),
            icon_emoji: Some("🎬".into()),
            accent_color: "#a78bfa".into(),
            uri_scheme: "davinciflow://timeline".into(),
            web_url: Some("https://www.biglexj.com/apps/davinciflow".into()),
            port: 49288,
            supported_domains: vec![
                ".drp".into(),
            ],
            is_web_app: false,
            status: "EXPERIMENTAL".into(),
            platforms: vec!["windows".into(), "macos".into()],
            package_name: None,
        },
        AppTarget {
            id: "mouzi".into(),
            name: "MouziFlow".into(),
            category: "Utilidades".into(),
            description: "Tus archivos, siempre en su sitio.".into(),
            icon_path: "/assets/icons/mouziflow/icon.webp".into(),
            icon_emoji: Some("🗂️".into()),
            accent_color: "#a855f7".into(),
            uri_scheme: "mouzi://organize".into(),
            web_url: Some("https://www.biglexj.com/apps/mouziflow".into()),
            port: 49293,
            supported_domains: vec![],
            is_web_app: false,
            status: "EXPERIMENTAL".into(),
            platforms: vec!["windows".into()],
            package_name: None,
        },
        AppTarget {
            id: "ely-intelligence".into(),
            name: "Ely Intelligence".into(),
            category: "IA".into(),
            description: "Háblale. Él entiende.".into(),
            icon_path: "/assets/icons/ely-intelligence/icon.webp".into(),
            icon_emoji: Some("🤖".into()),
            accent_color: "#10b981".into(),
            uri_scheme: "ely://".into(),
            web_url: Some("https://www.biglexj.com/desarrollo".into()),
            port: 7090,
            supported_domains: vec![],
            is_web_app: false,
            status: "INTERNO".into(),
            platforms: vec!["windows".into(), "linux".into(), "macos".into()],
            package_name: None,
        },
        AppTarget {
            id: "nubix".into(),
            name: "Nubix".into(),
            category: "Utilidades".into(),
            description: "Compara con Drive y sube solo lo que has revisado.".into(),
            icon_path: "/assets/icons/nubix/icon.webp".into(),
            icon_emoji: Some("☁️".into()),
            accent_color: "#f59e0b".into(),
            uri_scheme: "nubix://sync".into(),
            web_url: Some("https://www.biglexj.com/desarrollo".into()),
            port: 0,
            supported_domains: vec![],
            is_web_app: false,
            status: "PROXIMAMENTE".into(),
            platforms: vec!["windows".into()],
            package_name: None,
        },
        AppTarget {
            id: "lienzo".into(),
            name: "Super Galería".into(),
            category: "Multimedia".into(),
            description: "Visor, editor y reproductor multimedia todo en uno para Android.".into(),
            icon_path: "/assets/icons/lienzo-gallery/icon.webp".into(),
            icon_emoji: Some("🌄".into()),
            accent_color: "#2dd4bf".into(),
            uri_scheme: "supergallery://open".into(),
            web_url: Some("https://www.biglexj.com/apps/lienzo".into()),
            port: 49288,
            supported_domains: vec![],
            is_web_app: false,
            status: "ACTUALIZADA".into(),
            platforms: vec!["android".into()],
            package_name: Some("com.biglexj.lienzo".into()),
        },
        AppTarget {
            id: "pixistore".into(),
            name: "Pixi Store".into(),
            category: "Web".into(),
            description: "Tienda comunitaria oficial para temas interactivos, presets, visuales y extensiones.".into(),
            icon_path: "/assets/icons/pixi-store/icon.webp".into(),
            icon_emoji: Some("🛍️".into()),
            accent_color: "#f59e0b".into(),
            uri_scheme: "https://pixi.qzz.io/".into(),
            web_url: Some("https://pixi.qzz.io/".into()),
            port: 443,
            supported_domains: vec!["pixi.qzz.io".into()],
            is_web_app: true,
            status: "BETA".into(),
            platforms: vec!["web".into(), "windows".into(), "android".into(), "linux".into(), "macos".into()],
            package_name: None,
        },

        // ─── HERRAMIENTAS & APPS WEB (BIGLEXJ.COM) ───
        AppTarget {
            id: "aurora-web".into(),
            name: "Aurora".into(),
            category: "Web".into(),
            description: "Sitio web del ecosistema: contenido multimedia, música, comunidad y herramientas de creación.".into(),
            icon_path: "/assets/icons/aurora/icon.webp".into(),
            icon_emoji: Some("🌌".into()),
            accent_color: "#6366f1".into(),
            uri_scheme: "https://www.biglexj.com/".into(),
            web_url: Some("https://www.biglexj.com/".into()),
            port: 443,
            supported_domains: vec!["biglexj.com".into()],
            is_web_app: true,
            status: "ACTUALIZADA".into(),
            platforms: vec!["web".into(), "windows".into(), "android".into(), "linux".into(), "macos".into()],
            package_name: None,
        },
        AppTarget {
            id: "aurora-codex-provider".into(),
            name: "Aurora Codex Provider".into(),
            category: "Desarrollo".into(),
            description: "Proveedor personal de imágenes para Aurora basado en Codex CLI, sin claves de API externas.".into(),
            icon_path: "".into(),
            icon_emoji: Some("🖌️".into()),
            accent_color: "#3b82f6".into(),
            uri_scheme: "http://127.0.0.1:54799/".into(),
            web_url: Some("http://127.0.0.1:54799/".into()),
            port: 54799,
            supported_domains: vec![],
            is_web_app: true,
            status: "ACTUALIZADA".into(),
            platforms: vec!["windows".into(), "linux".into()],
            package_name: None,
        },
        AppTarget {
            id: "aurora-gemini-provider".into(),
            name: "Aurora Gemini Provider".into(),
            category: "Desarrollo".into(),
            description: "Proveedor de imágenes para Aurora impulsado por Gemini / Antigravity CLI, sin claves externas.".into(),
            icon_path: "".into(),
            icon_emoji: Some("🖌️".into()),
            accent_color: "#8b5cf6".into(),
            uri_scheme: "http://127.0.0.1:54800/".into(),
            web_url: Some("http://127.0.0.1:54800/".into()),
            port: 54800,
            supported_domains: vec![],
            is_web_app: true,
            status: "ACTUALIZADA".into(),
            platforms: vec!["windows".into(), "linux".into()],
            package_name: None,
        },
        AppTarget {
            id: "elychat".into(),
            name: "Ely Chat".into(),
            category: "IA".into(),
            description: "Asistente inteligente conversacional y portal oficial en biglexj.com/ely-chat.".into(),
            icon_path: "/assets/icons/ely-intelligence/icon.webp".into(),
            icon_emoji: Some("🤖".into()),
            accent_color: "#10b981".into(),
            uri_scheme: "https://www.biglexj.com/ely-chat".into(),
            web_url: Some("https://www.biglexj.com/ely-chat".into()),
            port: 443,
            supported_domains: vec![],
            is_web_app: true,
            status: "ACTUALIZADA".into(),
            platforms: vec!["web".into(), "windows".into(), "android".into(), "linux".into(), "macos".into()],
            package_name: None,
        },
        AppTarget {
            id: "twitter-ui".into(),
            name: "Tweet UI Clone".into(),
            category: "Desarrollo".into(),
            description: "Crea capturas realistas de publicaciones con IA, personalización de autor y descarga PNG.".into(),
            icon_path: "".into(),
            icon_emoji: Some("🐦".into()),
            accent_color: "#38bdf8".into(),
            uri_scheme: "https://www.biglexj.com/desarrollo/clone-twitter-ui".into(),
            web_url: Some("https://www.biglexj.com/desarrollo/clone-twitter-ui".into()),
            port: 443,
            supported_domains: vec![],
            is_web_app: true,
            status: "ACTUALIZADA".into(),
            platforms: vec!["web".into(), "windows".into(), "android".into(), "linux".into(), "macos".into()],
            package_name: None,
        },
        AppTarget {
            id: "colors-tailwind".into(),
            name: "Color Gen".into(),
            category: "Desarrollo".into(),
            description: "Generador interactivo de paletas tonales, sombras y degradados para TailwindCSS.".into(),
            icon_path: "".into(),
            icon_emoji: Some("🎨".into()),
            accent_color: "#ec4899".into(),
            uri_scheme: "https://www.biglexj.com/desarrollo/colors-tailwind".into(),
            web_url: Some("https://www.biglexj.com/desarrollo/colors-tailwind".into()),
            port: 443,
            supported_domains: vec![],
            is_web_app: true,
            status: "ACTUALIZADA".into(),
            platforms: vec!["web".into(), "windows".into(), "android".into(), "linux".into(), "macos".into()],
            package_name: None,
        },
        AppTarget {
            id: "aduana-calc".into(),
            name: "Calc. Aduana".into(),
            category: "Desarrollo".into(),
            description: "Calculadora rápida de aranceles, tasas e impuestos aduaneros para importaciones.".into(),
            icon_path: "".into(),
            icon_emoji: Some("🧮".into()),
            accent_color: "#818cf8".into(),
            uri_scheme: "https://www.biglexj.com/desarrollo/aduana-calculator".into(),
            web_url: Some("https://www.biglexj.com/desarrollo/aduana-calculator".into()),
            port: 443,
            supported_domains: vec![],
            is_web_app: true,
            status: "DISPONIBLE".into(),
            platforms: vec!["web".into(), "windows".into(), "android".into(), "linux".into(), "macos".into()],
            package_name: None,
        },
        AppTarget {
            id: "panel-creador".into(),
            name: "Panel de Creador".into(),
            category: "Web".into(),
            description: "Suite administrativa oficial, analíticas de creador y gestión de contenido de Aurora.".into(),
            icon_path: "".into(),
            icon_emoji: Some("🎛️".into()),
            accent_color: "#06b6d4".into(),
            uri_scheme: "https://www.biglexj.com/panel-creador".into(),
            web_url: Some("https://www.biglexj.com/panel-creador".into()),
            port: 443,
            supported_domains: vec![],
            is_web_app: true,
            status: "ACTUALIZADA".into(),
            platforms: vec!["web".into(), "windows".into(), "android".into(), "linux".into(), "macos".into()],
            package_name: None,
        },
        AppTarget {
            id: "chat-global".into(),
            name: "Chat Global".into(),
            category: "Web".into(),
            description: "Sala comunitaria de chat en tiempo real del ecosistema biglexj.com.".into(),
            icon_path: "".into(),
            icon_emoji: Some("💬".into()),
            accent_color: "#10b981".into(),
            uri_scheme: "https://www.biglexj.com/chat".into(),
            web_url: Some("https://www.biglexj.com/chat".into()),
            port: 443,
            supported_domains: vec![],
            is_web_app: true,
            status: "DISPONIBLE".into(),
            platforms: vec!["web".into(), "windows".into(), "android".into(), "linux".into(), "macos".into()],
            package_name: None,
        },
        AppTarget {
            id: "wallpapers".into(),
            name: "Wallpapers".into(),
            category: "Web".into(),
            description: "Galería oficial de fondos de pantalla en alta resolución y contenido visual.".into(),
            icon_path: "".into(),
            icon_emoji: Some("🖼️".into()),
            accent_color: "#f59e0b".into(),
            uri_scheme: "https://www.biglexj.com/wallpapers".into(),
            web_url: Some("https://www.biglexj.com/wallpapers".into()),
            port: 443,
            supported_domains: vec![],
            is_web_app: true,
            status: "ACTUALIZADA".into(),
            platforms: vec!["web".into(), "windows".into(), "android".into(), "linux".into(), "macos".into()],
            package_name: None,
        },
        AppTarget {
            id: "feedback".into(),
            name: "Buzón Feedback".into(),
            category: "Web".into(),
            description: "Envía sugerencias, ideas de nuevas funciones o reportes directos al creador.".into(),
            icon_path: "".into(),
            icon_emoji: Some("📬".into()),
            accent_color: "#34d399".into(),
            uri_scheme: "https://www.biglexj.com/feedback".into(),
            web_url: Some("https://www.biglexj.com/feedback".into()),
            port: 443,
            supported_domains: vec![],
            is_web_app: true,
            status: "DISPONIBLE".into(),
            platforms: vec!["web".into(), "windows".into(), "android".into(), "linux".into(), "macos".into()],
            package_name: None,
        },
        AppTarget {
            id: "musica".into(),
            name: "Explorar Música".into(),
            category: "Web".into(),
            description: "Catálogo musical, pistas originales y discografía del creador en Biglex J.".into(),
            icon_path: "".into(),
            icon_emoji: Some("🎵".into()),
            accent_color: "#ec4899".into(),
            uri_scheme: "https://www.biglexj.com/musica".into(),
            web_url: Some("https://www.biglexj.com/musica".into()),
            port: 443,
            supported_domains: vec![],
            is_web_app: true,
            status: "DISPONIBLE".into(),
            platforms: vec!["web".into(), "windows".into(), "android".into(), "linux".into(), "macos".into()],
            package_name: None,
        },
        AppTarget {
            id: "instrumentales".into(),
            name: "Instrumentales".into(),
            category: "Web".into(),
            description: "Pistas instrumentales de alta calidad para producción y karaoke.".into(),
            icon_path: "".into(),
            icon_emoji: Some("🎹".into()),
            accent_color: "#8b5cf6".into(),
            uri_scheme: "https://www.biglexj.com/instrumentales".into(),
            web_url: Some("https://www.biglexj.com/instrumentales".into()),
            port: 443,
            supported_domains: vec![],
            is_web_app: true,
            status: "DISPONIBLE".into(),
            platforms: vec!["web".into(), "windows".into(), "android".into(), "linux".into(), "macos".into()],
            package_name: None,
        },
        AppTarget {
            id: "karaoke".into(),
            name: "Karaokes".into(),
            category: "Web".into(),
            description: "Plataforma de letras sincronizadas y pistas karaoke interactivas.".into(),
            icon_path: "".into(),
            icon_emoji: Some("🎤".into()),
            accent_color: "#f43f5e".into(),
            uri_scheme: "https://www.biglexj.com/karaoke".into(),
            web_url: Some("https://www.biglexj.com/karaoke".into()),
            port: 443,
            supported_domains: vec![],
            is_web_app: true,
            status: "DISPONIBLE".into(),
            platforms: vec!["web".into(), "windows".into(), "android".into(), "linux".into(), "macos".into()],
            package_name: None,
        },
        AppTarget {
            id: "noticias".into(),
            name: "Noticias & Blog".into(),
            category: "Web".into(),
            description: "Artículos técnicos, bitácoras de desarrollo y anuncios del ecosistema.".into(),
            icon_path: "".into(),
            icon_emoji: Some("📰".into()),
            accent_color: "#0ea5e9".into(),
            uri_scheme: "https://www.biglexj.com/noticias".into(),
            web_url: Some("https://www.biglexj.com/noticias".into()),
            port: 443,
            supported_domains: vec![],
            is_web_app: true,
            status: "DISPONIBLE".into(),
            platforms: vec!["web".into(), "windows".into(), "android".into(), "linux".into(), "macos".into()],
            package_name: None,
        },
        AppTarget {
            id: "live".into(),
            name: "En Vivo".into(),
            category: "Web".into(),
            description: "Transmisiones oficiales en directo, sesiones interactivas y stream de VTuber.".into(),
            icon_path: "".into(),
            icon_emoji: Some("🔴".into()),
            accent_color: "#ef4444".into(),
            uri_scheme: "https://www.biglexj.com/live".into(),
            web_url: Some("https://www.biglexj.com/live".into()),
            port: 443,
            supported_domains: vec![],
            is_web_app: true,
            status: "DISPONIBLE".into(),
            platforms: vec!["web".into(), "windows".into(), "android".into(), "linux".into(), "macos".into()],
            package_name: None,
        },
    ]
}

#[tauri::command]
fn classify_intent(input: &str) -> ClassificationResult {
    classify_intent_impl(input)
}

/// Clasificador de intenciones, reutilizable por el puente MCP.
pub(crate) fn classify_intent_impl(input: &str) -> ClassificationResult {
    let lower = input.to_lowercase();
    let trimmed = lower.trim();

    // 1. DeviantArt, Pixiv, Image Boards -> Gallery-DL
    if trimmed.contains("deviantart.com")
        || trimmed.contains("pixiv.net")
        || trimmed.contains("artstation.com")
        || trimmed.contains("danbooru")
        || trimmed.contains("gelbooru")
        || trimmed.contains("safebooru")
        || trimmed.contains("e-hentai.org")
        || trimmed.contains("pinterest.com")
        || trimmed.contains("imgur.com")
    {
        return ClassificationResult {
            recommended_app_id: "gallerydl".into(),
            reason: "Galería de ilustraciones detectada (DeviantArt / Pixiv / Board)".into(),
            detected_type: "gallery".into(),
        };
    }

    // 2. Video / Music Streaming -> Luna Fetch
    if trimmed.contains("youtube.com")
        || trimmed.contains("youtu.be")
        || trimmed.contains("tiktok.com")
        || trimmed.contains("instagram.com")
        || trimmed.contains("twitter.com")
        || trimmed.contains("x.com")
        || trimmed.contains("soundcloud.com")
        || trimmed.contains("twitch.tv")
        || trimmed.contains("bilibili.com")
    {
        return ClassificationResult {
            recommended_app_id: "luna".into(),
            reason: "Video / Audio Streaming detectado (YouTube / TikTok / Redes)".into(),
            detected_type: "stream_video".into(),
        };
    }

    // 3. MIDI -> Ely-Tesia
    if trimmed.ends_with(".mid") || trimmed.ends_with(".midi") || trimmed.ends_with(".kar") || trimmed.ends_with(".elytheme.json") {
        return ClassificationResult {
            recommended_app_id: "elytesia".into(),
            reason: "Archivo de partitura MIDI o tema interactivo".into(),
            detected_type: "midi".into(),
        };
    }

    // 4. Audio files for LyraFlow (voice note)
    if trimmed.ends_with(".m4a") {
        return ClassificationResult {
            recommended_app_id: "lyraflow".into(),
            reason: "Nota de voz para transcripción y subtitulación con IA".into(),
            detected_type: "voice_note".into(),
        };
    }

    // 5. Video/Audio Files -> Prisma
    if trimmed.ends_with(".mp4")
        || trimmed.ends_with(".mkv")
        || trimmed.ends_with(".mp3")
        || trimmed.ends_with(".flac")
        || trimmed.ends_with(".wav")
        || trimmed.ends_with(".ogg")
        || trimmed.ends_with(".webm")
        || trimmed.ends_with(".avi")
    {
        return ClassificationResult {
            recommended_app_id: "prisma".into(),
            reason: "Archivo multimedia de video / música local".into(),
            detected_type: "media_file".into(),
        };
    }

    // 6. Pixi Store
    if trimmed.contains("pixi.qzz.io") {
        return ClassificationResult {
            recommended_app_id: "pixistore".into(),
            reason: "Portal oficial Pixi Store para temas interactivos y presets".into(),
            detected_type: "store".into(),
        };
    }

    // 7. Super-resolución de imágenes -> Prisma Upscaler
    if (trimmed.contains("escala")
        || trimmed.contains("escalad")
        || trimmed.contains("upscal")
        || trimmed.contains("super-resolucion")
        || trimmed.contains("superresolucion")
        || trimmed.contains("realesrgan"))
        && (trimmed.ends_with(".png")
            || trimmed.ends_with(".jpg")
            || trimmed.ends_with(".jpeg")
            || trimmed.ends_with(".webp"))
    {
        return ClassificationResult {
            recommended_app_id: "prisma-upscaler".into(),
            reason: "Super-resolución de imagen detectada (2x, 4x, modelo neuronal)".into(),
            detected_type: "image_upscale".into(),
        };
    }

    // 8. Default fallback
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        ClassificationResult {
            recommended_app_id: "gallerydl".into(),
            reason: "Enlace web genérico (Selecciona la app receptora)".into(),
            detected_type: "generic_url".into(),
        }
    } else {
        ClassificationResult {
            recommended_app_id: "wintts".into(),
            reason: "Texto plano para sintetizar a voz o dictado".into(),
            detected_type: "text".into(),
        }
    }
}

fn is_local_machine(device: &mcp::devices::Device) -> bool {
    if device.address == "127.0.0.1" || device.address == "localhost" || device.address == "::1" {
        return true;
    }
    if let Ok(comp) = std::env::var("COMPUTERNAME") {
        if !comp.trim().is_empty() && device.device_name.eq_ignore_ascii_case(comp.trim()) {
            return true;
        }
    }
    if let Ok(hostname) = std::env::var("HOSTNAME") {
        if !hostname.trim().is_empty() && device.device_name.eq_ignore_ascii_case(hostname.trim()) {
            return true;
        }
    }
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        for target in ["1.1.1.1:80", "8.8.8.8:80", "192.168.1.1:80", "10.0.0.1:80"] {
            if socket.connect(target).is_ok() {
                if let Ok(addr) = socket.local_addr() {
                    if addr.ip().to_string() == device.address {
                        return true;
                    }
                }
                break;
            }
        }
    }
    false
}

#[tauri::command]
fn get_paired_devices(is_mobile: Option<bool>) -> Vec<DeviceNode> {
    let mobile = is_mobile.unwrap_or(cfg!(target_os = "android") || cfg!(target_os = "ios"));
    let mut devices = vec![DeviceNode {
        id: if mobile { "android_phone" } else { "local_pc" }.into(),
        name: if mobile { "Este teléfono" } else { "Este equipo" }.into(),
        device_type: if mobile { "mobile" } else { "desktop" }.into(),
        ip: "127.0.0.1".into(),
        is_local: true,
        os: if mobile { "android" } else { "windows" }.into(),
    }];

    for device in mcp::devices::list() {
        if is_local_machine(&device) {
            // En escritorio y móvil: no registrar balizas de la propia máquina como dispositivos remotos
            continue;
        }

        devices.push(DeviceNode {
            id: device.device_id,
            name: format!("{} ({})", device.device_name, device.target_app),
            device_type: device.device_type,
            ip: device.address,
            is_local: false,
            os: device.os.to_lowercase(),
        });
    }

    devices
}

#[tauri::command]
async fn pair_gallery(address: String, code: String) -> Result<serde_json::Value, String> {
    mcp::gallery::pair(&address, &code).await
}

#[tauri::command]
async fn gallery_connection() -> Result<serde_json::Value, String> {
    mcp::gallery::status().await
}

#[tauri::command]
fn gallery_devices() -> Vec<mcp::devices::Device> {
    mcp::devices::list().into_iter().filter(|device| device.target_app == "supergallery").collect()
}

/// Estado real del puente MCP, de sus aplicaciones y de los agentes conectados.
///
/// El comando existe también en Android: allí el puente no arranca y la
/// respuesta es explícita (`running: false`, `transport: "none"`), nunca un
/// dato simulado.
#[tauri::command]
async fn mcp_server_status() -> mcp::status::McpServerStatus {
    mcp::status::status().await
}

/// Envía una orden desde el teléfono hacia el PC vinculado.
///
/// La única ruta autorizada es el canal emparejado. Antes se intentaban
/// endpoints LAN heredados sin credencial y con un contrato que las apps no
/// implementan: además de ejecutar sin autorización, podían repetir la orden.
#[allow(dead_code)]
async fn dispatch_to_pc(
    target_app: &AppTarget,
    content: &str,
    pc_ip_opt: Option<&str>,
) -> Result<String, String> {
    #[cfg(target_os = "android")]
    {
        return peer::dispatch(&target_app.id, content, pc_ip_opt.unwrap_or(""))
            .await?
            .ok_or_else(|| {
                "No hay un PC vinculado. En Ajustes → Conexiones, escanea el QR del PC para enviarle órdenes."
                    .to_string()
            });
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (target_app, content, pc_ip_opt);
        Err("El envío a un PC remoto se inicia desde el teléfono emparejado".into())
    }
}

/// Despacha localmente en el entorno de escritorio (Windows / Linux / macOS)
async fn dispatch_locally_desktop(
    app: &tauri::AppHandle,
    target_app: &AppTarget,
    content: &str,
) -> Result<String, String> {
    use tauri_plugin_opener::OpenerExt;

    if target_app.is_web_app {
        let url = target_app.web_url.as_deref().unwrap_or(&target_app.uri_scheme);
        app.opener().open_url(url, None::<&str>)
            .map_err(|e| format!("Error al abrir portal web: {}", e))?;
        return Ok(format!("Orden entregada al navegador para {} ({})", target_app.name, url));
    }

    let uri = if target_app.uri_scheme.ends_with("=") {
        if content.trim().is_empty() {
            if let Some(base) = target_app.uri_scheme.split('?').next() {
                base.to_string()
            } else {
                target_app.uri_scheme.clone()
            }
        } else {
            format!("{}{}", target_app.uri_scheme, urlencoding::encode(content))
        }
    } else {
        target_app.uri_scheme.clone()
    };

    match app.opener().open_url(&uri, None::<&str>) {
        // Entregar la URI no demuestra que la ventana esté visible: se informa
        // la entrega, no una ejecución comprobada.
        Ok(_) => Ok(format!("Orden entregada a {} ({})", target_app.name, uri)),
        Err(err) => {
            let client = reqwest::Client::new();
            let endpoint = format!("http://127.0.0.1:{}/synapse", target_app.port);
            let payload = serde_json::json!({
                "action": if content.trim().is_empty() { "focus" } else { "download" },
                "url": content,
                "timestamp": 20260910
            });

            match client.post(&endpoint).json(&payload).send().await {
                Ok(resp) if resp.status().is_success() => {
                    Ok(format!("Orden entregada a {} por su endpoint local ({})", target_app.name, endpoint))
                }
                _ => Err(format!("No se pudo abrir {}. Asegúrate de tener la app instalada ({})", target_app.name, err)),
            }
        }
    }
}

/// Receptor LAN del PC.
///
/// Solo enruta: la autorización vive en `peer::decide` y la ejecución local en
/// `dispatch_locally_desktop`. Una orden autorizada sigue un único camino, sin
/// reintentos ni rutas alternativas sin credencial.
#[cfg(desktop)]
fn start_lan_http_server(app: tauri::AppHandle) {
    // OJO: el setup hook de Tauri corre en el hilo principal FUERA de cualquier
    // runtime Tokio. `tokio::spawn` ahi entra en panic ("there is no reactor
    // running") y se lleva la app entera por delante: la ventana no llega a
    // abrir. `tauri::async_runtime::spawn` usa el runtime global de Tauri y
    // si funciona desde este contexto.
    tauri::async_runtime::spawn(async move {
        let Ok(listener) = tokio::net::TcpListener::bind("0.0.0.0:49295").await else { return; };
        peer::listening();
        let handle = app.clone();
        let dispatch: peer::Dispatcher = Arc::new(move |app_id: &str, content: &str| {
            let app = handle.clone();
            let app_id = app_id.to_string();
            let content = content.to_string();
            let future: peer::DispatchFuture = Box::pin(async move {
                let target = get_registered_apps()
                    .into_iter()
                    .find(|target| target.id == app_id)
                    .ok_or_else(|| format!("La aplicación «{app_id}» no figura en el catálogo de este equipo"))?;
                if matches!(target.status.as_str(), "PROXIMAMENTE" | "INTERNO") {
                    return Err(format!("{} todavía no está disponible en el ecosistema", target.name));
                }
                dispatch_locally_desktop(&app, &target, &content).await
            });
            future
        });
        peer::serve(listener, dispatch).await;
    });
}

#[tauri::command]
async fn dispatch_content(
    app: tauri::AppHandle,
    app_id: String,
    content: String,
    target_device_id: Option<String>,
    #[allow(unused)] pc_ip: Option<String>,
) -> Result<String, String> {
    let apps = get_registered_apps();
    let target_app = apps.into_iter().find(|a| a.id == app_id).ok_or("Aplicación no encontrada")?;

    if target_app.status == "PROXIMAMENTE" {
        return Err(format!("{} estará disponible próximamente en el ecosistema.", target_app.name));
    }

    let dev_id = target_device_id.unwrap_or_else(|| "local_pc".into());

    #[cfg(target_os = "android")]
    {
        // En Android el destino es explícito: el PC vinculado o este teléfono.
        // Un dispositivo descubierto por beacon no acredita un contrato de
        // despacho; antes se enviaba en silencio a la PC aunque el usuario
        // hubiera elegido otro equipo.
        if dev_id == "local_pc" {
            return dispatch_to_pc(&target_app, &content, pc_ip.as_deref()).await;
        }
        if dev_id != "android_phone" {
            return Err(format!(
                "«{dev_id}» es otro dispositivo descubierto en la red. Este Synapse no tiene un despacho verificado hacia él: elige «Este teléfono» o «Este equipo»."
            ));
        }
        if !target_app.platforms.contains(&"android".to_string()) && !target_app.is_web_app {
            return Err(format!(
                "{} no está disponible en Android. Elige «Este equipo» en el selector de dispositivo para enviarla al PC.",
                target_app.name
            ));
        }

        // Si es aplicación web en Android:
        if target_app.is_web_app {
            use tauri_plugin_opener::OpenerExt;
            let url = target_app.web_url.as_deref().unwrap_or(&target_app.uri_scheme);
            app.opener().open_url(url, None::<&str>)
                .map_err(|e| format!("Error al abrir portal web: {}", e))?;
            return Ok(format!("Orden entregada al navegador para {} ({})", target_app.name, url));
        }

        // Preparar URI scheme
        let uri = if target_app.uri_scheme.ends_with("=") {
            if content.trim().is_empty() {
                target_app.uri_scheme.split('?').next().unwrap_or(&target_app.uri_scheme).to_string()
            } else {
                format!("{}{}", target_app.uri_scheme, urlencoding::encode(&content))
            }
        } else {
            target_app.uri_scheme.clone()
        };

        // Invocar LauncherPlugin nativo de Android
        if let Some(launcher) = app.try_state::<tauri::plugin::PluginHandle<tauri::Wry>>() {
            let res = launcher.run_mobile_plugin::<serde_json::Value>(
                "launchApp",
                serde_json::json!({
                    "package_name": target_app.package_name,
                    "uri": uri,
                    "content": content
                })
            );
            match res {
                Ok(_) => return Ok(format!("Orden entregada a {} en el teléfono", target_app.name)),
                Err(err) => return Err(format!("No se pudo abrir {} en Android: {}", target_app.name, err)),
            }
        }

        // Fallback genérico a opener
        use tauri_plugin_opener::OpenerExt;
        match app.opener().open_url(&uri, None::<&str>) {
            Ok(_) => Ok(format!("Orden entregada a {} ({})", target_app.name, uri)),
            Err(e) => Err(format!("No se pudo abrir {}: {}", target_app.name, e)),
        }
    }

    #[cfg(not(target_os = "android"))]
    {
        // En escritorio (Windows):
        if dev_id == "local_pc" {
            return dispatch_locally_desktop(&app, &target_app, &content).await;
        }

        // PC → teléfono: Android no permite abrir una app de otro equipo y el
        // teléfono no expone un contrato remoto de apertura. La única orden
        // remota verificada es la transferencia a la Super Galería conectada.
        if target_app.id != "lienzo" {
            return Err(format!(
                "«{}» no admite órdenes remotas desde el PC: Android no permite abrir apps de otro dispositivo y no existe un contrato de apertura. Ábrela en el teléfono.",
                target_app.name
            ));
        }
        let device = mcp::devices::find(&dev_id).ok_or_else(|| {
            format!("El destino {dev_id} ya no está anunciado en la red local; el descubrimiento se actualiza cada minuto")
        })?;
        if device.target_app != "supergallery" {
            return Err(format!(
                "El dispositivo elegido ({}) no es Super Galería; no se envió nada",
                device.device_name
            ));
        }
        let configured = mcp::gallery::configured_url().ok_or_else(|| {
            "Super Galería no está conectada desde este PC. Conéctala con su código temporal en Ajustes → Conexiones."
                .to_string()
        })?;
        let configured = mcp::gallery::validate_url(&configured)?;
        if configured.host_str() != Some(device.address.as_str()) {
            return Err(format!(
                "El dispositivo elegido ({}) no coincide con la Super Galería conectada ({}). Elige el destino correcto o vuelve a conectarla.",
                device.device_name,
                configured.host_str().unwrap_or("desconocido")
            ));
        }
        let path = std::path::Path::new(content.trim());
        if content.trim().is_empty() || !path.is_absolute() {
            return Err(
                "Android no permite abrir una app del teléfono desde el PC. Para enviar algo a Super Galería, elige un archivo con ruta absoluta."
                    .into(),
            );
        }
        let result = mcp::gallery::send(&content).await?;
        Ok(format!(
            "Archivo recibido por Super Galería ({} bytes, SHA-256 {}). La apertura remota de la app no es una operación soportada por Android.",
            result["bytes"], result["sha256"]
        ))
    }
}

#[tauri::command]
fn get_synapse_settings(#[allow(unused)] app: tauri::AppHandle) -> SynapseSettings {
    #[cfg(desktop)]
    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);
    #[cfg(not(desktop))]
    let autostart_enabled = false;

    #[cfg(desktop)]
    let min_to_tray = MINIMIZE_TO_TRAY.load(Ordering::Relaxed);
    #[cfg(not(desktop))]
    let min_to_tray = false;

    SynapseSettings {
        autostart: autostart_enabled,
        minimize_to_tray: min_to_tray,
        lan_discovery: true,
    }
}

#[tauri::command]
fn set_autostart_setting(#[allow(unused)] app: tauri::AppHandle, enable: bool) -> Result<bool, String> {
    #[cfg(desktop)]
    {
        if enable {
            app.autolaunch().enable().map_err(|e| e.to_string())?;

            #[cfg(windows)]
            {
                // In debug mode or if installed release binary exists, ensure registry points to release binary
                // to avoid launching target\debug\aurora-synapse.exe on Windows boot (which causes 404/connection errors
                // because the Vite dev server is not running).
                if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                    let release_exe = std::path::PathBuf::from(&local_app_data)
                        .join("Aurora Synapse")
                        .join("aurora-synapse.exe");

                    if release_exe.exists() {
                        let cmd_val = format!("\"{}\" --autostart", release_exe.to_string_lossy());
                        use std::os::windows::process::CommandExt;
                        let _ = std::process::Command::new("reg")
                            .args([
                                "add",
                                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                                "/v",
                                "Aurora Synapse",
                                "/t",
                                "REG_SZ",
                                "/d",
                                &cmd_val,
                                "/f",
                            ])
                            .creation_flags(0x08000000)
                            .status();
                    }
                }
            }
        } else {
            app.autolaunch().disable().map_err(|e| e.to_string())?;

            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                let _ = std::process::Command::new("reg")
                    .args([
                        "delete",
                        "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                        "/v",
                        "Aurora Synapse",
                        "/f",
                    ])
                    .creation_flags(0x08000000)
                    .status();
            }
        }
    }
    Ok(enable)
}

#[tauri::command]
fn set_minimize_to_tray_setting(enable: bool) -> bool {
    #[cfg(desktop)]
    MINIMIZE_TO_TRAY.store(enable, Ordering::Relaxed);
    enable
}

#[tauri::command]
fn is_mobile_platform() -> bool {
    cfg!(mobile)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateCheckResult {
    pub available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_name: String,
    pub release_notes: String,
    pub published_at: String,
    pub html_url: String,
    pub exe_url: Option<String>,
    pub apk_url: Option<String>,
    pub assets: Vec<ReleaseAsset>,
}

fn is_version_greater(remote: &str, current: &str) -> bool {
    let parse_parts = |s: &str| -> Vec<u32> {
        s.split('.')
            .map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>())
            .filter_map(|p| p.parse::<u32>().ok())
            .collect()
    };

    let r_parts = parse_parts(remote);
    let c_parts = parse_parts(current);

    let max_len = r_parts.len().max(c_parts.len());
    for i in 0..max_len {
        let r = r_parts.get(i).copied().unwrap_or(0);
        let c = c_parts.get(i).copied().unwrap_or(0);
        if r > c {
            return true;
        } else if r < c {
            return false;
        }
    }
    false
}

#[tauri::command]
async fn check_for_updates() -> Result<UpdateCheckResult, String> {
    let client = reqwest::Client::builder()
        .user_agent(format!("Aurora-Synapse/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())?;

    let url = "https://api.github.com/repos/biglexj/Aurora-Synapse/releases/latest";
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Error conectando con GitHub: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API retornó status: {}", resp.status()));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Error parseando respuesta JSON: {}", e))?;

    let tag_name = json["tag_name"].as_str().unwrap_or("").to_string();
    let release_name = json["name"].as_str().unwrap_or(&tag_name).to_string();
    let body = json["body"].as_str().unwrap_or("").to_string();
    let published_at = json["published_at"].as_str().unwrap_or("").to_string();
    let html_url = json["html_url"].as_str().unwrap_or("").to_string();

    let clean_tag = tag_name.trim_start_matches('v');
    let current_version = env!("CARGO_PKG_VERSION");

    let mut assets = Vec::new();
    let mut exe_url = None;
    let mut apk_url = None;

    if let Some(asset_arr) = json["assets"].as_array() {
        for a in asset_arr {
            let name = a["name"].as_str().unwrap_or("").to_string();
            let dl_url = a["browser_download_url"].as_str().unwrap_or("").to_string();
            let size = a["size"].as_u64().unwrap_or(0);

            if name.ends_with(".exe") && !name.contains("-portable") {
                exe_url = Some(dl_url.clone());
            }
            if name.ends_with(".apk") {
                apk_url = Some(dl_url.clone());
            }

            assets.push(ReleaseAsset {
                name,
                browser_download_url: dl_url,
                size,
            });
        }
    }

    let available = is_version_greater(clean_tag, current_version);

    Ok(UpdateCheckResult {
        available,
        current_version: current_version.to_string(),
        latest_version: clean_tag.to_string(),
        release_name,
        release_notes: body,
        published_at,
        html_url,
        exe_url,
        apk_url,
        assets,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let is_dev_mode = cfg!(debug_assertions)
        || std::env::args().any(|a| a == "--dev" || a == "--multi-instance" || a == "-d")
        || std::env::var("SYNAPSE_DEV").is_ok()
        || std::env::var("SYNAPSE_MULTI_INSTANCE").is_ok();

    #[cfg(desktop)]
    if let Some(position) = std::env::args().position(|argument| argument == "--gallery-pair") {
        let address = std::env::args().nth(position + 1).unwrap_or_default();
        let mut code = String::new();
        std::io::stdin().read_line(&mut code).ok();
        match tauri::async_runtime::block_on(mcp::gallery::pair(&address, code.trim())) {
            Ok(result) => println!("{result}"),
            Err(error) => { eprintln!("{error}"); std::process::exit(1); }
        }
        return;
    }

    // Modo stdio: el binario se lanza como subproceso y habla MCP por
    // entrada/salida estándar, sin abrir la interfaz.
    #[cfg(desktop)]
    if std::env::args().any(|argument| argument == "--mcp-stdio") {
        #[cfg(windows)]
        use std::os::windows::process::CommandExt;
        let opener: mcp::dispatch::UriOpener = Arc::new(|uri: &str| {
            let mut command = std::process::Command::new("cmd");
            #[cfg(windows)]
            command
                .raw_arg("/C")
                .raw_arg(format!("start \"\" \"{uri}\""));
            #[cfg(not(windows))]
            command.arg("xdg-open").arg(uri);
            command
                .spawn()
                .map(|_| ())
                .map_err(|error| format!("Error al abrir {uri}: {error}"))
        });
        tauri::async_runtime::block_on(mcp::serve_stdio(opener));
        return;
    }

    // Modo headless: solo el puente MCP, sin interfaz. El puerto se puede
    // fijar con AURORA_SYNAPSE_MCP_ADDR (por defecto 127.0.0.1:49296).
    #[cfg(desktop)]
    if std::env::args().any(|argument| argument == "--mcp-http") {
        let addr = std::env::var("AURORA_SYNAPSE_MCP_ADDR")
            .unwrap_or_else(|_| mcp::server::DEFAULT_BIND_ADDR.to_string());
        let opener: mcp::dispatch::UriOpener = Arc::new(|uri: &str| {
            let mut command = std::process::Command::new("cmd");
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                command
                    .raw_arg("/C")
                    .raw_arg(format!("start \"\" \"{uri}\""));
            }
            #[cfg(not(windows))]
            command.arg("xdg-open").arg(uri);
            command
                .spawn()
                .map(|_| ())
                .map_err(|error| format!("Error al abrir {uri}: {error}"))
        });
        let result = tauri::async_runtime::block_on(mcp::start_fixed(opener, &addr));
        match result {
            Ok(port) => {
                println!("aurora-synapse: puente MCP activo en http://{addr}/mcp (puerto {port})");
                // El proceso se queda vivo hasta que lo termine el supervisor.
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3600));
                }
            }
            Err(error) => {
                eprintln!("aurora-synapse: no se pudo arrancar el puente MCP en {addr}: {error}");
                std::process::exit(1);
            }
        }
    }

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init());

    #[cfg(target_os = "android")]
    {
        builder = builder.plugin(
            tauri::plugin::Builder::<tauri::Wry, ()>::new("synapse_launcher")
                .setup(|app, api| {
                    let handle = api.register_android_plugin("com.biglexj.aurorasynapse", "LauncherPlugin")?;
                    let storage = handle.clone();
                    mcp::gallery_credentials::set_mobile_store(std::sync::Arc::new(move |token| {
                        let result = storage.run_mobile_plugin::<serde_json::Value>("galleryCredential", serde_json::json!({"token": token}))
                            .map_err(|_| "No se pudo acceder al almacén seguro Android".to_string())?;
                        Ok(result["token"].as_str().unwrap_or_default().to_string())
                    }));
                    mcp::gallery::set_config_dir(app.path().app_data_dir()?);
                    let peer_store = handle.clone();
                    peer::set_store(std::sync::Arc::new(move |token| {
                        let result = peer_store.run_mobile_plugin::<serde_json::Value>("peerCredential", serde_json::json!({"token":token})).map_err(|_| "No se pudo acceder a la conexión del PC".to_string())?;
                        Ok(result["token"].as_str().unwrap_or_default().to_string())
                    }));
                    app.manage(handle);
                    Ok(())
                })
                .build(),
        );
    }

    #[cfg(desktop)]
    {
        // La interfaz comparte puertos LAN/MCP fijos también en desarrollo.
        // Una segunda ventana en otro proceso no puede usar el receptor de la primera.
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.unminimize();
                    let _ = w.show();
                    let _ = w.set_focus();
                }
        }));

        builder = builder
            .plugin(tauri_plugin_window_state::Builder::default().build())
            .plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                Some(vec!["--autostart"]),
            ));
    }

    builder
        .setup(move |_app| {
            #[cfg(desktop)]
            {
                start_lan_http_server(_app.handle().clone());

                // Puente MCP del ecosistema: se expone solo en loopback porque
                // concede control sobre el resto de las aplicaciones.
                {
                    use tauri_plugin_opener::OpenerExt;
                    let handle = _app.handle().clone();
                    let opener: mcp::dispatch::UriOpener = Arc::new(move |uri: &str| {
                        handle
                            .opener()
                            .open_url(uri, None::<&str>)
                            .map_err(|error| format!("Error al abrir {uri}: {error}"))
                    });
                    tauri::async_runtime::spawn(async move {
                        if let Err(error) = mcp::start(opener).await {
                            eprintln!("[aurora-synapse mcp] no se pudo arrancar el puente: {error}");
                        }
                    });
                }

                let app = _app;
                // System Tray Menu Setup
                let show_item = MenuItemBuilder::with_id("show", "Abrir Aurora Synapse").build(app)?;
                let settings_item = MenuItemBuilder::with_id("settings", "Configuración").build(app)?;
                let separator = PredefinedMenuItem::separator(app)?;
                let quit_item = MenuItemBuilder::with_id("quit", "Salir de Aurora Synapse").build(app)?;

                let tray_menu = MenuBuilder::new(app)
                    .items(&[&show_item, &settings_item, &separator, &quit_item])
                    .build()?;

                let tray_tooltip = if is_dev_mode {
                    "Aurora Synapse (Dev) · Orquestador Universal"
                } else {
                    "Aurora Synapse · Orquestador Universal"
                };

                let tray_icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))
                    .unwrap_or_else(|_| app.default_window_icon().unwrap().clone());

                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.set_icon(tray_icon.clone());
                }

                let _tray = TrayIconBuilder::new()
                    .icon(tray_icon)
                    .tooltip(tray_tooltip)
                    .menu(&tray_menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| {
                        match event.id().as_ref() {
                            "show" => {
                                if let Some(w) = app.get_webview_window("main") {
                                    let _ = w.unminimize();
                                    let _ = w.show();
                                    let _ = w.set_focus();
                                }
                            }
                            "settings" => {
                                if let Some(w) = app.get_webview_window("main") {
                                    let _ = w.unminimize();
                                    let _ = w.show();
                                    let _ = w.set_focus();
                                    let _ = w.emit("synapse://open-settings", ());
                                }
                            }
                            "quit" => {
                                app.exit(0);
                            }
                            _ => {}
                        }
                    })
                    .on_tray_icon_event(|tray, event| {
                        match event {
                            TrayIconEvent::Click {
                                button: MouseButton::Left,
                                button_state: MouseButtonState::Up,
                                ..
                            }
                            | TrayIconEvent::DoubleClick {
                                button: MouseButton::Left,
                                ..
                            } => {
                                let app = tray.app_handle();
                                if let Some(w) = app.get_webview_window("main") {
                                    let _ = w.unminimize();
                                    let _ = w.show();
                                    let _ = w.set_focus();
                                }
                            }
                            _ => {}
                        }
                    })
                    .build(app)?;

                let is_autostart = std::env::args().any(|a| a == "--autostart");

                // Window visibility & autostart handling
                if let Some(main_window) = app.get_webview_window("main") {
                    if is_dev_mode {
                        let _ = main_window.set_title("Aurora Synapse (Dev)");
                    }

                    if !is_autostart {
                        let _ = main_window.show();
                        let _ = main_window.unminimize();
                        let _ = main_window.set_focus();
                    } else {
                        let _ = main_window.hide();
                    }

                    let win = main_window.clone();
                    main_window.on_window_event(move |event| {
                        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                            if MINIMIZE_TO_TRAY.load(Ordering::Relaxed) {
                                api.prevent_close();
                                let _ = win.hide();
                            }
                        }
                    });
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_registered_apps,
            classify_intent,
            get_paired_devices,
            pair_gallery,
            gallery_connection,
            gallery_devices,
            mcp_server_status,
            create_pc_invitation,
            cancel_pc_invitation,
            pc_pairing_status,
            scan_pc_qr,
            connect_pc_from_qr,
            paired_pc_status,
            dispatch_content,
            get_synapse_settings,
            set_autostart_setting,
            set_minimize_to_tray_setting,
            is_mobile_platform,
            check_for_updates
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
