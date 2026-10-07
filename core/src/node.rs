//! `Node`: fachada pública do Ducker. Cada dispositivo roda exatamente um Node,
//! que é ao mesmo tempo servidor (recebe) e cliente (envia) do protocolo LocalSend v2.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum_server::tls_rustls::RustlsConfig;
use serde::Serialize;
use tokio::net::UdpSocket;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{error, warn};

use crate::client::PeerClient;
use crate::discovery;
use crate::error::{DuckerError, Result};
use crate::identity::Identity;
use crate::model::*;
use crate::session::SessionStore;
use crate::tls;

/// Configuração de um nó.
#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub identity: Identity,
    pub save_dir: PathBuf,
    /// Porta TCP do servidor (0 = efêmera, útil em testes). Padrão 53317.
    pub port: u16,
    pub protocol: Protocol,
    pub device_type: DeviceType,
    pub device_model: Option<String>,
    pub multicast_addr: Ipv4Addr,
    pub multicast_port: u16,
    pub enable_discovery: bool,
    /// Aceita pedidos automaticamente (sem esperar `Node::respond`).
    pub auto_accept: bool,
    /// PIN exigido de quem envia (None = sem PIN).
    pub pin: Option<String>,
    /// Quanto tempo esperar a decisão do usuário antes de recusar.
    pub respond_timeout: Duration,
}

impl NodeConfig {
    pub fn new(identity: Identity, save_dir: PathBuf) -> Self {
        Self {
            identity,
            save_dir,
            port: DEFAULT_PORT,
            protocol: Protocol::Https,
            device_type: if cfg!(any(target_os = "android", target_os = "ios")) {
                DeviceType::Mobile
            } else {
                DeviceType::Desktop
            },
            device_model: Some(default_device_model().to_string()),
            multicast_addr: DEFAULT_MULTICAST_ADDR,
            multicast_port: DEFAULT_PORT,
            enable_discovery: true,
            auto_accept: false,
            pin: None,
            respond_timeout: Duration::from_secs(60),
        }
    }
}

pub fn default_device_model() -> &'static str {
    match std::env::consts::OS {
        "windows" => "Windows",
        "macos" => "macOS",
        "linux" => "Linux",
        "android" => "Android",
        "ios" => "iOS",
        other => other,
    }
}

/// Pasta padrão de recebimento no desktop: `~/Downloads/Ducker`.
pub fn default_save_dir() -> PathBuf {
    dirs::download_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Ducker")
}

/// Dispositivo remoto conhecido.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Peer {
    pub info: DeviceInfo,
    pub ip: IpAddr,
    pub port: u16,
    pub protocol: Protocol,
    /// Epoch em milissegundos.
    pub last_seen_ms: u64,
}

impl Peer {
    /// Chave única: fingerprint (se houver) ou ip:porta.
    pub fn key(&self) -> String {
        if self.info.fingerprint.is_empty() {
            format!("{}:{}", self.ip, self.port)
        } else {
            self.info.fingerprint.to_uppercase()
        }
    }

    pub fn is_ducker(&self) -> bool {
        self.info.quac_id.is_some()
    }

    fn client(&self, timeout: Option<Duration>) -> Result<PeerClient> {
        // Pinning estrito só entre nós Ducker (mesmo algoritmo de fingerprint garantido).
        let pin = (self.protocol == Protocol::Https && self.is_ducker()).then(|| self.info.fingerprint.clone());
        PeerClient::new(self.protocol, self.ip, self.port, pin, timeout)
    }
}

/// Eventos emitidos pelo nó (para CLI/UI).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum NodeEvent {
    PeerDiscovered { peer: Peer },
    /// Pedido de envio recebido. Se `auto_accepted == false`, chame `Node::respond`.
    IncomingRequest { session_id: String, sender: DeviceInfo, files: Vec<FileDto>, auto_accepted: bool },
    TextReceived { sender: DeviceInfo, text: String },
    ReceiveProgress { session_id: String, file_id: String, received: u64, total: u64 },
    FileReceived { session_id: String, file_id: String, file_name: String, path: String },
    SessionFinished { session_id: String },
    SessionCancelled { session_id: String },
}

/// Progresso de envio.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendProgress {
    pub file_id: String,
    pub file_name: String,
    pub file_sent: u64,
    pub file_total: u64,
    pub total_sent: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Default)]
pub struct SendOptions {
    pub pin: Option<String>,
    /// Extensão Ducker: o receptor recusa se o seu ID Quac for diferente.
    pub expected_quac: Option<u32>,
}

pub(crate) struct NodeInner {
    pub config: NodeConfig,
    identity: RwLock<Identity>,
    peers: RwLock<HashMap<String, Peer>>,
    pub sessions: SessionStore,
    events: broadcast::Sender<NodeEvent>,
    pub udp: OnceLock<Arc<UdpSocket>>,
    pub tasks: Mutex<Vec<JoinHandle<()>>>,
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

impl NodeInner {
    pub fn identity(&self) -> Identity {
        self.identity.read().unwrap().clone()
    }

    pub fn device_info(&self) -> DeviceInfo {
        let id = self.identity.read().unwrap();
        DeviceInfo {
            alias: id.alias.clone(),
            version: PROTOCOL_VERSION.to_string(),
            device_model: self.config.device_model.clone(),
            device_type: Some(self.config.device_type),
            fingerprint: id.fingerprint.clone(),
            port: Some(self.config.port),
            protocol: Some(self.config.protocol),
            download: false,
            quac_id: Some(id.quac_id),
        }
    }

    pub fn emit(&self, ev: NodeEvent) {
        let _ = self.events.send(ev);
    }

    /// Registra/atualiza um peer. Retorna `None` se for o próprio dispositivo.
    pub fn add_peer_from_info(&self, info: DeviceInfo, ip: IpAddr) -> Option<Peer> {
        let own_fp = self.identity.read().unwrap().fingerprint.clone();
        if !info.fingerprint.is_empty() && info.fingerprint.eq_ignore_ascii_case(&own_fp) {
            return None;
        }
        let peer = Peer {
            port: info.port.unwrap_or(DEFAULT_PORT),
            protocol: info.protocol.unwrap_or_default(),
            ip,
            info,
            last_seen_ms: now_ms(),
        };
        let key = peer.key();
        let is_new = {
            let mut peers = self.peers.write().unwrap();
            let changed = peers.get(&key).map(|p| p.info != peer.info || p.ip != peer.ip).unwrap_or(true);
            peers.insert(key, peer.clone());
            changed
        };
        if is_new {
            self.emit(NodeEvent::PeerDiscovered { peer: peer.clone() });
        }
        Some(peer)
    }
}

/// Handle clonável para um nó em execução.
#[derive(Clone)]
pub struct Node {
    inner: Arc<NodeInner>,
    server: axum_server::Handle,
}

impl Node {
    /// Sobe servidor HTTP(S) e descoberta. Retorna o nó e um receptor de eventos.
    pub async fn start(mut config: NodeConfig) -> Result<(Node, broadcast::Receiver<NodeEvent>)> {
        tls::install_crypto_provider();
        config.identity.ensure_certificate()?;

        let listener = std::net::TcpListener::bind((Ipv4Addr::UNSPECIFIED, config.port)).map_err(|e| {
            DuckerError::ConnectionFailed(format!(
                "não foi possível abrir a porta {} (outro Ducker/LocalSend já está rodando?): {e}",
                config.port
            ))
        })?;
        listener.set_nonblocking(true)?;
        config.port = listener.local_addr()?.port();

        let (tx, rx) = broadcast::channel(512);
        let inner = Arc::new(NodeInner {
            identity: RwLock::new(config.identity.clone()),
            config,
            peers: RwLock::new(HashMap::new()),
            sessions: SessionStore::default(),
            events: tx,
            udp: OnceLock::new(),
            tasks: Mutex::new(Vec::new()),
        });

        let handle = axum_server::Handle::new();
        let app = crate::server::router(inner.clone()).into_make_service_with_connect_info::<SocketAddr>();
        match inner.config.protocol {
            Protocol::Https => {
                let id = inner.identity();
                let rustls_cfg = RustlsConfig::from_config(tls::server_config(&id.cert_pem, &id.key_pem)?);
                let server = axum_server::from_tcp_rustls(listener, rustls_cfg).handle(handle.clone());
                tokio::spawn(async move {
                    if let Err(e) = server.serve(app).await {
                        error!("Servidor HTTPS encerrado com erro: {e}");
                    }
                });
            }
            Protocol::Http => {
                let server = axum_server::from_tcp(listener).handle(handle.clone());
                tokio::spawn(async move {
                    if let Err(e) = server.serve(app).await {
                        error!("Servidor HTTP encerrado com erro: {e}");
                    }
                });
            }
        }

        if inner.config.enable_discovery {
            if let Err(e) = discovery::start(inner.clone()).await {
                warn!("Descoberta multicast indisponível ({e}); use scan_subnet() ou conexão por IP.");
            }
        }

        Ok((Node { inner, server: handle }, rx))
    }

    pub fn subscribe(&self) -> broadcast::Receiver<NodeEvent> {
        self.inner.events.subscribe()
    }

    pub fn info(&self) -> DeviceInfo {
        self.inner.device_info()
    }

    pub fn identity(&self) -> Identity {
        self.inner.identity()
    }

    pub fn config(&self) -> &NodeConfig {
        &self.inner.config
    }

    pub fn port(&self) -> u16 {
        self.inner.config.port
    }

    pub fn save_dir(&self) -> &Path {
        &self.inner.config.save_dir
    }

    /// Altera o apelido em tempo de execução e reanuncia. (Persistência é do chamador.)
    pub async fn set_alias(&self, alias: &str) {
        self.inner.identity.write().unwrap().alias = alias.to_string();
        discovery::announce(&self.inner, true).await;
    }

    /// Peers conhecidos, ordenados por apelido.
    pub fn peers(&self) -> Vec<Peer> {
        let mut v: Vec<Peer> = self.inner.peers.read().unwrap().values().cloned().collect();
        v.sort_by(|a, b| a.info.alias.to_lowercase().cmp(&b.info.alias.to_lowercase()));
        v
    }

    /// Envia um anúncio multicast agora.
    pub async fn announce(&self) {
        discovery::announce(&self.inner, true).await;
    }

    /// Limpa a lista de peers e reanuncia.
    pub async fn refresh(&self) {
        self.inner.peers.write().unwrap().clear();
        self.announce().await;
    }

    /// Scan HTTP legado da sub-rede /24. Retorna quantos responderam.
    pub async fn scan_subnet(&self) -> usize {
        discovery::scan_subnet(self.inner.clone()).await
    }

    /// Conecta diretamente a um IP (tenta HTTPS e depois HTTP).
    pub async fn connect(&self, ip: IpAddr, port: u16) -> Result<Peer> {
        let me = self.inner.device_info();
        let mut last_err = DuckerError::DeviceNotFound;
        for proto in [Protocol::Https, Protocol::Http] {
            let client = PeerClient::new(proto, ip, port, None, Some(Duration::from_secs(5)))?;
            match client.register(&me).await {
                Ok(mut info) => {
                    info.port.get_or_insert(port);
                    info.protocol.get_or_insert(proto);
                    return self.inner.add_peer_from_info(info, ip).ok_or(DuckerError::Custom(
                        "Esse endereço é o próprio dispositivo.".into(),
                    ));
                }
                Err(e) => last_err = e,
            }
        }
        Err(last_err)
    }

    /// Encontra um peer por: ID Quac, apelido (exato ou parcial), prefixo do fingerprint, ou IP[:porta].
    pub fn find_peer(&self, query: &str) -> Option<Peer> {
        let q = query.trim();
        let ql = q.to_lowercase();
        let peers = self.peers();
        if let Ok(quac) = q.parse::<u32>() {
            if let Some(p) = peers.iter().find(|p| p.info.quac_id == Some(quac)) {
                return Some(p.clone());
            }
        }
        if let Some(p) = peers.iter().find(|p| p.info.alias.to_lowercase() == ql) {
            return Some(p.clone());
        }
        if q.len() >= 6 {
            if let Some(p) = peers.iter().find(|p| p.info.fingerprint.to_lowercase().starts_with(&ql)) {
                return Some(p.clone());
            }
        }
        if let Some(p) = peers.iter().find(|p| p.ip.to_string() == q || format!("{}:{}", p.ip, p.port) == q) {
            return Some(p.clone());
        }
        peers.into_iter().find(|p| p.info.alias.to_lowercase().contains(&ql))
    }

    /// Envia arquivos (ou pastas, recursivamente) para um peer.
    pub async fn send_files<F>(&self, peer: &Peer, paths: Vec<PathBuf>, opts: SendOptions, on_progress: F) -> Result<()>
    where
        F: Fn(SendProgress) + Send + Sync + 'static,
    {
        let mut entries = Vec::new();
        for p in &paths {
            collect_files(p, &mut entries)?;
        }
        if entries.is_empty() {
            return Err(DuckerError::Custom("Nenhum arquivo para enviar.".into()));
        }

        let mut files = HashMap::new();
        let mut by_id = HashMap::new();
        let mut total = 0u64;
        for (abs, rel, size) in entries {
            let id = uuid::Uuid::new_v4().to_string();
            let dto = FileDto {
                id: id.clone(),
                file_name: rel,
                size,
                file_type: mime_guess::from_path(&abs).first_or_octet_stream().essence_str().to_string(),
                sha256: None,
                preview: None,
                metadata: None,
            };
            total += size;
            by_id.insert(id.clone(), (abs, dto.clone()));
            files.insert(id, dto);
        }

        // prepare-upload pode demorar (o usuário do outro lado precisa aceitar).
        let client = peer.client(None)?;
        let req = PrepareUploadRequest { info: self.inner.device_info(), files };
        let Some(resp) = client.prepare_upload(&req, opts.pin.as_deref(), opts.expected_quac).await? else {
            return Ok(());
        };

        let cb = Arc::new(on_progress);
        let mut done = 0u64;
        for (file_id, token) in &resp.files {
            let Some((path, dto)) = by_id.get(file_id) else { continue };
            let (cb2, fid, name, size, base) = (cb.clone(), file_id.clone(), dto.file_name.clone(), dto.size, done);
            let progress = Arc::new(move |sent: u64| {
                cb2(SendProgress {
                    file_id: fid.clone(),
                    file_name: name.clone(),
                    file_sent: sent,
                    file_total: size,
                    total_sent: base + sent,
                    total,
                })
            });
            if let Err(e) = client.upload_file(&resp.session_id, file_id, token, path, size, progress).await {
                let _ = client.cancel(&resp.session_id).await;
                return Err(e);
            }
            done += size;
        }
        Ok(())
    }

    /// Envia uma mensagem de texto (exibida direto no destino).
    pub async fn send_text(&self, peer: &Peer, text: &str, opts: SendOptions) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let dto = FileDto {
            id: id.clone(),
            file_name: format!("{id}.txt"),
            size: text.len() as u64,
            file_type: "text/plain".into(),
            sha256: None,
            preview: Some(text.to_string()),
            metadata: None,
        };
        let client = peer.client(None)?;
        let req = PrepareUploadRequest { info: self.inner.device_info(), files: HashMap::from([(id.clone(), dto)]) };
        if let Some(resp) = client.prepare_upload(&req, opts.pin.as_deref(), opts.expected_quac).await? {
            // Receptor preferiu receber como arquivo.
            if let Some(token) = resp.files.get(&id) {
                client.upload_bytes(&resp.session_id, &id, token, text.as_bytes().to_vec()).await?;
            }
        }
        Ok(())
    }

    /// Responde a um `IncomingRequest`. Retorna `false` se o pedido já expirou.
    pub fn respond(&self, session_id: &str, accept: bool) -> bool {
        match self.inner.sessions.pending.lock().unwrap().remove(session_id) {
            Some(tx) => tx.send(accept).is_ok(),
            None => false,
        }
    }

    /// Cancela a sessão de recebimento ativa.
    pub fn cancel_receive(&self, session_id: &str) {
        let mut guard = self.inner.sessions.active.lock().unwrap();
        if guard.as_ref().map(|s| s.id == session_id).unwrap_or(false) {
            if let Some(s) = guard.take() {
                s.cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            drop(guard);
            self.inner.emit(NodeEvent::SessionCancelled { session_id: session_id.to_string() });
        }
    }

    /// Encerra servidor e tarefas de descoberta.
    pub fn shutdown(&self) {
        self.server.shutdown();
        for t in self.inner.tasks.lock().unwrap().drain(..) {
            t.abort();
        }
    }
}

/// Coleta (caminho absoluto, nome relativo com '/', tamanho). Pastas viram "pasta/sub/arquivo".
fn collect_files(path: &Path, out: &mut Vec<(PathBuf, String, u64)>) -> Result<()> {
    let meta = std::fs::metadata(path)
        .map_err(|_| DuckerError::Custom(format!("Arquivo não encontrado: {}", path.display())))?;
    if meta.is_file() {
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "arquivo".into());
        out.push((path.to_path_buf(), name, meta.len()));
        return Ok(());
    }
    let root_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "pasta".into());
    let mut stack = vec![(path.to_path_buf(), root_name)];
    while let Some((dir, prefix)) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let name = format!("{prefix}/{}", entry.file_name().to_string_lossy());
            let ft = entry.file_type()?;
            if ft.is_dir() {
                stack.push((entry.path(), name));
            } else if ft.is_file() {
                out.push((entry.path(), name, entry.metadata()?.len()));
            }
        }
    }
    Ok(())
}
