//! Servidor HTTP(S) do protocolo LocalSend v2 (lado receptor).
//! Rotas: register, info, prepare-upload, upload, cancel.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::{
    body::{Body, Bytes},
    extract::{ConnectInfo, DefaultBodyLimit, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::StreamExt;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tokio::sync::oneshot;
use tracing::{info, warn};

use crate::model::{DeviceInfo, PrepareUploadRequest, PrepareUploadResponse, API_PREFIX};
use crate::node::{NodeEvent, NodeInner};
use crate::session::{sanitize_relative_path, unique_path, ReceiveSession, SessionFile};

pub(crate) fn router(inner: Arc<NodeInner>) -> Router {
    Router::new()
        .route(&format!("{API_PREFIX}/register"), post(register))
        .route(&format!("{API_PREFIX}/info"), get(info_handler))
        .route(&format!("{API_PREFIX}/prepare-upload"), post(prepare_upload))
        .route(&format!("{API_PREFIX}/upload"), post(upload))
        .route(&format!("{API_PREFIX}/cancel"), post(cancel))
        .layer(DefaultBodyLimit::disable())
        .with_state(inner)
}

async fn register(
    State(inner): State<Arc<NodeInner>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    body: Bytes,
) -> Response {
    let Ok(info) = serde_json::from_slice::<DeviceInfo>(&body) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    inner.add_peer_from_info(info, addr.ip());
    Json(inner.device_info()).into_response()
}

async fn info_handler(State(inner): State<Arc<NodeInner>>) -> Json<DeviceInfo> {
    Json(inner.device_info())
}

#[derive(Deserialize)]
struct PrepareQuery {
    pin: Option<String>,
    /// Extensão Ducker: ID Quac esperado do destinatário.
    quac: Option<u32>,
}

async fn prepare_upload(
    State(inner): State<Arc<NodeInner>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(q): Query<PrepareQuery>,
    body: Bytes,
) -> Response {
    if let Some(pin) = &inner.config.pin {
        if q.pin.as_deref() != Some(pin.as_str()) {
            return (StatusCode::UNAUTHORIZED, "PIN_REQUIRED").into_response();
        }
    }
    if let Some(quac) = q.quac {
        if quac != inner.identity().quac_id {
            warn!("Pedido interceptado: ID Quac {} não corresponde a este dispositivo", quac);
            return (StatusCode::FORBIDDEN, "DESTINATION_ID_MISMATCH").into_response();
        }
    }
    let Ok(req) = serde_json::from_slice::<PrepareUploadRequest>(&body) else {
        return (StatusCode::BAD_REQUEST, "Invalid body").into_response();
    };
    if req.files.is_empty() {
        return (StatusCode::BAD_REQUEST, "No files").into_response();
    }

    inner.add_peer_from_info(req.info.clone(), addr.ip());

    // Mensagens de texto: exibidas direto, sem upload (comportamento do LocalSend).
    if req.files.values().all(|f| f.is_text_message()) {
        for f in req.files.values() {
            inner.emit(NodeEvent::TextReceived {
                sender: req.info.clone(),
                text: f.preview.clone().unwrap_or_default(),
            });
        }
        return StatusCode::NO_CONTENT.into_response();
    }

    if inner.sessions.active.lock().unwrap().is_some() {
        return (StatusCode::CONFLICT, "Blocked by another session").into_response();
    }

    let session_id = uuid::Uuid::new_v4().to_string();
    let files: Vec<_> = req.files.values().cloned().collect();
    let auto = inner.config.auto_accept;

    inner.emit(NodeEvent::IncomingRequest {
        session_id: session_id.clone(),
        sender: req.info.clone(),
        files,
        auto_accepted: auto,
    });

    let accepted = if auto {
        true
    } else {
        let (tx, rx) = oneshot::channel();
        inner.sessions.pending.lock().unwrap().insert(session_id.clone(), tx);
        let result = tokio::time::timeout(inner.config.respond_timeout, rx).await;
        inner.sessions.pending.lock().unwrap().remove(&session_id);
        matches!(result, Ok(Ok(true)))
    };

    if !accepted {
        inner.emit(NodeEvent::SessionCancelled { session_id });
        return (StatusCode::FORBIDDEN, "Rejected").into_response();
    }

    let mut active = inner.sessions.active.lock().unwrap();
    if active.is_some() {
        return (StatusCode::CONFLICT, "Blocked by another session").into_response();
    }
    let mut tokens = HashMap::new();
    let mut session_files = HashMap::new();
    for (id, dto) in req.files {
        let token = uuid::Uuid::new_v4().simple().to_string();
        tokens.insert(id.clone(), token.clone());
        session_files.insert(id, SessionFile { dto, token, done: false });
    }
    *active = Some(ReceiveSession {
        id: session_id.clone(),
        sender: req.info,
        sender_ip: addr.ip(),
        files: session_files,
        cancelled: Arc::new(AtomicBool::new(false)),
    });
    drop(active);

    info!("Sessão {} aceita ({} arquivo(s))", session_id, tokens.len());
    Json(PrepareUploadResponse { session_id, files: tokens }).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UploadQuery {
    session_id: String,
    file_id: String,
    token: String,
}

async fn upload(
    State(inner): State<Arc<NodeInner>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(q): Query<UploadQuery>,
    body: Body,
) -> Response {
    // 1. Validar sessão / token / IP
    let (dto, cancelled) = {
        let guard = inner.sessions.active.lock().unwrap();
        let Some(session) = guard.as_ref() else {
            return (StatusCode::CONFLICT, "No active session").into_response();
        };
        if session.id != q.session_id || session.sender_ip != addr.ip() {
            return (StatusCode::FORBIDDEN, "Invalid session or IP").into_response();
        }
        let Some(file) = session.files.get(&q.file_id) else {
            return (StatusCode::FORBIDDEN, "Unknown file").into_response();
        };
        if file.token != q.token || file.done {
            return (StatusCode::FORBIDDEN, "Invalid token").into_response();
        }
        (file.dto.clone(), session.cancelled.clone())
    };

    let save_dir = inner.config.save_dir.clone();
    if let Err(e) = tokio::fs::create_dir_all(&save_dir).await {
        warn!("Falha ao criar pasta de destino: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // 2. Gravar em arquivo temporário, calculando SHA-256 se fornecido
    let part_path = save_dir.join(format!(".{}.ducker-part", uuid::Uuid::new_v4().simple()));
    let mut file = match tokio::fs::File::create(&part_path).await {
        Ok(f) => f,
        Err(e) => {
            warn!("Falha ao criar arquivo temporário: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let mut hasher = dto.sha256.as_ref().map(|_| Sha256::new());
    let mut stream = body.into_data_stream();
    let mut received: u64 = 0;
    let mut last_emit: u64 = 0;
    let step = (dto.size / 100).max(256 * 1024);

    let fail = |status: StatusCode, part: std::path::PathBuf| async move {
        let _ = tokio::fs::remove_file(part).await;
        status.into_response()
    };

    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(c) => c,
            Err(e) => {
                warn!("Conexão interrompida durante upload: {e}");
                drop(file);
                return fail(StatusCode::INTERNAL_SERVER_ERROR, part_path).await;
            }
        };
        if cancelled.load(Ordering::SeqCst) {
            drop(file);
            return fail(StatusCode::CONFLICT, part_path).await;
        }
        if let Err(e) = file.write_all(&chunk).await {
            warn!("Erro ao gravar arquivo: {e}");
            drop(file);
            return fail(StatusCode::INTERNAL_SERVER_ERROR, part_path).await;
        }
        if let Some(h) = hasher.as_mut() {
            h.update(&chunk);
        }
        received += chunk.len() as u64;
        if received - last_emit >= step || received >= dto.size {
            last_emit = received;
            inner.emit(NodeEvent::ReceiveProgress {
                session_id: q.session_id.clone(),
                file_id: q.file_id.clone(),
                received,
                total: dto.size,
            });
        }
    }
    if file.flush().await.is_err() {
        drop(file);
        return fail(StatusCode::INTERNAL_SERVER_ERROR, part_path).await;
    }
    drop(file);

    if let (Some(h), Some(expected)) = (hasher, dto.sha256.as_ref()) {
        let actual = hex::encode(h.finalize());
        if !actual.eq_ignore_ascii_case(expected) {
            warn!("Checksum divergente para {}", dto.file_name);
            return fail(StatusCode::UNPROCESSABLE_ENTITY, part_path).await;
        }
    }

    // 3. Mover para o nome final (sem sobrescrever)
    let rel = sanitize_relative_path(&dto.file_name);
    let final_path = unique_path(&save_dir, &rel);
    if let Some(parent) = final_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    if let Err(e) = tokio::fs::rename(&part_path, &final_path).await {
        warn!("Falha ao mover arquivo recebido: {e}");
        return fail(StatusCode::INTERNAL_SERVER_ERROR, part_path).await;
    }

    // 4. Marcar como concluído
    let all_done = {
        let mut guard = inner.sessions.active.lock().unwrap();
        let mut all_done = false;
        if let Some(session) = guard.as_mut().filter(|s| s.id == q.session_id) {
            if let Some(f) = session.files.get_mut(&q.file_id) {
                f.done = true;
            }
            all_done = session.files.values().all(|f| f.done);
        }
        if all_done {
            *guard = None;
        }
        all_done
    };

    info!("Arquivo recebido: {}", final_path.display());
    inner.emit(NodeEvent::FileReceived {
        session_id: q.session_id.clone(),
        file_id: q.file_id.clone(),
        file_name: dto.file_name.clone(),
        path: final_path.to_string_lossy().to_string(),
    });
    if all_done {
        inner.emit(NodeEvent::SessionFinished { session_id: q.session_id });
    }
    StatusCode::OK.into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CancelQuery {
    session_id: String,
}

async fn cancel(State(inner): State<Arc<NodeInner>>, Query(q): Query<CancelQuery>) -> StatusCode {
    let mut cancelled = false;
    {
        let mut guard = inner.sessions.active.lock().unwrap();
        if guard.as_ref().map(|s| s.id == q.session_id).unwrap_or(false) {
            if let Some(s) = guard.take() {
                s.cancelled.store(true, Ordering::SeqCst);
            }
            cancelled = true;
        }
    }
    if let Some(tx) = inner.sessions.pending.lock().unwrap().remove(&q.session_id) {
        let _ = tx.send(false);
        cancelled = true;
    }
    if cancelled {
        inner.emit(NodeEvent::SessionCancelled { session_id: q.session_id });
    }
    StatusCode::OK
}
