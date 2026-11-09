//! Adapta el servidor y los beacons existentes de Super Galería. Sin ADB en runtime.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    net::IpAddr,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

fn config_path() -> PathBuf {
    #[cfg(target_os = "android")]
    if let Some(path) = CONFIG_DIR.get() {
        return path.join("gallery.json");
    }
    std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("biglexj/aurora-synapse/gallery.json")
}

#[cfg(target_os = "android")]
static CONFIG_DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
#[cfg(target_os = "android")]
pub fn set_config_dir(path: PathBuf) {
    let _ = CONFIG_DIR.set(path);
}

pub fn validate_url(raw: &str) -> Result<reqwest::Url, String> {
    let url = reqwest::Url::parse(raw).map_err(|_| "Dirección de Galería inválida")?;
    let ip = url
        .host_str()
        .unwrap_or("")
        .trim_matches(['[', ']'])
        .parse::<IpAddr>()
        .map_err(|_| "Usa la IP local del teléfono, sin dominios externos")?;
    if url.scheme() != "http"
        || !super::devices::local_address(ip)
        || url.port_or_known_default().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err("Galería requiere http://IP-local:puerto, sin credenciales ni rutas".into());
    }
    Ok(url)
}

fn client(timeout: u64) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(timeout))
        .build()
        .map_err(|e| e.to_string())
}

pub fn configured_url() -> Option<String> {
    std::env::var("AURORA_SYNAPSE_GALLERY_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| {
            let value: Value = serde_json::from_slice(&std::fs::read(config_path()).ok()?).ok()?;
            value["url"].as_str().map(str::to_owned)
        })
}

fn endpoint() -> Result<reqwest::Url, String> {
    validate_url(
        &configured_url()
            .ok_or("Conecta Super Galería mediante el código temporal del teléfono")?,
    )
}

fn token() -> Result<String, String> {
    std::env::var("AURORA_SYNAPSE_GALLERY_KEY")
        .ok()
        .filter(|v| !v.is_empty())
        .map(Ok)
        .unwrap_or_else(super::gallery_credentials::load)
}

async fn response_json(response: reqwest::Response) -> Result<Value, String> {
    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "Super Galería respondió HTTP {}{}",
            status.as_u16(),
            if status.as_u16() == 401 {
                ": vuelve a conectar mediante código"
            } else {
                ""
            }
        ));
    }
    let body: Value = response
        .json()
        .await
        .map_err(|_| "Respuesta JSON de Galería inválida")?;
    if body["success"] != true {
        return Err("Super Galería rechazó la operación".into());
    }
    Ok(body)
}

pub async fn pair(address: &str, code: &str) -> Result<Value, String> {
    if code.len() != 6 || !code.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("El código debe tener seis dígitos".into());
    }
    let url = validate_url(address)?;
    let status = response_json(
        client(5)?
            .get(url.join("api/v1/synapse/status").unwrap())
            .send()
            .await
            .map_err(|_| "No responde el teléfono")?,
    )
    .await?;
    if status["target_app"] != "supergallery" {
        return Err("La dirección no corresponde a Super Galería".into());
    }
    let paired = response_json(
        client(5)?
            .post(url.join("api/v1/synapse/agents/pair").unwrap())
            .json(&json!({"code": code}))
            .send()
            .await
            .map_err(|_| "No se pudo completar la conexión")?,
    )
    .await?;
    let credential = paired["token"]
        .as_str()
        .filter(|v| v.len() >= 32)
        .ok_or("Credencial de Galería inválida")?;
    super::gallery_credentials::save(credential)?;
    let path = config_path();
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(
        path,
        json!({"url": url.as_str(), "target_app": "supergallery"}).to_string(),
    )
    .map_err(|e| e.to_string())?;
    Ok(json!({"paired": true, "url": url.as_str(), "device_name": status["device_name"]}))
}

pub async fn status() -> Result<Value, String> {
    let Some(configured) = configured_url() else {
        return Ok(
            json!({"app": "Super Galería", "configured": false, "reachable": false, "paired": false,
            "devices": super::devices::list().into_iter().filter(|d| d.target_app == "supergallery").collect::<Vec<_>>() }),
        );
    };
    let url = validate_url(&configured)?;
    let response = client(5)?
        .get(url.join("api/v1/synapse/status").unwrap())
        .send()
        .await
        .map_err(|_| "Super Galería no responde")?;
    let status = response_json(response).await?;
    if status["target_app"] != "supergallery" {
        return Err("El dispositivo no es Super Galería".into());
    }
    let paired = match token() {
        Ok(key) => match client(5)?
            .get(url.join("api/v1/synapse/agents/status").unwrap())
            .bearer_auth(key)
            .send()
            .await
        {
            Ok(response) => response_json(response).await.is_ok(),
            Err(_) => false,
        },
        Err(_) => false,
    };
    Ok(
        json!({"app": "Super Galería", "configured": true, "reachable": true, "paired": paired, "url": url.as_str(), "status": status}),
    )
}

pub async fn list(arguments: &Value) -> Result<Value, String> {
    let mut url = endpoint()?.join("api/v1/synapse/agents/media").unwrap();
    for key in ["query", "media_type", "offset", "limit"] {
        if let Some(value) = arguments.get(key) {
            let value = value
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| value.to_string());
            url.query_pairs_mut().append_pair(key, &value);
        }
    }
    response_json(
        client(15)?
            .get(url)
            .bearer_auth(token()?)
            .send()
            .await
            .map_err(|_| "No se pudo consultar la galería")?,
    )
    .await
}

fn new_destination(raw: &str) -> Result<PathBuf, String> {
    let path = super::dispatch::sanitize_path(raw)?;
    if !path.is_absolute()
        || !path.parent().is_some_and(Path::is_dir)
        || std::fs::symlink_metadata(&path).is_ok()
    {
        return Err("output_path debe ser una ruta absoluta nueva en una carpeta existente".into());
    }
    Ok(path)
}

struct NewFile(PathBuf, bool);
impl Drop for NewFile {
    fn drop(&mut self) {
        if !self.1 {
            let _ = std::fs::remove_file(&self.0);
        }
    }
}

pub async fn download(id: &str, output: &str) -> Result<Value, String> {
    if id.parse::<u64>().ok().filter(|id| *id > 0).is_none() {
        return Err("media_id debe ser un ID numérico del catálogo".into());
    }
    let destination = new_destination(output)?;
    let url = endpoint()?
        .join(&format!("api/v1/synapse/agents/media/{id}"))
        .unwrap();
    let mut response = client(120)?
        .get(url)
        .bearer_auth(token()?)
        .send()
        .await
        .map_err(|_| "No se pudo descargar el archivo del teléfono")?;
    if !response.status().is_success() {
        return Err(format!(
            "Galería rechazó la descarga: HTTP {}",
            response.status().as_u16()
        ));
    }
    let length = response
        .content_length()
        .ok_or("El teléfono no informó el tamaño del archivo")?;
    let expected_hash = response
        .headers()
        .get("x-content-sha256")
        .and_then(|v| v.to_str().ok())
        .ok_or("El teléfono no informó SHA-256")?
        .to_owned();
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&destination)
        .await
        .map_err(|e| e.to_string())?;
    let mut cleanup = NewFile(destination.clone(), false);
    let mut bytes = 0u64;
    let mut hash = Sha256::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "Transferencia interrumpida")?
    {
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        hash.update(&chunk);
        bytes += chunk.len() as u64;
    }
    file.flush().await.map_err(|e| e.to_string())?;
    file.sync_all().await.map_err(|e| e.to_string())?;
    let hash = format!("{:x}", hash.finalize());
    if bytes != length || hash != expected_hash {
        drop(file);
        return Err("La integridad del archivo no coincide".into());
    }
    cleanup.1 = true;
    Ok(
        json!({"delivery": "completed", "completed": true, "saved_to": destination, "bytes": bytes, "sha256": hash}),
    )
}

pub async fn send(input: &str) -> Result<Value, String> {
    let url = endpoint()?.join("api/v1/synapse/agents/upload").unwrap();
    let credential = token()?;
    let path = super::dispatch::sanitize_path(input)?;
    if !path.is_absolute() || !path.is_file() {
        return Err("file_path debe ser un archivo existente con ruta absoluta".into());
    }
    let mut file = tokio::fs::File::open(&path)
        .await
        .map_err(|e| e.to_string())?;
    let size = file.metadata().await.map_err(|e| e.to_string())?.len();
    let mut hash = Sha256::new();
    let mut buffer = vec![0; 65536];
    loop {
        let count = file.read(&mut buffer).await.map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    let hash = format!("{:x}", hash.finalize());
    file.rewind().await.map_err(|e| e.to_string())?;
    let name = path.file_name().unwrap().to_string_lossy();
    let response = client(120)?
        .post(url)
        .bearer_auth(credential)
        .header("Content-Length", size)
        .header("X-File-Name", urlencoding::encode(&name).as_ref())
        .body(reqwest::Body::wrap_stream(
            tokio_util::io::ReaderStream::new(file),
        ))
        .send()
        .await
        .map_err(|_| "No se pudo enviar el archivo al teléfono")?;
    let received = response_json(response).await?;
    if received["size_bytes"].as_u64() != Some(size) || received["sha256"].as_str() != Some(&hash) {
        return Err(
            "El teléfono no confirmó la integridad del archivo; no reintentes automáticamente"
                .into(),
        );
    }
    Ok(
        json!({"delivery": "completed", "completed": true, "bytes": size, "sha256": hash, "phone": received}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn no_envia_credenciales_a_destinos_publicos_o_urls_arbitrarias() {
        for value in [
            "https://example.com",
            "http://8.8.8.8:49288",
            "http://user:pass@127.0.0.1:49288",
            "http://127.0.0.1:49288/path",
        ] {
            assert!(validate_url(value).is_err());
        }
        assert!(validate_url("http://192.168.1.10:49288").is_ok());
    }
}
