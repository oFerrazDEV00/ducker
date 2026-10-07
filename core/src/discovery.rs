//! Descoberta de dispositivos:
//! 1. Multicast UDP (padrão LocalSend): anúncio + resposta via `/register` (ou UDP como fallback).
//! 2. Scan HTTP legado da sub-rede /24 (quando multicast não funciona, ex: Android sem MulticastLock).

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use socket2::{Domain, Socket, SockRef, Type};
use tokio::net::UdpSocket;
use tracing::{debug, info, warn};

use crate::client::PeerClient;
use crate::error::Result;
use crate::model::{MulticastDto, Protocol};
use crate::node::{NodeInner, Peer};

/// IPv4 locais (não-loopback) de todas as interfaces ativas.
pub fn local_ipv4s() -> Vec<Ipv4Addr> {
    let mut out: Vec<Ipv4Addr> = if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter(|i| !i.is_loopback())
        .filter_map(|i| match i.ip() {
            IpAddr::V4(v4) if !v4.is_link_local() => Some(v4),
            _ => None,
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

pub(crate) async fn start(inner: Arc<NodeInner>) -> Result<()> {
    let group = inner.config.multicast_addr;
    let mport = inner.config.multicast_port;

    let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(socket2::Protocol::UDP))?;
    sock.set_reuse_address(true)?;
    #[cfg(all(unix, not(any(target_os = "solaris", target_os = "illumos"))))]
    sock.set_reuse_port(true)?;
    sock.bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, mport)).into())?;
    sock.set_multicast_loop_v4(true)?;
    let _ = sock.set_broadcast(true);

    let mut joined = 0;
    for ip in local_ipv4s() {
        match sock.join_multicast_v4(&group, &ip) {
            Ok(()) => joined += 1,
            Err(e) => debug!("Falha ao entrar no grupo multicast pela interface {ip}: {e}"),
        }
    }
    if joined == 0 {
        sock.join_multicast_v4(&group, &Ipv4Addr::UNSPECIFIED)?;
    }
    sock.set_nonblocking(true)?;
    let udp = Arc::new(UdpSocket::from_std(sock.into())?);
    let _ = inner.udp.set(udp.clone());
    info!("Descoberta multicast/broadcast ativa em {group}:{mport} ({joined} interface(s))");

    let listen = tokio::spawn(listen_loop(inner.clone(), udp));
    let inner2 = inner.clone();
    let announcer = tokio::spawn(async move {
        // Mesmo padrão do LocalSend: rajada inicial, depois anúncios periódicos.
        for delay in [100u64, 500, 1500, 3000] {
            tokio::time::sleep(Duration::from_millis(delay)).await;
            announce(&inner2, true).await;
        }
        loop {
            tokio::time::sleep(Duration::from_secs(15)).await;
            announce(&inner2, true).await;
        }
    });
    inner.tasks.lock().unwrap().extend([listen, announcer]);
    Ok(())
}

/// Envia nosso `MulticastDto` em todas as interfaces via multicast e broadcast local.
pub(crate) async fn announce(inner: &Arc<NodeInner>, is_announce: bool) {
    let Some(udp) = inner.udp.get() else { return };
    let dto = MulticastDto::new(inner.device_info(), is_announce);
    let Ok(bytes) = serde_json::to_vec(&dto) else { return };
    let mcast_target = SocketAddr::from((inner.config.multicast_addr, inner.config.multicast_port));
    let bcast_target = SocketAddr::from((Ipv4Addr::BROADCAST, inner.config.multicast_port));
    let ifaces = local_ipv4s();
    if ifaces.is_empty() {
        let _ = udp.send_to(&bytes, mcast_target).await;
        let _ = udp.send_to(&bytes, bcast_target).await;
        return;
    }
    for ip in ifaces {
        let _ = SockRef::from(udp.as_ref()).set_multicast_if_v4(&ip);
        if let Err(e) = udp.send_to(&bytes, mcast_target).await {
            debug!("Falha ao anunciar multicast pela interface {ip}: {e}");
        }
        // Broadcast como garantia caso o roteador Wi-Fi filtre multicast IGMP entre clientes
        if let Err(e) = udp.send_to(&bytes, bcast_target).await {
            debug!("Falha ao enviar broadcast pela interface {ip}: {e}");
        }
    }
}

async fn listen_loop(inner: Arc<NodeInner>, udp: Arc<UdpSocket>) {
    let mut buf = vec![0u8; 16 * 1024];
    loop {
        let (n, src) = match udp.recv_from(&mut buf).await {
            Ok(v) => v,
            Err(e) => {
                // No Windows, UDP pode retornar WSAECONNRESET; apenas ignore.
                debug!("Erro ao receber multicast: {e}");
                tokio::time::sleep(Duration::from_millis(50)).await;
                continue;
            }
        };
        let Ok(dto) = serde_json::from_slice::<MulticastDto>(&buf[..n]) else { continue };
        let is_announce = dto.is_announce();
        let Some(peer) = inner.add_peer_from_info(dto.info, src.ip()) else { continue };
        if is_announce {
            let inner = inner.clone();
            tokio::spawn(async move { respond_to_announce(inner, peer).await });
        }
    }
}

/// Responde a um anúncio: primeiro via HTTP `/register`, se falhar via UDP (unicast direto + multicast).
async fn respond_to_announce(inner: Arc<NodeInner>, peer: Peer) {
    let me = inner.device_info();
    let id = inner.identity();
    let cert_key = (id.cert_pem.as_str(), id.key_pem.as_str());
    let result = match PeerClient::new(peer.protocol, peer.ip, peer.port, Some(cert_key), None, Some(Duration::from_secs(3))) {
        Ok(client) => client.register(&me).await,
        Err(e) => Err(e),
    };
    match result {
        Ok(info) => {
            inner.add_peer_from_info(info, peer.ip);
        }
        Err(e) => {
            debug!("register em {} falhou ({e}); respondendo via UDP unicast e broadcast", peer.ip);
            if let Some(udp) = inner.udp.get() {
                let dto = MulticastDto::new(inner.device_info(), false);
                if let Ok(bytes) = serde_json::to_vec(&dto) {
                    let unicast_target = SocketAddr::new(peer.ip, peer.port);
                    let _ = udp.send_to(&bytes, unicast_target).await;
                }
            }
            announce(&inner, false).await;
        }
    }
}

/// Scan HTTP legado: tenta `/register` em todos os hosts /24 de cada interface.
/// Retorna quantos dispositivos responderam.
pub(crate) async fn scan_subnet(inner: Arc<NodeInner>) -> usize {
    let me = inner.device_info();
    let port = inner.config.port;
    let protocol = inner.config.protocol;
    let own: Vec<Ipv4Addr> = local_ipv4s();

    let mut targets = Vec::new();
    for ip in &own {
        let [a, b, c, _] = ip.octets();
        for d in 1..=254u8 {
            let t = Ipv4Addr::new(a, b, c, d);
            if !own.contains(&t) {
                targets.push(t);
            }
        }
    }
    if targets.is_empty() {
        warn!("Nenhuma interface de rede IPv4 encontrada para o scan");
        return 0;
    }

    let found = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let id = inner.identity();
    let cert_pair = Arc::new((id.cert_pem, id.key_pem));
    futures_util::stream::iter(targets)
        .for_each_concurrent(64, |ip| {
            let inner = inner.clone();
            let me = me.clone();
            let found = found.clone();
            let cert_pair = cert_pair.clone();
            async move {
                let cert_key = (cert_pair.0.as_str(), cert_pair.1.as_str());
                for proto in [protocol, other(protocol)] {
                    let Ok(client) = PeerClient::new(proto, IpAddr::V4(ip), port, Some(cert_key), None, Some(Duration::from_millis(1500))) else {
                        return;
                    };
                    if let Ok(mut info) = client.register(&me).await {
                        info.port.get_or_insert(port);
                        info.protocol.get_or_insert(proto);
                        if inner.add_peer_from_info(info, IpAddr::V4(ip)).is_some() {
                            found.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        }
                        return;
                    }
                }
            }
        })
        .await;
    found.load(std::sync::atomic::Ordering::SeqCst)
}

fn other(p: Protocol) -> Protocol {
    match p {
        Protocol::Http => Protocol::Https,
        Protocol::Https => Protocol::Http,
    }
}
