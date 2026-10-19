use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use ducker_core::{DuckiveProfile, Identity, Node, Peer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Radar,
    Duckive,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogKind {
    Info,
    Success,
    Warning,
    Error,
    Message,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub text: String,
    pub kind: LogKind,
}

#[derive(Debug, Clone)]
pub struct TransferInfo {
    pub file_name: String,
    pub is_incoming: bool,
    pub total_bytes: u64,
    pub transferred_bytes: u64,
    pub start_time: Instant,
    pub pct: u32,
    pub speed_mbps: f64,
}

#[derive(Debug, Clone)]
pub enum ModalMode {
    None,
    IncomingRequest {
        session_id: String,
        sender_name: String,
        files_summary: String,
        total_size: u64,
    },
    InputText {
        title: String,
        prompt: String,
        value: String,
        target_peer: Peer,
    },
    InputFilePath {
        title: String,
        prompt: String,
        value: String,
        target_peer: Peer,
    },
}

pub struct TuiApp {
    pub identity: Identity,
    pub config_dir: PathBuf,
    pub node: Arc<Node>,
    pub peers: Vec<Peer>,
    pub selected_peer_idx: usize,
    pub duckive: DuckiveProfile,
    pub active_transfer: Option<TransferInfo>,
    pub logs: Vec<LogEntry>,
    pub active_tab: ActiveTab,
    pub modal: ModalMode,
    pub tick: u64,
    pub status_message: Option<(String, Instant)>,
    pub should_quit: bool,
}

impl TuiApp {
    pub fn new(identity: Identity, config_dir: PathBuf, node: Arc<Node>) -> Self {
        let alias = identity.alias.clone();
        let duckive = DuckiveProfile::load_or_create(&config_dir, &alias)
            .unwrap_or_else(|_| DuckiveProfile::new(&alias));

        let mut app = Self {
            identity,
            config_dir,
            node,
            peers: Vec::new(),
            selected_peer_idx: 0,
            duckive,
            active_transfer: None,
            logs: Vec::new(),
            active_tab: ActiveTab::Radar,
            modal: ModalMode::None,
            tick: 0,
            status_message: None,
            should_quit: false,
        };

        app.add_log(
            format!(
                "Ducker inicializado! Ouvindo na porta {} ({})",
                app.node.port(),
                app.node.config().protocol.scheme()
            ),
            LogKind::Success,
        );
        app.add_log(
            "Pressione [r] para buscar dispositivos, [d] para Duckive ou [?] para ajuda."
                .to_string(),
            LogKind::Info,
        );

        app
    }

    pub fn add_log(&mut self, text: String, kind: LogKind) {
        let now = chrono_or_simple_time();
        self.logs.insert(
            0,
            LogEntry {
                timestamp: now,
                text,
                kind,
            },
        );
        if self.logs.len() > 80 {
            self.logs.truncate(80);
        }
    }

    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    pub fn selected_peer(&self) -> Option<&Peer> {
        self.peers.get(self.selected_peer_idx)
    }

    pub fn select_next_peer(&mut self) {
        if !self.peers.is_empty() {
            self.selected_peer_idx = (self.selected_peer_idx + 1) % self.peers.len();
        }
    }

    pub fn select_prev_peer(&mut self) {
        if !self.peers.is_empty() {
            if self.selected_peer_idx == 0 {
                self.selected_peer_idx = self.peers.len() - 1;
            } else {
                self.selected_peer_idx -= 1;
            }
        }
    }

    pub fn update_peers(&mut self, new_peers: Vec<Peer>) {
        let previously_selected = self
            .peers
            .get(self.selected_peer_idx)
            .map(|p| p.key().to_string());
        self.peers = new_peers;

        if self.peers.is_empty() {
            self.selected_peer_idx = 0;
        } else if let Some(ref key) = previously_selected {
            if let Some(pos) = self.peers.iter().position(|p| p.key() == key) {
                self.selected_peer_idx = pos;
            } else if self.selected_peer_idx >= self.peers.len() {
                self.selected_peer_idx = 0;
            }
        } else if self.selected_peer_idx >= self.peers.len() {
            self.selected_peer_idx = 0;
        }
    }
}

fn chrono_or_simple_time() -> String {
    // Retorna timestamp sem depender de chrono extra
    let now = std::time::SystemTime::now();
    let since_epoch = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = since_epoch.as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ducker_core::{DeviceInfo, NodeConfig, Protocol};
    use std::net::Ipv4Addr;

    fn mock_peer(alias: &str, ip_last_octet: u8) -> Peer {
        Peer {
            key: format!("key_{}", alias),
            ip: std::net::IpAddr::V4(Ipv4Addr::new(192, 168, 1, ip_last_octet)),
            port: 53317,
            protocol: Protocol::Https,
            info: DeviceInfo {
                alias: alias.to_string(),
                version: "2.1".to_string(),
                device_model: Some("Test Device".to_string()),
                device_type: None,
                fingerprint: format!("fp_{}", alias),
                port: Some(53317),
                protocol: Some(Protocol::Https),
                download: false,
                quac_id: None,
            },
            last_seen_ms: 1000,
        }
    }

    #[tokio::test]
    async fn test_peer_selection_navigation() {
        let temp_dir = std::env::temp_dir().join(format!("ducker-test-tui-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let id = Identity::generate("Tester").unwrap();
        let mut config = NodeConfig::new(id.clone(), temp_dir.clone());
        config.port = 0;
        let (node, _) = Node::start(config).await.unwrap();
        let mut app = TuiApp::new(id, temp_dir, Arc::new(node));

        app.peers = vec![
            mock_peer("Peer1", 10),
            mock_peer("Peer2", 20),
            mock_peer("Peer3", 30),
        ];
        assert_eq!(app.selected_peer_idx, 0);
        app.select_next_peer();
        assert_eq!(app.selected_peer_idx, 1);
        app.select_next_peer();
        assert_eq!(app.selected_peer_idx, 2);
        app.select_next_peer();
        assert_eq!(app.selected_peer_idx, 0);

        app.select_prev_peer();
        assert_eq!(app.selected_peer_idx, 2);
    }

    #[tokio::test]
    async fn test_update_peers_retention() {
        let temp_dir = std::env::temp_dir().join(format!("ducker-test-ret-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let id = Identity::generate("Tester").unwrap();
        let mut config = NodeConfig::new(id.clone(), temp_dir.clone());
        config.port = 0;
        let (node, _) = Node::start(config).await.unwrap();
        let mut app = TuiApp::new(id, temp_dir, Arc::new(node));

        app.peers = vec![mock_peer("Peer1", 10), mock_peer("Peer2", 20)];
        app.selected_peer_idx = 1;

        let new_peers = vec![mock_peer("Peer2", 20), mock_peer("Peer3", 30)];
        app.update_peers(new_peers);
        assert_eq!(app.selected_peer_idx, 0);
        assert_eq!(app.selected_peer().unwrap().info.alias, "Peer2");
    }

    #[tokio::test]
    async fn test_activity_log_truncation() {
        let temp_dir = std::env::temp_dir().join(format!("ducker-test-log-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let id = Identity::generate("Tester").unwrap();
        let mut config = NodeConfig::new(id.clone(), temp_dir.clone());
        config.port = 0;
        let (node, _) = Node::start(config).await.unwrap();
        let mut app = TuiApp::new(id, temp_dir, Arc::new(node));

        for i in 0..100 {
            app.add_log(format!("Log message {}", i), LogKind::Info);
        }

        assert_eq!(app.logs.len(), 80);
        assert!(app.logs[0].text.contains("99"));
    }
}
