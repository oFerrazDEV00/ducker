use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::{
    body::{Body, Bytes},
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path as AxumPath, State,
    },
    http::{header, HeaderMap, StatusCode},
    response::{Html, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, oneshot, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

use crate::discovery::DiscoveryManager;
use crate::error::{DuckerError, ErrorCode};
use crate::identity::DeviceIdentity;
use crate::protocol::{HandshakeStatus, CURRENT_PROTOCOL_VERSION};
use crate::transfer::FileReceiver;

pub const DEFAULT_MOBILE_BRIDGE_PORT: u16 = 7876;

const DOWNLOAD_CHUNK_SIZE: usize = 64 * 1024;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(30);
const COMPLETION_TIMEOUT: Duration = Duration::from_secs(30 * 60);

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

#[derive(Debug, Clone, Deserialize)]
pub struct MobileMessagePayload {
    pub sender_name: String,
    pub sender_quac_id: u32,
    pub destination_quac_id: u32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MobileApiResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saved_path: Option<String>,
}

/// Mensagens trocadas via WebSocket com o app Mobile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MobileWsMessage {
    #[serde(rename = "mobile_announce")]
    Announce { quac_id: u32, device_name: String },
    TransferRequest {
        transfer_id: String,
        protocol_version: u32,
        sender_name: String,
        sender_quac_id: u32,
        destination_quac_id: u32,
        file_name: String,
        file_size: u64,
        download_path: String,
    },
    HandshakeResponse {
        transfer_id: String,
        status: HandshakeStatus,
        #[serde(default)]
        reason: Option<String>,
        #[serde(default)]
        message: Option<String>,
    },
    TransferProgress {
        transfer_id: String,
        bytes_sent: u64,
        total_bytes: u64,
    },
    TransferComplete {
        transfer_id: String,
        success: bool,
        #[serde(default)]
        message: Option<String>,
    },
    TransferStatus {
        #[serde(default)]
        transfer_id: String,
        status: String,
        message: String,
    },
}

/// Celular atualmente conectado ao bridge
#[derive(Debug, Clone)]
pub struct ConnectedMobile {
    pub quac_id: u32,
    pub device_name: String,
}

/// Eventos emitidos durante `send_to_mobile` (para UI de progresso)
#[derive(Debug, Clone)]
pub enum MobileSendEvent {
    WaitingAcceptance,
    Accepted,
    Progress { sent: u64, total: u64 },
}

struct MobileClient {
    device_name: String,
    tx: mpsc::Sender<Message>,
}

pub struct ActiveTransfer {
    pub id: String,
    pub file_name: String,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub destination_quac_id: u32,
    pub bytes_sent: AtomicU64,
}

struct HandshakeReply {
    status: HandshakeStatus,
    reason: Option<String>,
    message: Option<String>,
}

pub struct MobileBridgeState {
    pub identity: DeviceIdentity,
    pub discovery: Arc<DiscoveryManager>,
    pub save_dir: PathBuf,
    mobile_clients: RwLock<HashMap<u32, MobileClient>>,
    pending_transfers: RwLock<HashMap<String, Arc<ActiveTransfer>>>,
    handshake_waiters: Mutex<HashMap<String, oneshot::Sender<HandshakeReply>>>,
    completion_waiters: Mutex<HashMap<String, oneshot::Sender<Result<(), String>>>>,
}

pub struct MobileBridge {
    state: Arc<MobileBridgeState>,
    port: u16,
}

impl MobileBridge {
    pub fn new(identity: DeviceIdentity, discovery: Arc<DiscoveryManager>) -> Self {
        Self::with_port(identity, discovery, DEFAULT_MOBILE_BRIDGE_PORT)
    }

    pub fn with_port(identity: DeviceIdentity, discovery: Arc<DiscoveryManager>, port: u16) -> Self {
        let save_dir = FileReceiver::default_save_dir();
        Self::with_port_and_save_dir(identity, discovery, port, save_dir)
    }

    pub fn with_port_and_save_dir(
        identity: DeviceIdentity,
        discovery: Arc<DiscoveryManager>,
        port: u16,
        save_dir: PathBuf,
    ) -> Self {
        Self {
            state: Arc::new(MobileBridgeState {
                identity,
                discovery,
                save_dir,
                mobile_clients: RwLock::new(HashMap::new()),
                pending_transfers: RwLock::new(HashMap::new()),
                handshake_waiters: Mutex::new(HashMap::new()),
                completion_waiters: Mutex::new(HashMap::new()),
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
            .route("/", get(serve_dashboard))
            .route("/api/status", get(get_status))
            .route("/api/devices", get(get_devices))
            .route("/api/ws", get(ws_handler))
            .route("/api/download/:transfer_id", get(download_file))
            .route("/api/upload", post(upload_file))
            .route("/api/message", post(receive_message))
            .route("/api/pc/send-message", post(pc_send_message))
            .route("/api/pc/upload-and-send", post(pc_upload_and_send))
            .route("/api/pc/open-folder", post(pc_open_folder))
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

async fn serve_dashboard(State(state): State<Arc<MobileBridgeState>>) -> Html<String> {
    let local_ip = DiscoveryManager::get_local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| "127.0.0.1".to_string());

    let template = include_str!("dashboard.html");
    let html = template
        .replace("{{DEVICE_NAME}}", &state.identity.device_name)
        .replace("{{QUAC_ID}}", &state.identity.quac_id.to_string())
        .replace("{{ADDRESS}}", &local_ip)
        .replace("{{PORT}}", &DEFAULT_MOBILE_BRIDGE_PORT.to_string())
        .replace("{{SAVE_DIR}}", &state.save_dir.display().to_string());

    Html(html)
}

#[derive(Debug, Deserialize)]
struct PcSendMessagePayload {
    text: String,
    to: Option<u32>,
}

async fn pc_send_message(
    State(state): State<Arc<MobileBridgeState>>,
    Json(payload): Json<PcSendMessagePayload>,
) -> Json<MobileApiResponse> {
    let mobiles = state.connected_mobiles().await;
    let target = if let Some(wanted) = payload.to {
        mobiles.into_iter().find(|m| m.quac_id == wanted)
    } else {
        mobiles.into_iter().next()
    };

    let target = match target {
        Some(t) => t,
        None => {
            return Json(MobileApiResponse {
                success: false,
                message: "Nenhum celular conectado ao Ducker no momento. Abra o app no celular!".to_string(),
                file_name: None,
                saved_path: None,
            });
        }
    };

    let temp_file = std::env::temp_dir().join(format!("mensagem_{}.txt", rand::random::<u32>()));
    if let Err(e) = tokio::fs::write(&temp_file, payload.text.as_bytes()).await {
        return Json(MobileApiResponse {
            success: false,
            message: format!("Erro ao gerar arquivo de mensagem: {}", e),
            file_name: None,
            saved_path: None,
        });
    }

    let temp_clone = temp_file.clone();
    let res = state.send_to_mobile(target.quac_id, temp_file, |_| {}).await;
    let _ = tokio::fs::remove_file(temp_clone).await;

    match res {
        Ok(()) => Json(MobileApiResponse {
            success: true,
            message: format!("Mensagem entregue com sucesso para {}!", target.device_name),
            file_name: None,
            saved_path: None,
        }),
        Err(e) => Json(MobileApiResponse {
            success: false,
            message: format!("Falha no envio: {}", e.user_friendly_message()),
            file_name: None,
            saved_path: None,
        }),
    }
}

async fn pc_upload_and_send(
    State(state): State<Arc<MobileBridgeState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Json<MobileApiResponse> {
    let raw_name = headers
        .get("x-file-name")
        .and_then(|h| h.to_str().ok())
        .map(|s| percent_decode_str(s))
        .unwrap_or_else(|| "arquivo_web".to_string());

    let mobiles = state.connected_mobiles().await;
    let target = match mobiles.into_iter().next() {
        Some(t) => t,
        None => {
            return Json(MobileApiResponse {
                success: false,
                message: "Nenhum celular conectado ao Ducker no momento. Abra o app no celular!".to_string(),
                file_name: None,
                saved_path: None,
            });
        }
    };

    let temp_dir = std::env::temp_dir().join("ducker_web_uploads");
    let _ = tokio::fs::create_dir_all(&temp_dir).await;
    let temp_file = temp_dir.join(&raw_name);

    if let Err(e) = tokio::fs::write(&temp_file, &body).await {
        return Json(MobileApiResponse {
            success: false,
            message: format!("Falha ao processar arquivo: {}", e),
            file_name: None,
            saved_path: None,
        });
    }

    let temp_clone = temp_file.clone();
    let res = state.send_to_mobile(target.quac_id, temp_file, |_| {}).await;
    let _ = tokio::fs::remove_file(temp_clone).await;

    match res {
        Ok(()) => Json(MobileApiResponse {
            success: true,
            message: format!("Arquivo '{}' entregue com sucesso para {}!", raw_name, target.device_name),
            file_name: Some(raw_name),
            saved_path: None,
        }),
        Err(e) => Json(MobileApiResponse {
            success: false,
            message: format!("Falha no envio: {}", e.user_friendly_message()),
            file_name: Some(raw_name),
            saved_path: None,
        }),
    }
}

async fn pc_open_folder(State(state): State<Arc<MobileBridgeState>>) -> Json<MobileApiResponse> {
    let dir = &state.save_dir;
    let _ = tokio::fs::create_dir_all(dir).await;

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer.exe")
            .arg(dir)
            .spawn();
    }

    Json(MobileApiResponse {
        success: true,
        message: "Pasta de downloads aberta no Windows Explorer.".to_string(),
        file_name: None,
        saved_path: Some(dir.display().to_string()),
    })
}


async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<MobileBridgeState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn send_ws(tx: &mpsc::Sender<Message>, msg: &MobileWsMessage) -> bool {
    match serde_json::to_string(msg) {
        Ok(json) => tx.send(Message::Text(json)).await.is_ok(),
        Err(_) => false,
    }
}

async fn handle_socket(socket: WebSocket, state: Arc<MobileBridgeState>) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<Message>(64);

    let sender_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut registered_quac_id: Option<u32> = None;

    while let Some(Ok(msg)) = receiver.next().await {
        let text = match msg {
            Message::Text(text) => text,
            Message::Close(_) => break,
            _ => continue,
        };

        let ws_msg = match serde_json::from_str::<MobileWsMessage>(&text) {
            Ok(m) => m,
            Err(e) => {
                warn!("Mensagem WS inválida do celular ({}): {}", e, text);
                continue;
            }
        };

        match ws_msg {
            MobileWsMessage::Announce { quac_id, device_name } => {
                info!("Mobile device anunciado: {} [{}]", device_name, quac_id);
                registered_quac_id = Some(quac_id);

                state.mobile_clients.write().await.insert(
                    quac_id,
                    MobileClient {
                        device_name,
                        tx: tx.clone(),
                    },
                );

                // Envia confirmação de boas vindas
                let welcome = MobileWsMessage::TransferStatus {
                    transfer_id: String::new(),
                    status: "CONNECTED".to_string(),
                    message: format!("Conectado ao Ducker Node: {}", state.identity.device_name),
                };
                send_ws(&tx, &welcome).await;
            }
            MobileWsMessage::HandshakeResponse {
                transfer_id,
                status,
                reason,
                message,
            } => {
                info!("Resposta de handshake do Mobile [{}]: {:?}", transfer_id, status);
                let waiter = state.handshake_waiters.lock().unwrap().remove(&transfer_id);
                if let Some(waiter) = waiter {
                    let _ = waiter.send(HandshakeReply { status, reason, message });
                }
            }
            MobileWsMessage::TransferComplete {
                transfer_id,
                success,
                message,
            } => {
                let waiter = state.completion_waiters.lock().unwrap().remove(&transfer_id);
                if let Some(waiter) = waiter {
                    let result = if success {
                        Ok(())
                    } else {
                        Err(message.unwrap_or_else(|| "O celular não conseguiu salvar o arquivo.".into()))
                    };
                    let _ = waiter.send(result);
                }
            }
            _ => {}
        }
    }

    if let Some(id) = registered_quac_id {
        let mut clients = state.mobile_clients.write().await;
        if clients.get(&id).map(|c| c.tx.same_channel(&tx)).unwrap_or(false) {
            clients.remove(&id);
        }
    }
    sender_task.abort();
}

fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for b in input.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

async fn download_file(
    AxumPath(transfer_id): AxumPath<String>,
    State(state): State<Arc<MobileBridgeState>>,
) -> Result<Response, StatusCode> {
    let transfer = state
        .pending_transfers
        .read()
        .await
        .get(&transfer_id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;

    let file = tokio::fs::File::open(&transfer.file_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client_tx = state
        .mobile_clients
        .read()
        .await
        .get(&transfer.destination_quac_id)
        .map(|c| c.tx.clone());

    transfer.bytes_sent.store(0, Ordering::SeqCst);

    let stream = futures_util::stream::unfold(
        (file, Arc::clone(&transfer), client_tx, 0u64),
        |(mut file, transfer, tx, mut last_reported)| async move {
            let mut buf = vec![0u8; DOWNLOAD_CHUNK_SIZE];
            match file.read(&mut buf).await {
                Ok(0) => None,
                Ok(n) => {
                    buf.truncate(n);
                    let sent = transfer.bytes_sent.fetch_add(n as u64, Ordering::SeqCst) + n as u64;
                    let step = (transfer.file_size / 50).max(DOWNLOAD_CHUNK_SIZE as u64);
                    if let Some(ref tx) = tx {
                        if sent - last_reported >= step || sent >= transfer.file_size {
                            last_reported = sent;
                            let msg = MobileWsMessage::TransferProgress {
                                transfer_id: transfer.id.clone(),
                                bytes_sent: sent,
                                total_bytes: transfer.file_size,
                            };
                            if let Ok(json) = serde_json::to_string(&msg) {
                                let _ = tx.try_send(Message::Text(json));
                            }
                        }
                    }
                    Some((Ok::<Bytes, std::io::Error>(Bytes::from(buf)), (file, transfer, tx, last_reported)))
                }
                Err(e) => Some((Err(e), (file, transfer, tx, last_reported))),
            }
        },
    );

    let ascii_name: String = transfer
        .file_name
        .chars()
        .map(|c| if c.is_ascii_graphic() && c != '"' && c != '\\' { c } else { '_' })
        .collect();

    Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_LENGTH, transfer.file_size)
        .header(
            header::CONTENT_DISPOSITION,
            format!(
                "attachment; filename=\"{}\"; filename*=UTF-8''{}",
                ascii_name,
                percent_encode(&transfer.file_name)
            ),
        )
        .body(Body::from_stream(stream))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Recebe uma mensagem de texto curta enviada pelo iPhone
async fn receive_message(
    State(state): State<Arc<MobileBridgeState>>,
    Json(payload): Json<MobileMessagePayload>,
) -> Result<Json<MobileApiResponse>, (StatusCode, Json<MobileApiResponse>)> {
    // Validação estrita do ID Quac de destino
    if payload.destination_quac_id != state.identity.quac_id {
        eprintln!(
            "\n❌ Tentativa de envio de mensagem interceptada: destino {} não confere com este PC ({})",
            payload.destination_quac_id, state.identity.quac_id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(MobileApiResponse {
                success: false,
                message: "O ID Quac de destino não corresponde a este computador.".into(),
                file_name: None,
                saved_path: None,
            }),
        ));
    }

    // Confirmação do usuário via caixa de diálogo nativa
    let prompt_msg = format!(
        "De: {} [Quac: {}]\n\n\"{}\"\n\nDeseja salvar esta mensagem no seu PC?",
        payload.sender_name, payload.sender_quac_id, payload.text
    );
    let accepted = tokio::task::spawn_blocking(move || {
        crate::dialog::prompt_user_acceptance("Ducker - Mensagem do Celular", &prompt_msg)
    })
    .await
    .unwrap_or(true);

    if !accepted {
        println!("\n⛔ Mensagem de {} recusada pelo usuário.", payload.sender_name);
        return Err((
            StatusCode::FORBIDDEN,
            Json(MobileApiResponse {
                success: false,
                message: "Mensagem recusada pelo usuário no PC.".into(),
                file_name: None,
                saved_path: None,
            }),
        ));
    }

    let _ = tokio::fs::create_dir_all(&state.save_dir).await;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let file_name = format!("mensagem_celular_{}.txt", now);
    let file_path = state.save_dir.join(&file_name);
    let body = format!(
        "De: {} [Quac: {}]\nData: (timestamp {})\n\n{}\n",
        payload.sender_name, payload.sender_quac_id, now, payload.text
    );

    if let Err(e) = tokio::fs::write(&file_path, body).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MobileApiResponse {
                success: false,
                message: format!("Falha ao salvar arquivo no PC: {}", e),
                file_name: None,
                saved_path: None,
            }),
        ));
    }

    println!(
        "\n📨 Nova mensagem recebida do Celular!\n  De:       {} [Quac: {}]\n  Texto:    \"{}\"\n  Salvo em: {}\n",
        payload.sender_name,
        payload.sender_quac_id,
        payload.text,
        file_path.display()
    );

    Ok(Json(MobileApiResponse {
        success: true,
        message: "Mensagem recebida com sucesso no PC!".into(),
        file_name: Some(file_name),
        saved_path: Some(file_path.to_string_lossy().to_string()),
    }))
}

/// Recebe um arquivo ou foto enviado pelo iPhone (upload HTTP binário)
async fn upload_file(
    State(state): State<Arc<MobileBridgeState>>,
    headers: HeaderMap,
    body: Body,
) -> Result<Json<MobileApiResponse>, (StatusCode, Json<MobileApiResponse>)> {
    let sender_name = headers
        .get("x-sender-name")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Celular")
        .to_string();

    let sender_quac_id = headers
        .get("x-sender-quac-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    let destination_quac_id = headers
        .get("x-destination-quac-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    let raw_file_name = headers
        .get("x-file-name")
        .and_then(|h| h.to_str().ok())
        .map(|s| {
            percent_decode_str(s)
        })
        .unwrap_or_else(|| "arquivo_celular.bin".to_string());

    // Validação estrita do ID Quac de destino
    if destination_quac_id != state.identity.quac_id {
        eprintln!(
            "\n❌ Tentativa de upload interceptada: destino {} não confere com este PC ({})",
            destination_quac_id, state.identity.quac_id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(MobileApiResponse {
                success: false,
                message: "O ID Quac de destino não corresponde a este computador.".into(),
                file_name: None,
                saved_path: None,
            }),
        ));
    }

    let safe_base_name = Path::new(&raw_file_name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("arquivo_celular.bin");

    // Confirmação do usuário via caixa de diálogo nativa
    let prompt_msg = format!(
        "Deseja aceitar o arquivo:\n\"{}\"\n\nEnviado por: {} [Quac: {}]?",
        safe_base_name, sender_name, sender_quac_id
    );
    let accepted = tokio::task::spawn_blocking(move || {
        crate::dialog::prompt_user_acceptance("Ducker - Recebimento de Arquivo", &prompt_msg)
    })
    .await
    .unwrap_or(true);

    if !accepted {
        println!("\n⛔ Transferência de \"{}\" rejeitada pelo usuário.", safe_base_name);
        return Err((
            StatusCode::FORBIDDEN,
            Json(MobileApiResponse {
                success: false,
                message: "Transferência rejeitada pelo usuário no PC.".into(),
                file_name: None,
                saved_path: None,
            }),
        ));
    }

    let _ = tokio::fs::create_dir_all(&state.save_dir).await;

    // Gerar caminho único seguro para não sobrescrever
    let dest_path = unique_path_in(&state.save_dir, safe_base_name);

    let mut dest_file = tokio::fs::File::create(&dest_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MobileApiResponse {
                success: false,
                message: format!("Falha ao criar arquivo no PC: {}", e),
                file_name: None,
                saved_path: None,
            }),
        )
    })?;

    // Streaming dos bytes recebidos
    let mut stream = body.into_data_stream();
    let mut total_bytes = 0u64;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MobileApiResponse {
                    success: false,
                    message: format!("Erro ao receber fluxo de dados: {}", e),
                    file_name: None,
                    saved_path: None,
                }),
            )
        })?;

        dest_file.write_all(&chunk).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MobileApiResponse {
                    success: false,
                    message: format!("Erro ao gravar no disco: {}", e),
                    file_name: None,
                    saved_path: None,
                }),
            )
        })?;
        total_bytes += chunk.len() as u64;
    }

    let _ = dest_file.flush().await;

    let display_name = dest_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("arquivo")
        .to_string();

    println!(
        "\n📦 Arquivo recebido do Celular!\n  Arquivo:  {} ({:.2} MB)\n  De:       {} [Quac: {}]\n  Salvo em: {}\n",
        display_name,
        (total_bytes as f64) / 1024.0 / 1024.0,
        sender_name,
        sender_quac_id,
        dest_path.display()
    );

    Ok(Json(MobileApiResponse {
        success: true,
        message: "Arquivo recebido e salvo com sucesso no PC!".into(),
        file_name: Some(display_name),
        saved_path: Some(dest_path.to_string_lossy().to_string()),
    }))
}

fn percent_decode_str(s: &str) -> String {
    let mut bytes = Vec::new();
    let mut chars = s.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let h1 = chars.next();
            let h2 = chars.next();
            if let (Some(h1), Some(h2)) = (h1, h2) {
                if let Ok(val) = u8::from_str_radix(&format!("{}{}", h1 as char, h2 as char), 16) {
                    bytes.push(val);
                    continue;
                }
            }
        }
        bytes.push(b);
    }
    String::from_utf8(bytes).unwrap_or_else(|_| s.to_string())
}

fn unique_path_in(dir: &Path, file_name: &str) -> PathBuf {
    let candidate = dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }

    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("arquivo");
    let ext = Path::new(file_name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();

    for i in 1..1000 {
        let test_name = format!("{} ({}){}", stem, i, ext);
        let path = dir.join(test_name);
        if !path.exists() {
            return path;
        }
    }

    dir.join(format!("{}_{}{}", stem, rand::random::<u32>(), ext))
}

impl MobileBridgeState {
    /// Lista os celulares conectados no momento via WebSocket
    pub async fn connected_mobiles(&self) -> Vec<ConnectedMobile> {
        self.mobile_clients
            .read()
            .await
            .iter()
            .map(|(quac_id, c)| ConnectedMobile {
                quac_id: *quac_id,
                device_name: c.device_name.clone(),
            })
            .collect()
    }

    /// Envia um arquivo para um celular conectado.
    pub async fn send_to_mobile<F>(
        &self,
        destination_quac_id: u32,
        file_path: PathBuf,
        mut on_event: F,
    ) -> Result<(), DuckerError>
    where
        F: FnMut(MobileSendEvent) + Send,
    {
        let client_tx = self
            .mobile_clients
            .read()
            .await
            .get(&destination_quac_id)
            .map(|c| c.tx.clone())
            .ok_or(DuckerError::DeviceNotFound)?;

        let metadata = tokio::fs::metadata(&file_path).await?;
        if !metadata.is_file() {
            return Err(DuckerError::InvalidRequest("o caminho informado não é um arquivo".into()));
        }

        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("arquivo")
            .to_string();
        let file_size = metadata.len();
        let transfer_id = format!("t_{}_{:08x}", destination_quac_id, rand::random::<u32>());

        let transfer = Arc::new(ActiveTransfer {
            id: transfer_id.clone(),
            file_name: file_name.clone(),
            file_path,
            file_size,
            destination_quac_id,
            bytes_sent: AtomicU64::new(0),
        });

        self.pending_transfers
            .write()
            .await
            .insert(transfer_id.clone(), Arc::clone(&transfer));
        let (hs_tx, hs_rx) = oneshot::channel();
        self.handshake_waiters.lock().unwrap().insert(transfer_id.clone(), hs_tx);
        let (done_tx, done_rx) = oneshot::channel();
        self.completion_waiters.lock().unwrap().insert(transfer_id.clone(), done_tx);

        let result = self
            .run_transfer(&client_tx, &transfer, hs_rx, done_rx, &mut on_event)
            .await;

        self.pending_transfers.write().await.remove(&transfer_id);
        self.handshake_waiters.lock().unwrap().remove(&transfer_id);
        self.completion_waiters.lock().unwrap().remove(&transfer_id);

        result
    }

    async fn run_transfer<F>(
        &self,
        client_tx: &mpsc::Sender<Message>,
        transfer: &Arc<ActiveTransfer>,
        hs_rx: oneshot::Receiver<HandshakeReply>,
        mut done_rx: oneshot::Receiver<Result<(), String>>,
        on_event: &mut F,
    ) -> Result<(), DuckerError>
    where
        F: FnMut(MobileSendEvent) + Send,
    {
        let request = MobileWsMessage::TransferRequest {
            transfer_id: transfer.id.clone(),
            protocol_version: CURRENT_PROTOCOL_VERSION,
            sender_name: self.identity.device_name.clone(),
            sender_quac_id: self.identity.quac_id,
            destination_quac_id: transfer.destination_quac_id,
            file_name: transfer.file_name.clone(),
            file_size: transfer.file_size,
            download_path: format!("/api/download/{}", transfer.id),
        };
        if !send_ws(client_tx, &request).await {
            return Err(DuckerError::ConnectionFailed("o celular desconectou.".into()));
        }
        on_event(MobileSendEvent::WaitingAcceptance);

        let reply = match tokio::time::timeout(HANDSHAKE_TIMEOUT, hs_rx).await {
            Ok(Ok(reply)) => reply,
            Ok(Err(_)) => return Err(DuckerError::TransferFailed("Canal de resposta fechado.".into())),
            Err(_) => return Err(DuckerError::TransferFailed("Destinatário demorou a responder.".into())),
        };

        if reply.status == HandshakeStatus::Rejected {
            if reply.reason.as_deref() == Some(ErrorCode::DestinationIdMismatch.as_str()) {
                return Err(DuckerError::DestinationIdMismatch);
            }
            return Err(DuckerError::TransferFailed(
                reply.message.unwrap_or_else(|| "Rejeitado pelo destinatário.".into()),
            ));
        }

        info!("Handshake aceito pelo Mobile para {}", transfer.file_name);
        on_event(MobileSendEvent::Accepted);

        let deadline = tokio::time::sleep(COMPLETION_TIMEOUT);
        tokio::pin!(deadline);
        let mut tick = tokio::time::interval(Duration::from_millis(200));

        loop {
            tokio::select! {
                res = &mut done_rx => {
                    on_event(MobileSendEvent::Progress {
                        sent: transfer.bytes_sent.load(Ordering::SeqCst),
                        total: transfer.file_size,
                    });
                    return match res {
                        Ok(Ok(())) => Ok(()),
                        Ok(Err(msg)) => Err(DuckerError::TransferFailed(msg)),
                        Err(_) => Err(DuckerError::TransferFailed("Canal de confirmação fechado.".into())),
                    };
                }
                _ = &mut deadline => {
                    return Err(DuckerError::TransferFailed("Tempo limite excedido aguardando o celular.".into()));
                }
                _ = tick.tick() => {
                    on_event(MobileSendEvent::Progress {
                        sent: transfer.bytes_sent.load(Ordering::SeqCst),
                        total: transfer.file_size,
                    });
                    if !self.mobile_clients.read().await.contains_key(&transfer.destination_quac_id) {
                        return Err(DuckerError::ConnectionFailed("o celular desconectou durante a transferência.".into()));
                    }
                }
            }
        }
    }
}
