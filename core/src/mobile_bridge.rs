use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path as AxumPath, State,
    },
    http::StatusCode,
    response::Response,
    routing::get,
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

use crate::discovery::{DiscoveredDevice, DiscoveryManager};
use crate::error::{DuckerError, ErrorCode};
use crate::identity::DeviceIdentity;
use crate::protocol::{
    HandshakeStatus, TransferHandshakeRequest, TransferHandshakeResponse, CURRENT_PROTOCOL_VERSION,
};

pub const DEFAULT_MOBILE_BRIDGE_PORT: u16 = 7876;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileNodeStatus {
    pub device_name: String,
    pub quac_id: u32,
    pub protocol_version: u32,
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileDeviceDto {
    pub quac_id: u32,
    pub device_name: String,
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MobileWsMessage {
    #[serde(rename = "mobile_announce")]
    Announce {
        quac_id: u32,
        device_name: String,
    },
    #[serde(rename = "transfer_request")]
    TransferRequest(TransferHandshakeRequest),
    #[serde(rename = "handshake_response")]
    HandshakeResponse(TransferHandshakeResponse),
    #[serde(rename = "file_chunk")]
    FileChunk {
        transfer_id: String,
        chunk_base64: String,
        is_last: bool,
    },
    #[serde(rename = "transfer_status")]
    TransferStatus {
        transfer_id: String,
        status: String,
        message: String,
    },
}

pub struct ActiveTransfer {
    pub id: String,
    pub file_name: String,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub destination_quac_id: u32,
}

pub struct MobileBridgeState {
    pub identity: DeviceIdentity,
    pub discovery: Arc<DiscoveryManager>,
    pub mobile_clients: Arc<RwLock<HashMap<u32, mpsc::Sender<Message>>>>,
    pub pending_transfers: Arc<RwLock<HashMap<String, ActiveTransfer>>>,
    pub transfer_responses: Arc<RwLock<HashMap<String, broadcast::Sender<TransferHandshakeResponse>>>>,
}

pub struct MobileBridge {
    state: Arc<MobileBridgeState>,
    port: u16,
}

impl MobileBridge {
    pub fn new(identity: DeviceIdentity, discovery: Arc<DiscoveryManager>) -> Self {
        Self {
            state: Arc::new(MobileBridgeState {
                identity,
                discovery,
                mobile_clients: Arc::new(RwLock::new(HashMap::new())),
                pending_transfers: Arc::new(RwLock::new(HashMap::new())),
                transfer_responses: Arc::new(RwLock::new(HashMap::new())),
            }),
            port: DEFAULT_MOBILE_BRIDGE_PORT,
        }
    }

    pub fn with_port(identity: DeviceIdentity, discovery: Arc<DiscoveryManager>, port: u16) -> Self {
        Self {
            state: Arc::new(MobileBridgeState {
                identity,
                discovery,
                mobile_clients: Arc::new(RwLock::new(HashMap::new())),
                pending_transfers: Arc::new(RwLock::new(HashMap::new())),
                transfer_responses: Arc::new(RwLock::new(HashMap::new())),
            }),
            port,
        }
    }

    pub fn state(&self) -> Arc<MobileBridgeState> {
        Arc::clone(&self.state)
    }

    pub async fn start(&self) -> Result<(), DuckerError> {
        let port = self.port;
        let state = Arc::clone(&self.state);

        let app = Router::new()
            .route("/api/status", get(get_status))
            .route("/api/devices", get(get_devices))
            .route("/api/ws", get(ws_handler))
            .route("/api/download/:transfer_id", get(download_file))
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            .with_state(state);

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
            DuckerError::ConnectionFailed(format!("Falha ao abrir porta do Mobile Bridge {}: {}", port, e))
        })?;

        info!("Ducker Mobile Bridge escutando em http://{}", addr);

        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                error!("Erro no servidor Mobile Bridge: {}", e);
            }
        });

        Ok(())
    }
}

async fn get_status(State(state): State<Arc<MobileBridgeState>>) -> Json<MobileNodeStatus> {
    let local_ip = DiscoveryManager::get_local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| "127.0.0.1".to_string());

    Json(MobileNodeStatus {
        device_name: state.identity.device_name.clone(),
        quac_id: state.identity.quac_id,
        protocol_version: CURRENT_PROTOCOL_VERSION,
        address: local_ip,
        port: DEFAULT_MOBILE_BRIDGE_PORT,
    })
}

async fn get_devices(State(state): State<Arc<MobileBridgeState>>) -> Json<Vec<MobileDeviceDto>> {
    let devices = state.discovery.list_devices().await;
    let dtos = devices
        .into_iter()
        .map(|d| MobileDeviceDto {
            quac_id: d.quac_id,
            device_name: d.device_name,
            address: d.address,
            port: d.port,
        })
        .collect();
    Json(dtos)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<MobileBridgeState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<MobileBridgeState>) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<Message>(32);

    let sender_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut registered_quac_id: Option<u32> = None;

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            if let Ok(ws_msg) = serde_json::from_str::<MobileWsMessage>(&text) {
                match ws_msg {
                    MobileWsMessage::Announce { quac_id, device_name } => {
                        info!("Mobile device anunciado: {} [{}]", device_name, quac_id);
                        registered_quac_id = Some(quac_id);

                        state
                            .mobile_clients
                            .write()
                            .await
                            .insert(quac_id, tx.clone());

                        let local_ip = DiscoveryManager::get_local_ip()
                            .map(|ip| ip.to_string())
                            .unwrap_or_else(|| "127.0.0.1".to_string());

                        // Registra no discovery manager para aparecer no ducker devices / send
                        state
                            .discovery
                            .register_device(DiscoveredDevice {
                                quac_id,
                                device_name: device_name.clone(),
                                address: local_ip,
                                port: DEFAULT_MOBILE_BRIDGE_PORT,
                                protocol_version: CURRENT_PROTOCOL_VERSION,
                                last_seen: Instant::now(),
                            })
                            .await;

                        // Envia confirmação de boas vindas
                        let status_resp = MobileWsMessage::TransferStatus {
                            transfer_id: "".to_string(),
                            status: "CONNECTED".to_string(),
                            message: format!("Conectado ao Ducker Node: {}", state.identity.device_name),
                        };
                        if let Ok(json) = serde_json::to_string(&status_resp) {
                            let _ = tx.send(Message::Text(json)).await;
                        }
                    }
                    MobileWsMessage::HandshakeResponse(resp) => {
                        info!("Resposta de handshake do Mobile: {:?}", resp);
                        let responses = state.transfer_responses.read().await;
                        for sender in responses.values() {
                            let _ = sender.send(resp.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if let Some(id) = registered_quac_id {
        state.mobile_clients.write().await.remove(&id);
    }
    sender_task.abort();
}

async fn download_file(
    AxumPath(transfer_id): AxumPath<String>,
    State(state): State<Arc<MobileBridgeState>>,
) -> Result<Response, StatusCode> {
    let transfers = state.pending_transfers.read().await;
    let transfer = transfers.get(&transfer_id).ok_or(StatusCode::NOT_FOUND)?;

    let file_bytes = tokio::fs::read(&transfer.file_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = Response::builder()
        .header("Content-Type", "application/octet-stream")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", transfer.file_name),
        )
        .body(axum::body::Body::from(file_bytes))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

impl MobileBridgeState {
    /// Inicia envio de arquivo para um dispositivo Mobile conectado
    pub async fn send_to_mobile(
        &self,
        destination_quac_id: u32,
        file_path: PathBuf,
    ) -> Result<(), DuckerError> {
        let clients = self.mobile_clients.read().await;
        let client_tx = clients.get(&destination_quac_id).ok_or_else(|| {
            DuckerError::DeviceNotFound
        })?;

        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("arquivo")
            .to_string();

        let metadata = tokio::fs::metadata(&file_path).await?;
        let file_size = metadata.len();
        let transfer_id = format!("t_{}_{}", destination_quac_id, rand::random::<u32>());

        // 1. Armazenar a transferência ativa para download pelo Mobile
        {
            let mut transfers = self.pending_transfers.write().await;
            transfers.insert(
                transfer_id.clone(),
                ActiveTransfer {
                    id: transfer_id.clone(),
                    file_name: file_name.clone(),
                    file_path: file_path.clone(),
                    file_size,
                    destination_quac_id,
                },
            );
        }

        // 2. Criar canal para aguardar resposta do handshake do Mobile
        let (resp_tx, mut resp_rx) = broadcast::channel(1);
        {
            let mut responses = self.transfer_responses.write().await;
            responses.insert(transfer_id.clone(), resp_tx);
        }

        // 3. Enviar requisição de handshake para o Mobile
        let handshake_req = TransferHandshakeRequest::new(
            &self.identity.device_name,
            self.identity.quac_id,
            destination_quac_id,
            &file_name,
            file_size,
        );

        let ws_msg = MobileWsMessage::TransferRequest(handshake_req);
        let msg_str = serde_json::to_string(&ws_msg)?;

        client_tx
            .send(Message::Text(msg_str))
            .await
            .map_err(|e| DuckerError::ConnectionFailed(e.to_string()))?;

        // 4. Aguardar resposta do Mobile (timeout de 15 segundos)
        let response = match tokio::time::timeout(std::time::Duration::from_secs(15), resp_rx.recv()).await {
            Ok(Ok(resp)) => resp,
            Ok(Err(_)) => return Err(DuckerError::TransferFailed("Canal de resposta fechado.".into())),
            Err(_) => return Err(DuckerError::TransferFailed("Destinatário demorou a responder.".into())),
        };

        // Limpar canais
        {
            let mut responses = self.transfer_responses.write().await;
            responses.remove(&transfer_id);
        }

        match response.status {
            HandshakeStatus::Accepted => {
                info!("Handshake aceito pelo Mobile para {}", file_name);
                Ok(())
            }
            HandshakeStatus::Rejected => {
                if let Some(reason) = response.reason {
                    if reason == ErrorCode::DestinationIdMismatch {
                        return Err(DuckerError::DestinationIdMismatch);
                    }
                }
                Err(DuckerError::TransferFailed(
                    response.message.unwrap_or_else(|| "Rejeitado pelo destinatário.".into()),
                ))
            }
        }
    }
}
