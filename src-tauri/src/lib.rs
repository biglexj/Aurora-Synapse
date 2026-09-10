use serde::{Deserialize, Serialize};
use tauri::Manager;

#[cfg(desktop)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(desktop)]
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
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
    vec![
        // ─── APLICACIONES NATIVAS (DESKTOP & MOBILE) ───
        AppTarget {
            id: "gallerydl".into(),
            name: "Gallery-DL GUI".into(),
            category: "Multimedia".into(),
            description: "Descargas masivas desde DeviantArt, Pixiv, ArtStation, Danbooru e imágenes.".into(),
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
            description: "Descarga de medios desde YouTube, TikTok, Instagram, Twitter/X y SoundCloud.".into(),
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
            description: "Reproductor multimedia de alta fidelidad, sincronización de letras y Handoff.".into(),
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
            id: "elytesia".into(),
            name: "Ely-Tesia".into(),
            category: "Multimedia".into(),
            description: "Visualizador interactivo de partituras y sintetizador para archivos MIDI.".into(),
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
            description: "Sintetizador neuronal offline para locución de textos y artículos a voz WAV.".into(),
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
            description: "Dictado por voz en tiempo real, transcripción de audios y subtitulación IA.".into(),
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
            description: "Automatización de post-producción, marcadores de timeline y exportación.".into(),
            icon_path: "/assets/icons/davinci-flow/icon.webp".into(),
            icon_emoji: Some("🎬".into()),
            accent_color: "#a78bfa".into(),
            uri_scheme: "davinciflow://timeline".into(),
            web_url: Some("https://www.biglexj.com/apps/davinciflow".into()),
            port: 49294,
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
            description: "Clasificación automática de descargas, renombrado inteligente y orden de biblioteca.".into(),
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
            id: "lienzo".into(),
            name: "Super Galería".into(),
            category: "Multimedia".into(),
            description: "Visor táctil de imágenes y videos sincronizado con la red local de alta fidelidad.".into(),
            icon_path: "/assets/icons/lienzo-gallery/icon.webp".into(),
            icon_emoji: Some("🌄".into()),
            accent_color: "#2dd4bf".into(),
            uri_scheme: "supergallery://open".into(),
            web_url: Some("https://www.biglexj.com/apps/lienzo".into()),
            port: 49294,
            supported_domains: vec![],
            is_web_app: false,
            status: "ACTUALIZADA".into(),
            platforms: vec!["android".into()],
            package_name: Some("com.biglexj.lienzo".into()),
        },
        AppTarget {
            id: "codex".into(),
            name: "codex-go".into(),
            category: "Desarrollo".into(),
            description: "Generador de código, herramientas de automatización y agentes de consola.".into(),
            icon_path: "/assets/icons/codex-go/icon.webp".into(),
            icon_emoji: Some("⚡".into()),
            accent_color: "#3b82f6".into(),
            uri_scheme: "codex://run".into(),
            web_url: Some("https://www.biglexj.com/apps/codex".into()),
            port: 7090,
            supported_domains: vec![],
            is_web_app: false,
            status: "EXPERIMENTAL".into(),
            platforms: vec!["windows".into(), "linux".into()],
            package_name: None,
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

    // 7. Default fallback
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

#[tauri::command]
fn get_paired_devices(is_mobile: Option<bool>) -> Vec<DeviceNode> {
    let on_mobile = is_mobile.unwrap_or(cfg!(target_os = "android") || cfg!(target_os = "ios"));
    vec![
        DeviceNode {
            id: "local_pc".into(),
            name: if on_mobile { "PC de Escritorio (Windows)".into() } else { "Esta Computadora (PC Windows)".into() },
            device_type: "desktop".into(),
            ip: "192.168.1.121".into(),
            is_local: !on_mobile,
            os: "windows".into(),
        },
        DeviceNode {
            id: "android_phone".into(),
            name: if on_mobile { "Este Teléfono (Android)".into() } else { "Teléfono Móvil (Android)".into() },
            device_type: "mobile".into(),
            ip: "192.168.1.105".into(),
            is_local: on_mobile,
            os: "android".into(),
        },
    ]
}

/// Envía contenido desde el teléfono hacia la aplicación o servidor central de Aurora Synapse en la PC
#[allow(dead_code)]
async fn dispatch_to_pc(
    target_app: &AppTarget,
    content: &str,
    pc_ip_opt: Option<&str>,
) -> Result<String, String> {
    let pc_ip = pc_ip_opt.filter(|s| !s.trim().is_empty()).unwrap_or("192.168.1.121");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .build()
        .map_err(|e| e.to_string())?;

    // 1. Intentar endpoint directo de la aplicación en la PC (ej. Gallery-DL :18274, Luna :49288)
    let app_endpoint = format!("http://{}:{}/synapse", pc_ip, target_app.port);
    let payload = serde_json::json!({
        "action": if content.trim().is_empty() { "focus" } else { "download" },
        "url": content,
        "source": "AuroraSynapseAndroid",
        "timestamp": 20260910
    });

    match client.post(&app_endpoint).json(&payload).send().await {
        Ok(resp) if resp.status().is_success() => {
            return Ok(format!("Enviado exitosamente a {} en la PC ({})", target_app.name, pc_ip));
        }
        _ => {
            // 2. Fallback: Servidor central de Aurora Synapse en la PC (:49295)
            let synapse_endpoint = format!("http://{}:49295/dispatch", pc_ip);
            let synapse_payload = serde_json::json!({
                "app_id": target_app.id,
                "content": content,
                "device": "android_phone"
            });
            match client.post(&synapse_endpoint).json(&synapse_payload).send().await {
                Ok(resp) if resp.status().is_success() => {
                    return Ok(format!("Despachado a través de Aurora Synapse en la PC ({})", pc_ip));
                }
                _ => {
                    return Err(format!(
                        "No se pudo conectar con {} en la PC ({}). Asegúrate de que la PC esté conectada a la misma red Wi-Fi y la aplicación o Synapse estén ejecutándose.",
                        target_app.name, pc_ip
                    ));
                }
            }
        }
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
        return Ok(format!("Abriendo {} en el navegador ({})", target_app.name, url));
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
        Ok(_) => Ok(format!("Despachado exitosamente hacia {} ({})", target_app.name, uri)),
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
                    Ok(format!("Despachado a través de endpoint HTTP LAN ({})", endpoint))
                }
                _ => Err(format!("No se pudo abrir {}. Asegúrate de tener la app instalada ({})", target_app.name, err)),
            }
        }
    }
}

#[cfg(desktop)]
fn start_lan_http_server(app: tauri::AppHandle) {
    tokio::spawn(async move {
        if let Ok(listener) = tokio::net::TcpListener::bind("0.0.0.0:49295").await {
            while let Ok((mut socket, _)) = listener.accept().await {
                let app_handle = app.clone();
                tokio::spawn(async move {
                    use tokio::io::{AsyncReadExt, AsyncWriteExt};
                    let mut buf = [0u8; 8192];
                    if let Ok(n) = socket.read(&mut buf).await {
                        if n == 0 { return; }
                        let req = String::from_utf8_lossy(&buf[..n]);

                        if req.starts_with("OPTIONS") {
                            let resp = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, Authorization, *\r\n\r\n";
                            let _ = socket.write_all(resp.as_bytes()).await;
                            return;
                        }

                        if req.starts_with("GET /status") {
                            let body = "{\"status\":\"online\",\"app\":\"Aurora Synapse\",\"version\":\"1.0.0\",\"protocol\":\"synapse-v1\"}";
                            let resp = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
                                body.len(), body
                            );
                            let _ = socket.write_all(resp.as_bytes()).await;
                            return;
                        }

                        if req.starts_with("POST /synapse") || req.starts_with("POST /dispatch") {
                            if let Some(idx) = req.find("\r\n\r\n") {
                                let body_str = &req[idx + 4..];
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(body_str) {
                                    let app_id = json["app_id"].as_str().or(json["app"].as_str()).unwrap_or("");
                                    let content = json["content"].as_str().or(json["url"].as_str()).unwrap_or("");

                                    let apps = get_registered_apps();
                                    if let Some(target) = apps.into_iter().find(|a| a.id == app_id) {
                                        let res = dispatch_locally_desktop(&app_handle, &target, content).await;
                                        let status_ok = res.is_ok();
                                        let msg = res.unwrap_or_else(|e| e);
                                        let resp_body = serde_json::json!({
                                            "status": if status_ok { "ok" } else { "error" },
                                            "message": msg
                                        }).to_string();
                                        let resp = format!(
                                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
                                            resp_body.len(), resp_body
                                        );
                                        let _ = socket.write_all(resp.as_bytes()).await;
                                        return;
                                    }
                                }
                            }
                        }

                        let body = "{\"error\":\"not_found\"}";
                        let resp = format!(
                            "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
                            body.len(), body
                        );
                        let _ = socket.write_all(resp.as_bytes()).await;
                    }
                });
            }
        }
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
        // En Android:
        // Si el destino es la PC, O la app es exclusiva de PC (no soporta Android): enrutar por LAN
        let is_target_pc = dev_id == "local_pc" || !target_app.platforms.contains(&"android".to_string());
        if is_target_pc {
            return dispatch_to_pc(&target_app, &content, pc_ip.as_deref()).await;
        }

        // Si es aplicación web en Android:
        if target_app.is_web_app {
            use tauri_plugin_opener::OpenerExt;
            let url = target_app.web_url.as_deref().unwrap_or(&target_app.uri_scheme);
            app.opener().open_url(url, None::<&str>)
                .map_err(|e| format!("Error al abrir portal web: {}", e))?;
            return Ok(format!("Abriendo {} en el navegador ({})", target_app.name, url));
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
                Ok(_) => return Ok(format!("Abriendo {} en el teléfono", target_app.name)),
                Err(err) => return Err(format!("No se pudo abrir {} en Android: {}", target_app.name, err)),
            }
        }

        // Fallback genérico a opener
        use tauri_plugin_opener::OpenerExt;
        match app.opener().open_url(&uri, None::<&str>) {
            Ok(_) => Ok(format!("Abriendo {} ({})", target_app.name, uri)),
            Err(e) => Err(format!("No se pudo abrir {}: {}", target_app.name, e)),
        }
    }

    #[cfg(not(target_os = "android"))]
    {
        // En escritorio (Windows):
        if dev_id == "android_phone" && !target_app.platforms.contains(&"windows".to_string()) {
            return Ok(format!("{} está instalada en tu teléfono Android.", target_app.name));
        }

        dispatch_locally_desktop(&app, &target_app, &content).await
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
        .user_agent("Aurora-Synapse/1.0.0")
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

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init());

    #[cfg(target_os = "android")]
    {
        builder = builder.plugin(
            tauri::plugin::Builder::<tauri::Wry, ()>::new("synapse_launcher")
                .setup(|app, api| {
                    let handle = api.register_android_plugin("com.biglexj.aurorasynapse", "LauncherPlugin")?;
                    app.manage(handle);
                    Ok(())
                })
                .build(),
        );
    }

    #[cfg(desktop)]
    {
        // En producción/estable protegemos contra múltiples instancias.
        // En modo desarrollo permitimos coexistencia con la versión instalada.
        if !is_dev_mode {
            builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.unminimize();
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }));
        }

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

                let _tray = TrayIconBuilder::new()
                    .icon(app.default_window_icon().unwrap().clone())
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
