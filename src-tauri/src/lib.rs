use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppTarget {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub icon: String,
    pub accent_color: String,
    pub uri_scheme: String,
    pub port: u16,
    pub supported_domains: Vec<String>,
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
    pub detected_type: String, // "gallery", "stream_video", "media_file", "midi", "text", "generic_url"
}

#[tauri::command]
fn get_registered_apps() -> Vec<AppTarget> {
    vec![
        AppTarget {
            id: "gallerydl".into(),
            name: "Gallery-DL GUI".into(),
            category: "Galerías & Arte".into(),
            description: "Descargas por lotes de DeviantArt, Pixiv, ArtStation, Danbooru e imágenes.".into(),
            icon: "🖼️".into(),
            accent_color: "#06B6D4".into(),
            uri_scheme: "gallerydl://download?url=".into(),
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
        },
        AppTarget {
            id: "luna".into(),
            name: "Luna Fetch".into(),
            category: "Video & Audio".into(),
            description: "Descarga de medios desde YouTube, TikTok, Instagram, Twitter/X y SoundCloud.".into(),
            icon: "🌙".into(),
            accent_color: "#8B5CF6".into(),
            uri_scheme: "luna://download?url=".into(),
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
        },
        AppTarget {
            id: "prisma".into(),
            name: "Prisma".into(),
            category: "Reproductor & Cast".into(),
            description: "Reproductor multimedia avanzado, continuidad (handoff) y gestión de catálogo.".into(),
            icon: "✨".into(),
            accent_color: "#3B82F6".into(),
            uri_scheme: "prisma://open?path=".into(),
            port: 49290,
            supported_domains: vec![
                ".mp4".into(),
                ".mkv".into(),
                ".mp3".into(),
                ".flac".into(),
                ".webm".into(),
                ".avi".into(),
                ".mov".into(),
            ],
        },
        AppTarget {
            id: "elytesia".into(),
            name: "Ely-Tesia".into(),
            category: "Práctica MIDI".into(),
            description: "Visualizador y sintetizador interactivo para archivos MIDI y temas comunitarios.".into(),
            icon: "🎹".into(),
            accent_color: "#EC4899".into(),
            uri_scheme: "elytesia://play?file=".into(),
            port: 49291,
            supported_domains: vec![
                ".mid".into(),
                ".midi".into(),
                ".kar".into(),
                ".elytheme.json".into(),
            ],
        },
        AppTarget {
            id: "wintts".into(),
            name: "WinTTS".into(),
            category: "Texto a Voz".into(),
            description: "Sintetizador neuronal y locución por voz de textos, narraciones y artículos.".into(),
            icon: "🗣️".into(),
            accent_color: "#10B981".into(),
            uri_scheme: "wintts://speak?text=".into(),
            port: 49285,
            supported_domains: vec![
                ".txt".into(),
                ".md".into(),
                ".epub".into(),
            ],
        },
        AppTarget {
            id: "lyraflow".into(),
            name: "LyraFlow".into(),
            category: "Transcripción IA".into(),
            description: "Asistente de voz, dictado en vivo y subtitulación con inteligencia artificial.".into(),
            icon: "🎙️".into(),
            accent_color: "#F59E0B".into(),
            uri_scheme: "lyraflow://listen".into(),
            port: 49292,
            supported_domains: vec![],
        },
        AppTarget {
            id: "lienzo".into(),
            name: "Lienzo Gallery".into(),
            category: "Visor Móvil".into(),
            description: "Visor y organizador fotográfico táctil sincronizado con la red local.".into(),
            icon: "🎨".into(),
            accent_color: "#14B8A6".into(),
            uri_scheme: "supergallery://open".into(),
            port: 49294,
            supported_domains: vec![],
        },
        AppTarget {
            id: "mouzi".into(),
            name: "MouziFlow".into(),
            category: "Organizador de Medios".into(),
            description: "Clasificación automática de descargas, renombrado inteligente y orden de biblioteca.".into(),
            icon: "📁".into(),
            accent_color: "#6366F1".into(),
            uri_scheme: "mouzi://organize".into(),
            port: 49293,
            supported_domains: vec![],
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
    {
        return ClassificationResult {
            recommended_app_id: "gallerydl".into(),
            reason: "Dominio de galería de ilustraciones detectado".into(),
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
            reason: "Plataforma de streaming de video / audio detectada".into(),
            detected_type: "stream_video".into(),
        };
    }

    // 3. MIDI -> Ely-Tesia
    if trimmed.ends_with(".mid") || trimmed.ends_with(".midi") || trimmed.ends_with(".elytheme.json") {
        return ClassificationResult {
            recommended_app_id: "elytesia".into(),
            reason: "Archivo musical MIDI o tema interactivo detectado".into(),
            detected_type: "midi".into(),
        };
    }

    // 4. Video/Audio Files -> Prisma
    if trimmed.ends_with(".mp4")
        || trimmed.ends_with(".mkv")
        || trimmed.ends_with(".mp3")
        || trimmed.ends_with(".flac")
        || trimmed.ends_with(".webm")
        || trimmed.ends_with(".avi")
    {
        return ClassificationResult {
            recommended_app_id: "prisma".into(),
            reason: "Archivo multimedia de video / audio local".into(),
            detected_type: "media_file".into(),
        };
    }

    // 5. Default fallback
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        ClassificationResult {
            recommended_app_id: "gallerydl".into(),
            reason: "Enlace web genérico (Selecciona el receptor deseado)".into(),
            detected_type: "generic_url".into(),
        }
    } else {
        ClassificationResult {
            recommended_app_id: "wintts".into(),
            reason: "Texto plano para locución o procesamiento".into(),
            detected_type: "text".into(),
        }
    }
}

#[tauri::command]
fn get_paired_devices() -> Vec<DeviceNode> {
    vec![
        DeviceNode {
            id: "local_pc".into(),
            name: "Esta Computadora (PC Local)".into(),
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

    let uri = format!("{}{}", target_app.uri_scheme, urlencoding::encode(&content));

    // Try opening through system protocol handler
    match app.opener().open_url(&uri, None::<&str>) {
        Ok(_) => Ok(format!("Despachado exitosamente hacia {} ({})", target_app.name, uri)),
        Err(err) => {
            // Fallback: If HTTP endpoint exists, try posting directly
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
                _ => Err(format!("No se pudo despachar hacia {}: {}", target_app.name, err)),
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
