use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

use crate::error::DuckerError;
use crate::identity::DeviceIdentity;
use crate::protocol::{DiscoveryAnnouncement, DEFAULT_DISCOVERY_PORT, DEFAULT_TRANSFER_PORT};

#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub quac_id: u32,
    pub device_name: String,
    pub address: String,
    pub port: u16,
    pub protocol_version: u32,
    pub last_seen: Instant,
}

pub struct DiscoveryManager {
    identity: DeviceIdentity,
    transfer_port: u16,
    discovery_port: u16,
    devices: Arc<RwLock<HashMap<u32, DiscoveredDevice>>>,
}

impl DiscoveryManager {
    pub fn new(identity: DeviceIdentity) -> Self {
        Self {
            identity,
            transfer_port: DEFAULT_TRANSFER_PORT,
            discovery_port: DEFAULT_DISCOVERY_PORT,
            devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_ports(identity: DeviceIdentity, transfer_port: u16, discovery_port: u16) -> Self {
        Self {
            identity,
            transfer_port,
            discovery_port,
            devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Obtém o IP da interface de rede local ativa
    pub fn get_local_ip() -> Option<IpAddr> {
        // Conexão fictícia para descobrir a rota de saída da LAN
        let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
        socket.connect("8.8.8.8:80").ok()?;
        socket.local_addr().ok().map(|addr| addr.ip())
    }

    /// Inicia o serviço de anúncio e escuta de descoberta
    pub async fn start(&self) -> Result<(), DuckerError> {
        let own_quac_id = self.identity.quac_id;
        let own_name = self.identity.device_name.clone();
        let transfer_port = self.transfer_port;
        let discovery_port = self.discovery_port;

        // 1. Iniciar tarefa de escuta de anúncios UDP
        let devices_store = Arc::clone(&self.devices);
        tokio::spawn(async move {
            let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), discovery_port);
            let socket = match tokio::net::UdpSocket::bind(bind_addr).await {
                Ok(s) => s,
                Err(e) => {
                    warn!("Não foi possível escutar na porta de descoberta {}: {}", discovery_port, e);
                    return;
                }
            };

            let mut buf = [0u8; 2048];
            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((len, peer_addr)) => {
                        if let Ok(announcement) = serde_json::from_slice::<DiscoveryAnnouncement>(&buf[..len]) {
                            if announcement.is_valid() && announcement.quac_id != own_quac_id {
                                // Se o endereço anunciado for 0.0.0.0 ou vazio, usa o peer_addr
                                let resolved_address = if announcement.address.is_empty() || announcement.address == "0.0.0.0" {
                                    peer_addr.ip().to_string()
                                } else {
                                    announcement.address.clone()
                                };

                                let device = DiscoveredDevice {
                                    quac_id: announcement.quac_id,
                                    device_name: announcement.device_name,
                                    address: resolved_address,
                                    port: announcement.port,
                                    protocol_version: announcement.protocol_version,
                                    last_seen: Instant::now(),
                                };

                                let mut lock = devices_store.write().await;
                                lock.insert(device.quac_id, device);
                            }
                        }
                    }
                    Err(e) => {
                        debug!("Erro ao receber pacote UDP: {}", e);
                    }
                }
            }
        });

        // 2. Iniciar tarefa de envio periódico de anúncio (Beacon)
        tokio::spawn(async move {
            let socket = match tokio::net::UdpSocket::bind("0.0.0.0:0").await {
                Ok(s) => {
                    if let Err(e) = s.set_broadcast(true) {
                        warn!("Falha ao habilitar broadcast UDP: {}", e);
                    }
                    s
                }
                Err(e) => {
                    error!("Falha ao criar socket de broadcast: {}", e);
                    return;
                }
            };

            let broadcast_target = SocketAddr::new(
                IpAddr::V4(Ipv4Addr::BROADCAST),
                discovery_port,
            );

            loop {
                let local_ip = Self::get_local_ip()
                    .map(|ip| ip.to_string())
                    .unwrap_or_else(|| "127.0.0.1".to_string());

                let announcement = DiscoveryAnnouncement::new(
                    own_quac_id,
                    &own_name,
                    &local_ip,
                    transfer_port,
                );

                if let Ok(bytes) = serde_json::to_vec(&announcement) {
                    let _ = socket.send_to(&bytes, broadcast_target).await;
                }

                tokio::time::sleep(Duration::from_millis(1500)).await;
            }
        });

        Ok(())
    }

    /// Registra manualmente um dispositivo (ex: cliente Mobile conectado via bridge)
    pub async fn register_device(&self, device: DiscoveredDevice) {
        let mut lock = self.devices.write().await;
        lock.insert(device.quac_id, device);
    }

    /// Retorna lista de dispositivos ativos na rede (vistos nos últimos 6 segundos)
    pub async fn list_devices(&self) -> Vec<DiscoveredDevice> {
        let mut lock = self.devices.write().await;
        let now = Instant::now();
        let timeout = Duration::from_secs(6);

        // Limpar dispositivos expirados
        lock.retain(|_, dev| now.duration_since(dev.last_seen) <= timeout);

        lock.values().cloned().collect()
    }
}
