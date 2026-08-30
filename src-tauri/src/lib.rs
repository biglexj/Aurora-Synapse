use serde::{Deserialize, Serialize};

#[cfg(desktop)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(desktop)]
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
#[cfg(desktop)]
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
#[cfg(desktop)]
use tauri::{Manager, Emitter};
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
            status: "NUEVA".into(),
            platforms: vec!["windows".into(), "linux".into(), "android".into()],
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
            platforms: vec!["windows".into()],
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
        },
        AppTarget {
            id: "pixistore".into(),
            name: "Pixi Store".into(),
            category: "Experimentos".into(),
            description: "Tienda comunitaria oficial para temas interactivos, presets y extensiones.".into(),
            icon_path: "/assets/icons/pixi-store/icon.webp".into(),
            icon_emoji: Some("🛍️".into()),
            accent_color: "#f59e0b".into(),
            uri_scheme: "https://www.biglexj.com".into(),
            web_url: Some("https://www.biglexj.com".into()),
            port: 443,
            supported_domains: vec![],
            is_web_app: false,
            status: "PROXIMAMENTE".into(),
            platforms: vec!["windows".into(), "android".into(), "web".into()],
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

    // 6. Default fallback
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
fn get_paired_devices() -> Vec<DeviceNode> {
    vec![
        DeviceNode {
            id: "local_pc".into(),
            name: "Esta Computadora (PC Windows)".into(),
            device_type: "desktop".into(),
            ip: "127.0.0.1".into(),
            is_local: true,
            os: "windows".into(),
        },
        DeviceNode {
            id: "android_phone".into(),
            name: "Teléfono Móvil (Android)".into(),
            device_type: "mobile".into(),
            ip: "192.168.1.105".into(),
            is_local: false,
            os: "android".into(),
        },
    ]
}

#[tauri::command]
async fn dispatch_content(
    app: tauri::AppHandle,
    app_id: String,
    content: String,
    _target_device_id: String,
) -> Result<String, String> {
    use tauri_plugin_opener::OpenerExt;

    let apps = get_registered_apps();
    let target_app = apps.into_iter().find(|a| a.id == app_id).ok_or("Aplicación no encontrada")?;

    if target_app.status == "PROXIMAMENTE" {
        return Err(format!("{} estará disponible próximamente en el ecosistema.", target_app.name));
    }

    if target_app.is_web_app {
        let url = target_app.web_url.unwrap_or(target_app.uri_scheme);
        app.opener().open_url(&url, None::<&str>)
            .map_err(|e| format!("Error al abrir portal web: {}", e))?;
        return Ok(format!("Abriendo {} en el navegador ({})", target_app.name, url));
    }

    let uri = if target_app.uri_scheme.ends_with("=") {
        format!("{}{}", target_app.uri_scheme, urlencoding::encode(&content))
    } else {
        target_app.uri_scheme.clone()
    };

    match app.opener().open_url(&uri, None::<&str>) {
        Ok(_) => Ok(format!("Despachado exitosamente hacia {} ({})", target_app.name, uri)),
        Err(err) => {
            let client = reqwest::Client::new();
            let endpoint = format!("http://127.0.0.1:{}/synapse", target_app.port);
            let payload = serde_json::json!({
                "action": "download",
                "url": content,
                "timestamp": 20260830
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
        } else {
            app.autolaunch().disable().map_err(|e| e.to_string())?;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init());

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_window_state::Builder::default().build())
            .plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                Some(vec!["--autostart"]),
            ));
    }

    builder
        .setup(|_app| {
            #[cfg(desktop)]
            {
                let app = _app;
                // System Tray Menu Setup
                let show_item = MenuItemBuilder::with_id("show", "Abrir Aurora Synapse").build(app)?;
                let settings_item = MenuItemBuilder::with_id("settings", "Configuración").build(app)?;
                let separator = PredefinedMenuItem::separator(app)?;
                let quit_item = MenuItemBuilder::with_id("quit", "Salir de Aurora Synapse").build(app)?;

                let tray_menu = MenuBuilder::new(app)
                    .items(&[&show_item, &settings_item, &separator, &quit_item])
                    .build()?;

                let _tray = TrayIconBuilder::new()
                    .icon(app.default_window_icon().unwrap().clone())
                    .tooltip("Aurora Synapse · Orquestador Universal")
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

                // Prevent window close if minimize to tray is enabled
                if let Some(main_window) = app.get_webview_window("main") {
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
            is_mobile_platform
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
