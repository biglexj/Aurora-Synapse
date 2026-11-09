//! La credencial del teléfono reside en el almacén del sistema, nunca en JSON.
#[cfg(windows)]
const TARGET: &str = "AuroraSynapse/SuperGallery";

#[cfg(windows)]
pub fn save(token: &str) -> Result<(), String> {
    use windows_sys::Win32::Security::Credentials::*;
    let mut target: Vec<u16> = TARGET.encode_utf16().chain(Some(0)).collect();
    let mut blob = token.as_bytes().to_vec();
    let mut credential: CREDENTIALW = unsafe { std::mem::zeroed() };
    credential.Type = CRED_TYPE_GENERIC;
    credential.TargetName = target.as_mut_ptr();
    credential.CredentialBlobSize = blob.len() as u32;
    credential.CredentialBlob = blob.as_mut_ptr();
    credential.Persist = CRED_PERSIST_LOCAL_MACHINE;
    let written = unsafe { CredWriteW(&credential, 0) };
    blob.fill(0);
    if written == 0 {
        return Err("No se pudo guardar la credencial de Galería en Windows".into());
    }
    Ok(())
}

#[cfg(windows)]
pub fn load() -> Result<String, String> {
    use windows_sys::Win32::Security::Credentials::*;
    let target: Vec<u16> = TARGET.encode_utf16().chain(Some(0)).collect();
    let mut credential = std::ptr::null_mut();
    if unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut credential) } == 0 {
        return Err("Super Galería requiere conexión mediante código temporal".into());
    }
    let token = unsafe {
        let bytes = std::slice::from_raw_parts(
            (*credential).CredentialBlob,
            (*credential).CredentialBlobSize as usize,
        );
        let token = String::from_utf8(bytes.to_vec());
        CredFree(credential.cast());
        token
    };
    token.map_err(|_| "Credencial de Galería ilegible".into())
}

#[cfg(all(not(windows), not(target_os = "android")))]
pub fn save(_token: &str) -> Result<(), String> {
    Err("La conexión persistente por código está disponible en Windows; otros sistemas usan AURORA_SYNAPSE_GALLERY_KEY".into())
}

#[cfg(all(not(windows), not(target_os = "android")))]
pub fn load() -> Result<String, String> {
    Err("Falta AURORA_SYNAPSE_GALLERY_KEY".into())
}

#[cfg(target_os = "android")]
type MobileStore = std::sync::Arc<dyn Fn(Option<String>) -> Result<String, String> + Send + Sync>;
#[cfg(target_os = "android")]
static MOBILE_STORE: std::sync::OnceLock<MobileStore> = std::sync::OnceLock::new();
#[cfg(target_os = "android")]
pub fn set_mobile_store(store: MobileStore) {
    let _ = MOBILE_STORE.set(store);
}
#[cfg(target_os = "android")]
pub fn save(token: &str) -> Result<(), String> {
    MOBILE_STORE
        .get()
        .ok_or("Almacén Android no inicializado")?(Some(token.to_owned()))
    .map(|_| ())
}
#[cfg(target_os = "android")]
pub fn load() -> Result<String, String> {
    MOBILE_STORE
        .get()
        .ok_or("Almacén Android no inicializado")?(None)
}
