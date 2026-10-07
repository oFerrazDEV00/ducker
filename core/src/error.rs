use thiserror::Error;

/// Erros do Ducker. As mensagens são em português pois são exibidas ao usuário final.
#[derive(Error, Debug)]
pub enum DuckerError {
    #[error("Ocorreu um erro: a transferência foi interceptada porque o ID Quac de destino não corresponde a este dispositivo. (DESTINATION_ID_MISMATCH)")]
    DestinationIdMismatch,

    #[error("Dispositivo não encontrado ou indisponível na rede. (DEVICE_NOT_FOUND)")]
    DeviceNotFound,

    #[error("Não foi possível estabelecer conexão com o dispositivo: {0} (CONNECTION_FAILED)")]
    ConnectionFailed(String),

    #[error("O destinatário recusou a transferência. (REJECTED)")]
    Rejected,

    #[error("PIN obrigatório ou inválido. (PIN_REQUIRED)")]
    PinRequired,

    #[error("O destinatário está ocupado com outra transferência. (BLOCKED)")]
    Busy,

    #[error("Falha na verificação de integridade (SHA-256) do arquivo. (CHECKSUM_MISMATCH)")]
    ChecksumMismatch,

    #[error("A impressão digital (fingerprint) do certificado não confere. (FINGERPRINT_MISMATCH)")]
    FingerprintMismatch,

    #[error("Falha durante a transferência do arquivo: {0} (TRANSFER_FAILED)")]
    TransferFailed(String),

    #[error("Requisição inválida: {0} (INVALID_REQUEST)")]
    InvalidRequest(String),

    #[error("Erro de TLS/certificado: {0}")]
    Tls(String),

    #[error("Erro de I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("Erro de serialização/JSON: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Erro HTTP: {0}")]
    Http(#[from] reqwest::Error),

    #[error("{0}")]
    Custom(String),
}

impl DuckerError {
    /// Código estável (para UI / logs / testes).
    pub fn code(&self) -> &'static str {
        match self {
            DuckerError::DestinationIdMismatch => "DESTINATION_ID_MISMATCH",
            DuckerError::DeviceNotFound => "DEVICE_NOT_FOUND",
            DuckerError::ConnectionFailed(_) => "CONNECTION_FAILED",
            DuckerError::Rejected => "REJECTED",
            DuckerError::PinRequired => "PIN_REQUIRED",
            DuckerError::Busy => "BLOCKED",
            DuckerError::ChecksumMismatch => "CHECKSUM_MISMATCH",
            DuckerError::FingerprintMismatch => "FINGERPRINT_MISMATCH",
            DuckerError::TransferFailed(_) => "TRANSFER_FAILED",
            DuckerError::InvalidRequest(_) => "INVALID_REQUEST",
            DuckerError::Tls(_) => "TLS_ERROR",
            DuckerError::Io(_) => "IO_ERROR",
            DuckerError::Serialization(_) => "SERIALIZATION_ERROR",
            DuckerError::Http(_) => "HTTP_ERROR",
            DuckerError::Custom(_) => "ERROR",
        }
    }
}

pub type Result<T> = std::result::Result<T, DuckerError>;
