//! Escucha los mismos beacons que Prisma, sin reclamar el puerto en exclusiva.
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, UdpSocket},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Device {
    pub device_id: String,
    pub device_name: String,
    pub device_type: String,
    pub port: u16,
    pub os: String,
    pub target_app: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub address: String,
}

static DEVICES: OnceLock<Mutex<Vec<(Device, Instant)>>> = OnceLock::new();

pub fn local_address(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => ip.is_private() || ip.is_loopback() || ip.is_link_local(),
        IpAddr::V6(ip) => ip.is_loopback() || (ip.segments()[0] & 0xfe00 == 0xfc00),
    }
}

pub fn start() {
    DEVICES.get_or_init(|| {
        std::thread::spawn(|| {
            let socket = socket2::Socket::new(
                socket2::Domain::IPV4,
                socket2::Type::DGRAM,
                Some(socket2::Protocol::UDP),
            );
            let Ok(socket) = socket.and_then(|socket| {
                socket.set_reuse_address(true)?;
                socket.bind(
                    &"0.0.0.0:49289"
                        .parse::<std::net::SocketAddr>()
                        .unwrap()
                        .into(),
                )?;
                Ok(UdpSocket::from(socket))
            }) else {
                return;
            };
            let mut buffer = [0; 8192];
            while let Ok((size, sender)) = socket.recv_from(&mut buffer) {
                if !local_address(sender.ip()) {
                    continue;
                }
                let Ok(mut device) = serde_json::from_slice::<Device>(&buffer[..size]) else {
                    continue;
                };
                if device.port == 0
                    || device.device_id.len() > 200
                    || device.device_name.len() > 200
                {
                    continue;
                }
                device.address = sender.ip().to_string();
                if let Some(devices) = DEVICES.get() {
                    if let Ok(mut devices) = devices.lock() {
                        devices.retain(|(_, seen)| seen.elapsed() < Duration::from_secs(60));
                        devices.retain(|(known, _)| {
                            !(known.device_id == device.device_id
                                && known.target_app == device.target_app)
                        });
                        if devices.len() < 256 {
                            devices.push((device, Instant::now()));
                        }
                    }
                }
            }
        });
        Mutex::new(Vec::new())
    });
}

pub fn list() -> Vec<Device> {
    start();
    DEVICES
        .get()
        .and_then(|registry| registry.lock().ok())
        .map(|mut devices| {
            devices.retain(|(_, seen)| seen.elapsed() < Duration::from_secs(60));
            devices.iter().map(|(device, _)| device.clone()).collect()
        })
        .unwrap_or_default()
}

/// Dispositivos anunciados por una app concreta.
///
/// El beacon acredita que el dispositivo existe y por dónde se le alcanza.
/// No acredita autorización: la credencial se comprueba aparte.
pub fn list_for(target_app: &str) -> Vec<Device> {
    list()
        .into_iter()
        .filter(|device| device.target_app == target_app)
        .collect()
}

/// Busca un dispositivo por su identidad estable.
///
/// Permite resolver de nuevo la dirección de un dispositivo ya emparejado
/// cuando cambia su IP, en lugar de depender de una dirección congelada.
pub fn find(device_id: &str) -> Option<Device> {
    list()
        .into_iter()
        .find(|device| device.device_id == device_id)
}
