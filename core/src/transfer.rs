use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crate::error::{DuckerError, ErrorCode};
use crate::identity::DeviceIdentity;
use crate::protocol::{
    HandshakeStatus, TransferHandshakeRequest, TransferHandshakeResponse, TransferResult,
    DEFAULT_TRANSFER_PORT,
};

const CHUNK_SIZE: usize = 64 * 1024; // 64 KB

#[derive(Debug, Clone)]
pub enum TransferEvent {
    IncomingRequest {
        sender_name: String,
        sender_quac_id: u32,
        destination_quac_id: u32,
        file_name: String,
        file_size: u64,
    },
    Progress {
        file_name: String,
        bytes_received: u64,
        total_bytes: u64,
    },
    Completed {
        file_name: String,
        saved_path: PathBuf,
    },
    Rejected {
        reason: ErrorCode,
        message: String,
    },
    Failed {
        file_name: String,
        error: String,
    },
}

pub struct FileReceiver {
    identity: DeviceIdentity,
    save_dir: PathBuf,
    port: u16,
    event_sender: broadcast::Sender<TransferEvent>,
}

impl FileReceiver {
    pub fn new(identity: DeviceIdentity, save_dir: PathBuf) -> (Self, broadcast::Receiver<TransferEvent>) {
        let (tx, rx) = broadcast::channel(100);
        (
            Self {
                identity,
                save_dir,
                port: DEFAULT_TRANSFER_PORT,
                event_sender: tx,
            },
            rx,
        )
    }

    pub fn with_port(
        identity: DeviceIdentity,
        save_dir: PathBuf,
        port: u16,
    ) -> (Self, broadcast::Receiver<TransferEvent>) {
        let (tx, rx) = broadcast::channel(100);
        (
            Self {
                identity,
                save_dir,
                port,
                event_sender: tx,
            },
            rx,
        )
    }

    pub fn subscribe(&self) -> broadcast::Receiver<TransferEvent> {
        self.event_sender.subscribe()
    }

    pub fn default_save_dir() -> PathBuf {
        if let Some(download_dir) = dirs::download_dir() {
            download_dir.join("Ducker")
        } else if let Some(home) = dirs::home_dir() {
            home.join("Downloads").join("Ducker")
        } else {
            PathBuf::from("./ducker_downloads")
        }
    }

    pub async fn start(&self) -> Result<(), DuckerError> {
        let listener = TcpListener::bind(("0.0.0.0", self.port))
            .await
            .map_err(|e| DuckerError::ConnectionFailed(format!("Falha ao abrir porta {}: {}", self.port, e)))?;

        info!("Ducker Receiver aguardando conexões na porta {}", self.port);

        let identity = self.identity.clone();
        let save_dir = self.save_dir.clone();
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        info!("Nova conexão recebida de {}", peer_addr);
                        let id = identity.clone();
                        let dir = save_dir.clone();
                        let events = event_sender.clone();

                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(stream, id, dir, events).await {
                                warn!("Erro no tratamento de conexão de {}: {}", peer_addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Erro ao aceitar conexão TCP: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    async fn handle_connection(
        mut stream: TcpStream,
        identity: DeviceIdentity,
        save_dir: PathBuf,
        events: broadcast::Sender<TransferEvent>,
    ) -> Result<(), DuckerError> {
        // 1. Ler comprimento da requisição de handshake
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let msg_len = u32::from_be_bytes(len_buf) as usize;

        if msg_len > 10 * 1024 {
            return Err(DuckerError::InvalidRequest("Mensagem de handshake excessivamente grande.".into()));
        }

        // 2. Ler JSON da requisição
        let mut req_buf = vec![0u8; msg_len];
        stream.read_exact(&mut req_buf).await?;
        let request: TransferHandshakeRequest = serde_json::from_slice(&req_buf)
            .map_err(|e| DuckerError::InvalidRequest(format!("JSON inválido: {}", e)))?;

        let _ = events.send(TransferEvent::IncomingRequest {
            sender_name: request.sender_name.clone(),
            sender_quac_id: request.sender_quac_id,
            destination_quac_id: request.destination_quac_id,
            file_name: request.file_name.clone(),
            file_size: request.file_size,
        });

        // 3. Validação estrita do ID Quac de destino
        match request.validate_destination(identity.quac_id) {
            Ok(()) => {
                // Pergunta ao usuário se aceita o arquivo
                let prompt_msg = format!(
                    "Deseja aceitar o arquivo:\n\"{}\" ({:.2} MB)\n\nEnviado por: {} [Quac: {}]?",
                    request.file_name,
                    (request.file_size as f64) / 1024.0 / 1024.0,
                    request.sender_name,
                    request.sender_quac_id
                );
                let accepted = tokio::task::spawn_blocking(move || {
                    crate::dialog::prompt_user_acceptance("Ducker - Transferência Recebida", &prompt_msg)
                })
                .await
                .unwrap_or(true);

                if !accepted {
                    let err = DuckerError::TransferFailed("Transferência recusada pelo usuário no PC.".into());
                    let response = TransferHandshakeResponse::rejected(&err);
                    let resp_bytes = serde_json::to_vec(&response)?;
                    let resp_len = (resp_bytes.len() as u32).to_be_bytes();
                    stream.write_all(&resp_len).await?;
                    stream.write_all(&resp_bytes).await?;
                    stream.flush().await?;

                    let _ = events.send(TransferEvent::Rejected {
                        reason: err.code(),
                        message: err.user_friendly_message(),
                    });
                    return Err(err);
                }

                // ID correto e aceito: responder com ACCEPTED
                let response = TransferHandshakeResponse::accepted();
                let resp_bytes = serde_json::to_vec(&response)?;
                let resp_len = (resp_bytes.len() as u32).to_be_bytes();
                stream.write_all(&resp_len).await?;
                stream.write_all(&resp_bytes).await?;
                stream.flush().await?;
            }
            Err(validation_err) => {
                // ID INCORRETO: Rejeitar imediatamente e NÃO salvar nenhum arquivo
                let response = TransferHandshakeResponse::rejected(&validation_err);
                let resp_bytes = serde_json::to_vec(&response)?;
                let resp_len = (resp_bytes.len() as u32).to_be_bytes();
                stream.write_all(&resp_len).await?;
                stream.write_all(&resp_bytes).await?;
                stream.flush().await?;

                let _ = events.send(TransferEvent::Rejected {
                    reason: validation_err.code(),
                    message: validation_err.user_friendly_message(),
                });

                return Err(validation_err);
            }
        }

        // 4. Receber os bytes do arquivo e salvar
        tokio::fs::create_dir_all(&save_dir).await?;

        // Evitar sobrescrever arquivos existentes com o mesmo nome
        let safe_filename = Path::new(&request.file_name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("arquivo_recebido");

        let dest_file_path = save_dir.join(safe_filename);
        let mut file = tokio::fs::File::create(&dest_file_path).await?;

        let mut remaining = request.file_size;
        let mut received: u64 = 0;
        let mut buffer = [0u8; CHUNK_SIZE];

        while remaining > 0 {
            let to_read = std::cmp::min(remaining as usize, buffer.len());
            let n = stream.read(&mut buffer[..to_read]).await?;
            if n == 0 {
                let _ = tokio::fs::remove_file(&dest_file_path).await;
                let _ = events.send(TransferEvent::Failed {
                    file_name: request.file_name.clone(),
                    error: "Conexão interrompida antes da conclusão da transferência.".into(),
                });
                return Err(DuckerError::TransferFailed("Conexão interrompida prematuramente.".into()));
            }

            file.write_all(&buffer[..n]).await?;
            received += n as u64;
            remaining -= n as u64;

            let _ = events.send(TransferEvent::Progress {
                file_name: request.file_name.clone(),
                bytes_received: received,
                total_bytes: request.file_size,
            });
        }

        file.flush().await?;

        // 5. Enviar confirmação de conclusão
        let result = TransferResult::ok();
        let res_bytes = serde_json::to_vec(&result)?;
        let res_len = (res_bytes.len() as u32).to_be_bytes();
        stream.write_all(&res_len).await?;
        stream.write_all(&res_bytes).await?;
        stream.flush().await?;

        let _ = events.send(TransferEvent::Completed {
            file_name: request.file_name,
            saved_path: dest_file_path,
        });

        Ok(())
    }
}

pub struct FileSender;

impl FileSender {
    pub async fn send_file<F>(
        target_addr: &str,
        destination_quac_id: u32,
        sender_identity: &DeviceIdentity,
        file_path: &Path,
        on_progress: F,
    ) -> Result<(), DuckerError>
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        if !file_path.exists() {
            return Err(DuckerError::Custom(format!("Arquivo não encontrado: {}", file_path.display())));
        }

        let metadata = tokio::fs::metadata(file_path).await?;
        let file_size = metadata.len();
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("arquivo")
            .to_string();

        let mut stream = TcpStream::connect(target_addr).await.map_err(|e| {
            DuckerError::ConnectionFailed(format!("Não foi possível conectar a {}: {}", target_addr, e))
        })?;

        // 1. Enviar solicitação de handshake
        let handshake = TransferHandshakeRequest::new(
            &sender_identity.device_name,
            sender_identity.quac_id,
            destination_quac_id,
            &file_name,
            file_size,
        );

        let req_bytes = serde_json::to_vec(&handshake)?;
        let req_len = (req_bytes.len() as u32).to_be_bytes();
        stream.write_all(&req_len).await?;
        stream.write_all(&req_bytes).await?;
        stream.flush().await?;

        // 2. Receber resposta do handshake
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let resp_len = u32::from_be_bytes(len_buf) as usize;

        let mut resp_buf = vec![0u8; resp_len];
        stream.read_exact(&mut resp_buf).await?;
        let response: TransferHandshakeResponse = serde_json::from_slice(&resp_buf)
            .map_err(|e| DuckerError::InvalidRequest(format!("Resposta de handshake inválida: {}", e)))?;

        match response.status {
            HandshakeStatus::Accepted => (),
            HandshakeStatus::Rejected => {
                if let Some(reason) = response.reason {
                    if reason == ErrorCode::DestinationIdMismatch {
                        return Err(DuckerError::DestinationIdMismatch);
                    }
                }
                let msg = response.message.unwrap_or_else(|| "Transferência rejeitada pelo destinatário.".into());
                return Err(DuckerError::TransferFailed(msg));
            }
        }

        // 3. Enviar bytes do arquivo com progresso
        let mut file = tokio::fs::File::open(file_path).await?;
        let mut buffer = [0u8; CHUNK_SIZE];
        let mut sent: u64 = 0;

        on_progress(0, file_size);

        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }

            stream.write_all(&buffer[..n]).await?;
            sent += n as u64;
            on_progress(sent, file_size);
        }

        stream.flush().await?;

        // 4. Receber resultado final
        let mut res_len_buf = [0u8; 4];
        stream.read_exact(&mut res_len_buf).await?;
        let res_len = u32::from_be_bytes(res_len_buf) as usize;

        let mut res_buf = vec![0u8; res_len];
        stream.read_exact(&mut res_buf).await?;
        let result: TransferResult = serde_json::from_slice(&res_buf)
            .map_err(|e| DuckerError::InvalidRequest(format!("Resultado de transferência inválido: {}", e)))?;

        if !result.success {
            return Err(DuckerError::TransferFailed(result.message));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[tokio::test]
    async fn test_successful_transfer_and_validation() {
        let receiver_id = DeviceIdentity::with_id("Receptor Teste", 12345678);
        let sender_id = DeviceIdentity::with_id("Remetente Teste", 87654321);

        let temp_dir = tempfile::tempdir().unwrap();
        let save_dir = temp_dir.path().join("received");

        // Usar porta efêmera para teste
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let (receiver, _events) = FileReceiver::with_port(receiver_id.clone(), save_dir.clone(), port);
        receiver.start().await.unwrap();

        // Criar arquivo de teste para enviar
        let test_file_path = temp_dir.path().join("teste.txt");
        tokio::fs::write(&test_file_path, b"Hello Ducker Network!").await.unwrap();

        let progress_called = Arc::new(AtomicU64::new(0));
        let progress_clone = Arc::clone(&progress_called);

        let target_addr = format!("127.0.0.1:{}", port);
        let result = FileSender::send_file(
            &target_addr,
            12345678, // ID correto do receptor
            &sender_id,
            &test_file_path,
            move |sent, _total| {
                progress_clone.store(sent, Ordering::SeqCst);
            },
        ).await;

        assert!(result.is_ok(), "Transferência com ID correto deve ser bem-sucedida");
        assert!(progress_called.load(Ordering::SeqCst) > 0);

        let saved_file = save_dir.join("teste.txt");
        assert!(saved_file.exists(), "Arquivo deve ser salvo pelo receptor");
        let content = tokio::fs::read_to_string(&saved_file).await.unwrap();
        assert_eq!(content, "Hello Ducker Network!");
    }

    #[tokio::test]
    async fn test_rejection_on_destination_id_mismatch() {
        let receiver_id = DeviceIdentity::with_id("Receptor Teste", 12345678);
        let sender_id = DeviceIdentity::with_id("Remetente Teste", 87654321);

        let temp_dir = tempfile::tempdir().unwrap();
        let save_dir = temp_dir.path().join("received_mismatch");

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let (receiver, _events) = FileReceiver::with_port(receiver_id.clone(), save_dir.clone(), port);
        receiver.start().await.unwrap();

        let test_file_path = temp_dir.path().join("secret.txt");
        tokio::fs::write(&test_file_path, b"Do not save this file!").await.unwrap();

        let target_addr = format!("127.0.0.1:{}", port);
        // Remetente envia com ID de destino ERRADO (99999999 != 12345678)
        let result = FileSender::send_file(
            &target_addr,
            99999999,
            &sender_id,
            &test_file_path,
            |_sent, _total| {},
        ).await;

        assert!(result.is_err(), "Transferência com ID divergente deve ser rejeitada");
        match result.unwrap_err() {
            DuckerError::DestinationIdMismatch => (),
            other => panic!("Esperado erro DestinationIdMismatch, recebido: {:?}", other),
        }

        // CRÍTICO: verificar que NENHUM arquivo foi salvo no receptor
        let saved_file = save_dir.join("secret.txt");
        assert!(!saved_file.exists(), "Arquivo NÃO deve ser salvo quando o ID Quac não corresponder!");
    }
}
