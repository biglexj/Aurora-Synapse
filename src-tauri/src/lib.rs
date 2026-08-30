use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppTarget {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub icon_path: String,
    pub accent_color: String,
    pub uri_scheme: String,
    pub web_url: Option<String>,
    pub port: u16,
    pub supported_domains: Vec<String>,
    pub is_web_app: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceNode {
    pub id: String,
    pub name: String,
    pub device_type: String, // "desktop", "mobile", "tablet"
    pub ip: String,
    pub is_local: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub recommended_app_id: String,
    pub reason: String,
    pub detected_type: String,
}

#[tauri::command]
fn get_registered_apps() -> Vec<AppTarget> {
    vec![
        AppTarget {
            id: "gallerydl".into(),
            name: "Gallery-DL GUI".into(),
            category: "Galerías & Arte".into(),
            description: "Descargas masivas desde DeviantArt, Pixiv, ArtStation, Danbooru e imágenes.".into(),
            icon_path: "/assets/icons/gallery-dl-gui/icon.webp".into(),
            accent_color: "#38bdf8".into(),
            uri_scheme: "gallerydl://download?url=".into(),
            web_url: Some("https://biglexj.com/apps/gallerydl".into()),
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
        },
        AppTarget {
            id: "luna".into(),
            name: "Luna Fetch".into(),
            category: "Video & Audio".into(),
            description: "Descarga de medios desde YouTube, TikTok, Instagram, Twitter/X y SoundCloud.".into(),
            icon_path: "/assets/icons/luna-fetch/icon.webp".into(),
            accent_color: "#818cf8".into(),
            uri_scheme: "luna://download?url=".into(),
            web_url: Some("https://biglexj.com/apps/luna-fetch".into()),
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
        },
        AppTarget {
            id: "prisma".into(),
            name: "Prisma".into(),
            category: "Reproductor & Cast".into(),
            description: "Reproductor multimedia de alta fidelidad, sincronización de letras y Handoff.".into(),
            icon_path: "/assets/icons/prisma/icon.webp".into(),
            accent_color: "#60a5fa".into(),
            uri_scheme: "prisma://open?path=".into(),
            web_url: Some("https://biglexj.com/apps/prisma".into()),
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
        },
        AppTarget {
            id: "elytesia".into(),
            name: "Ely-Tesia".into(),
            category: "Práctica MIDI".into(),
            description: "Visualizador interactivo de partituras y sintetizador para archivos MIDI.".into(),
            icon_path: "/assets/icons/ely-tesia/icon.webp".into(),
            accent_color: "#f472b6".into(),
            uri_scheme: "elytesia://play?file=".into(),
            web_url: Some("https://biglexj.com/apps/ely-tesia".into()),
            port: 49291,
            supported_domains: vec![
                ".mid".into(),
                ".midi".into(),
                ".kar".into(),
                ".elytheme.json".into(),
            ],
            is_web_app: false,
        },
        AppTarget {
            id: "wintts".into(),
            name: "WinTTS".into(),
            category: "Texto a Voz".into(),
            description: "Sintetizador neuronal offline para locución de textos y artículos a voz WAV.".into(),
            icon_path: "/assets/icons/wintts/icon.webp".into(),
            accent_color: "#34d399".into(),
            uri_scheme: "wintts://speak?text=".into(),
            web_url: Some("https://biglexj.com/apps/wintts".into()),
            port: 49285,
            supported_domains: vec![
                ".txt".into(),
                ".md".into(),
                ".epub".into(),
            ],
            is_web_app: false,
        },
        AppTarget {
            id: "lyraflow".into(),
            name: "LyraFlow".into(),
            category: "Transcripción IA".into(),
            description: "Dictado por voz en tiempo real, transcripción de audios y subtitulación IA.".into(),
            icon_path: "/assets/icons/lyraflow/icon.webp".into(),
            accent_color: "#fbbf24".into(),
            uri_scheme: "lyraflow://listen".into(),
            web_url: Some("https://biglexj.com/apps/lyraflow".into()),
            port: 49292,
            supported_domains: vec![
                ".m4a".into(),
            ],
            is_web_app: false,
        },
        AppTarget {
            id: "davinciflow".into(),
            name: "DaVinci Flow".into(),
            category: "Edición de Video".into(),
            description: "Automatización de post-producción, marcadores de timeline y exportación.".into(),
            icon_path: "/assets/icons/davinci-flow/icon.webp".into(),
            accent_color: "#a78bfa".into(),
            uri_scheme: "davinciflow://timeline".into(),
            web_url: Some("https://biglexj.com/apps/davinciflow".into()),
            port: 49294,
            supported_domains: vec![
                ".drp".into(),
            ],
            is_web_app: false,
        },
        AppTarget {
            id: "mouzi".into(),
            name: "MouziFlow".into(),
            category: "Organizador de Medios".into(),
            description: "Clasificación automática de descargas, renombrado inteligente y orden de biblioteca.".into(),
            icon_path: "/assets/icons/mouziflow/icon.webp".into(),
            accent_color: "#a855f7".into(),
            uri_scheme: "mouzi://organize".into(),
            web_url: Some("https://biglexj.com/apps/mouziflow".into()),
            port: 49293,
            supported_domains: vec![],
            is_web_app: false,
        },
        AppTarget {
            id: "lienzo".into(),
            name: "Super Galería".into(),
            category: "Visor Móvil".into(),
            description: "Visor táctil de imágenes y videos sincronizado con la red local de alta fidelidad.".into(),
            icon_path: "/assets/icons/lienzo-gallery/icon.webp".into(),
            accent_color: "#2dd4bf".into(),
            uri_scheme: "supergallery://open".into(),
            web_url: Some("https://biglexj.com/apps/lienzo".into()),
            port: 49294,
            supported_domains: vec![],
            is_web_app: false,
        },
        AppTarget {
            id: "elychat".into(),
            name: "Ely Chat".into(),
            category: "Asistente IA".into(),
            description: "Inteligencia artificial conversacional y portal de soporte del ecosistema.".into(),
            icon_path: "/assets/icons/aurora-blog/icon.webp".into(),
            accent_color: "#10b981".into(),
            uri_scheme: "https://biglexj.com/chat".into(),
            web_url: Some("https://biglexj.com/chat".into()),
            port: 443,
            supported_domains: vec![],
            is_web_app: true,
        },
        AppTarget {
            id: "codex".into(),
            name: "codex-go".into(),
            category: "Desarrollo & CLI".into(),
            description: "Generador de código, herramientas de automatización y agentes de consola.".into(),
            icon_path: "/assets/icons/codex-go/icon.webp".into(),
            accent_color: "#3b82f6".into(),
            uri_scheme: "codex://run".into(),
            web_url: Some("https://biglexj.com/apps/codex".into()),
            port: 7090,
            supported_domains: vec![],
            is_web_app: false,
        },
        AppTarget {
            id: "pixistore".into(),
            name: "Pixi Store".into(),
            category: "Catálogo & Temas".into(),
            description: "Tienda comunitaria oficial para temas interactivos, presets y extensiones.".into(),
            icon_path: "/assets/icons/pixi-store/icon.webp".into(),
            accent_color: "#f59e0b".into(),
            uri_scheme: "https://biglexj.com/store".into(),
            web_url: Some("https://biglexj.com/store".into()),
            port: 443,
            supported_domains: vec![],
            is_web_app: true,
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
        },
        DeviceNode {
            id: "android_phone".into(),
            name: "Teléfono Móvil (Android)".into(),
            device_type: "mobile".into(),
            ip: "192.168.1.105".into(),
            is_local: false,
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_registered_apps,
            classify_intent,
            get_paired_devices,
            dispatch_content
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
