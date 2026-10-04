use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    #[serde(rename = "DESTINATION_ID_MISMATCH")]
    DestinationIdMismatch,
    #[serde(rename = "DEVICE_NOT_FOUND")]
    DeviceNotFound,
    #[serde(rename = "CONNECTION_FAILED")]
    ConnectionFailed,
    #[serde(rename = "TRANSFER_FAILED")]
    TransferFailed,
    #[serde(rename = "INVALID_REQUEST")]
    InvalidRequest,
    #[serde(rename = "PROTOCOL_VERSION_MISMATCH")]
    ProtocolVersionMismatch,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::DestinationIdMismatch => "DESTINATION_ID_MISMATCH",
            ErrorCode::DeviceNotFound => "DEVICE_NOT_FOUND",
            ErrorCode::ConnectionFailed => "CONNECTION_FAILED",
            ErrorCode::TransferFailed => "TRANSFER_FAILED",
            ErrorCode::InvalidRequest => "INVALID_REQUEST",
            ErrorCode::ProtocolVersionMismatch => "PROTOCOL_VERSION_MISMATCH",
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Error, Debug)]
pub enum DuckerError {
    #[error("Ocorreu um erro: a transferência foi interceptada porque o ID Quac de destino não corresponde a este dispositivo. (DESTINATION_ID_MISMATCH)")]
    DestinationIdMismatch,

    #[error("Dispositivo não encontrado ou indisponível na rede. (DEVICE_NOT_FOUND)")]
    DeviceNotFound,

    #[error("Não foi possível estabelecer conexão com o dispositivo: {0} (CONNECTION_FAILED)")]
    ConnectionFailed(String),

    #[error("Falha durante a transferência do arquivo: {0} (TRANSFER_FAILED)")]
    TransferFailed(String),

    #[error("Requisição inválida ou formato de mensagem inesperado: {0} (INVALID_REQUEST)")]
    InvalidRequest(String),

    #[error("Incompatibilidade de versão do protocolo. Esperado {expected}, recebido {received}. (PROTOCOL_VERSION_MISMATCH)")]
    ProtocolVersionMismatch { expected: u32, received: u32 },

    #[error("Erro de I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("Erro de serialização/JSON: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    Custom(String),
}

impl DuckerError {
    pub fn code(&self) -> ErrorCode {
        match self {
            DuckerError::DestinationIdMismatch => ErrorCode::DestinationIdMismatch,
            DuckerError::DeviceNotFound => ErrorCode::DeviceNotFound,
            DuckerError::ConnectionFailed(_) => ErrorCode::ConnectionFailed,
            DuckerError::TransferFailed(_) => ErrorCode::TransferFailed,
            DuckerError::InvalidRequest(_) => ErrorCode::InvalidRequest,
            DuckerError::ProtocolVersionMismatch { .. } => ErrorCode::ProtocolVersionMismatch,
            _ => ErrorCode::TransferFailed,
        }
    }

    pub fn user_friendly_message(&self) -> String {
        match self {
            DuckerError::DestinationIdMismatch => {
                "Ocorreu um erro: a transferência foi interceptada porque o ID Quac de destino não corresponde a este dispositivo.".to_string()
            }
            _ => self.to_string(),
        }
    }
}
