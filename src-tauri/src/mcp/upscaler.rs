//! Escalado con resultado persistido: HTTP o CLI local, nunca apertura de GUI.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::AsyncWriteExt;

use super::{dispatch, routes};

pub const JOB_TIMEOUT: Duration = Duration::from_secs(120);

pub fn output_path(input: &Path, requested: Option<&str>) -> Result<PathBuf, String> {
    let output = match requested {
        Some(raw) => dispatch::sanitize_path(raw)?,
        None => input.with_file_name(format!(
            "{}_synapse_{}.png",
            input.file_stem().unwrap_or_default().to_string_lossy(),
            dispatch::idempotency_key()
        )),
    };
    if !output.is_absolute() {
        return Err("output_path debe ser una ruta absoluta".into());
    }
    if output.exists() || std::fs::symlink_metadata(&output).is_ok() {
        return Err(format!(
            "la salida ya existe; no se sobrescribe: {}",
            output.display()
        ));
    }
    if !output
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
    {
        return Err("output_path debe tener extensión .png".into());
    }
    if !output.parent().is_some_and(Path::is_dir) {
        return Err("la carpeta de salida no existe".into());
    }
    Ok(output)
}

pub fn validate_options(scale: u64, engine: &str, sharpness: f64) -> Result<(), String> {
    if !(1..=4).contains(&scale) || !(0.0..=1.0).contains(&sharpness) {
        return Err("escala admitida: 1..4; nitidez admitida: 0..1".into());
    }
    match engine {
        "ncnn" if scale == 1 => Err("NCNN admite escalas 2, 3 y 4".into()),
        "dlss-nr" if scale != 1 => Err("DLSS NR solo admite escala 1".into()),
        "ncnn" | "nis" | "fsr1" | "dlss-nr" | "rtx-video" => Ok(()),
        _ => Err(format!("motor no admitido: {engine}")),
    }
}

/// Comprobación básica del contenedor y sus dimensiones; no sustituye un decodificador.
pub fn png_dimensions(bytes: &[u8]) -> Result<(u32, u32), String> {
    if bytes.len() < 33 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" || &bytes[12..16] != b"IHDR" {
        return Err("el motor no devolvió un PNG con cabecera IHDR válida".into());
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    if width == 0 || height == 0 {
        return Err("el PNG devuelto tiene dimensiones vacías".into());
    }
    Ok((width, height))
}

/// Crear exclusivamente un archivo nuevo, incluso si aparece otro durante el trabajo.
pub async fn save_new(output: &Path, bytes: &[u8]) -> Result<Value, String> {
    let (width, height) = png_dimensions(bytes)?;
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .await
        .map_err(|error| format!("no se pudo crear {}: {error}", output.display()))?;
    let result = async {
        file.write_all(bytes).await?;
        file.flush().await?;
        file.sync_all().await
    }
    .await;
    drop(file);
    if let Err(error) = result {
        let _ = tokio::fs::remove_file(output).await;
        return Err(format!("no se pudo guardar {}: {error}", output.display()));
    }
    Ok(json!({ "saved_to": output, "bytes": bytes.len(), "width": width, "height": height }))
}

/// El temporal pertenece a esta operación y nunca usa el nombre de un archivo del usuario.
struct TemporaryOutput(PathBuf);

impl Drop for TemporaryOutput {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

pub async fn via_cli(
    input: &Path,
    output: &Path,
    scale: u64,
    engine: &str,
    model: Option<&str>,
    sharpness: f64,
) -> Result<Value, String> {
    let executable = routes::upscaler_cli_exe();
    if !executable.is_file() {
        return Err(format!(
            "CLI local no instalada en {}. La GUI no ejecuta este contrato.",
            executable.display()
        ));
    }
    let temporary = TemporaryOutput(
        output.with_file_name(format!(".synapse-{}.png", dispatch::idempotency_key())),
    );
    let mut command = tokio::process::Command::new(&executable);
    command
        .kill_on_drop(true)
        .arg("--local")
        .arg("--engine")
        .arg(engine)
        .arg("--scale")
        .arg(scale.to_string())
        .arg("--sharpness")
        .arg(sharpness.to_string())
        .arg("--output")
        .arg(&temporary.0);
    if let Some(model) = model {
        command.arg("--model").arg(model);
    }
    command.arg("--").arg(input);
    #[cfg(windows)]
    command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    let result = tokio::time::timeout(JOB_TIMEOUT, command.output())
        .await
        .map_err(|_| "el escalado local superó 120 s; no se confirmó una salida".to_string())?
        .map_err(|error| format!("no se pudo ejecutar la CLI: {error}"))?;
    if !result.status.success() {
        return Err(format!(
            "la CLI falló ({}): {}",
            result.status,
            dispatch::truncate(&String::from_utf8_lossy(&result.stderr), 1200)
        ));
    }
    let bytes = tokio::fs::read(&temporary.0)
        .await
        .map_err(|error| format!("la CLI terminó sin una salida legible: {error}"))?;
    save_new(output, &bytes).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rechaza_motores_y_escalas_que_la_app_no_acepta() {
        assert!(validate_options(1, "ncnn", 0.5).is_err());
        assert!(validate_options(4, "dlss-nr", 0.5).is_err());
        assert!(validate_options(8, "fsr1", 0.5).is_err());
        assert!(validate_options(2, "fsr1", 0.5).is_ok());
    }

    #[tokio::test]
    async fn no_guarda_json_disfrazado_de_imagen() {
        let path = std::env::temp_dir().join(format!("{}.png", dispatch::idempotency_key()));
        assert!(save_new(&path, br#"{"success":true}"#).await.is_err());
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn no_sobrescribe_una_salida_que_aparece_durante_el_trabajo() {
        let path = std::env::temp_dir().join(format!("{}.png", dispatch::idempotency_key()));
        let mut header = vec![0; 33];
        header[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        header[12..16].copy_from_slice(b"IHDR");
        header[19] = 1;
        header[23] = 1;
        std::fs::write(&path, b"conservar").unwrap();
        assert!(save_new(&path, &header).await.is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"conservar");
        std::fs::remove_file(path).unwrap();
    }
}
