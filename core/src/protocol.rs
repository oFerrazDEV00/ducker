use serde::{Deserialize, Serialize};
use crate::error::{DuckerError, ErrorCode};

pub const CURRENT_PROTOCOL_VERSION: u32 = 1;
pub const DEFAULT_TRANSFER_PORT: u16 = 7878;
pub const DEFAULT_DISCOVERY_PORT: u16 = 7879;

/// Mensagem de anúncio de descoberta transmitida na rede local
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveryAnnouncement {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub quac_id: u32,
    pub device_name: String,
    pub address: String,
    pub port: u16,
    pub protocol_version: u32,
}

impl DiscoveryAnnouncement {
    pub const MSG_TYPE: &'static str = "ducker_discovery";

    pub fn new(quac_id: u32, device_name: impl Into<String>, address: impl Into<String>, port: u16) -> Self {
        Self {
            msg_type: Self::MSG_TYPE.to_string(),
            quac_id,
            device_name: device_name.into(),
            address: address.into(),
            port,
            protocol_version: CURRENT_PROTOCOL_VERSION,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.msg_type == Self::MSG_TYPE && self.protocol_version == CURRENT_PROTOCOL_VERSION
    }
}

/// Solicitação de início de transferência com validação do ID Quac de destino
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferHandshakeRequest {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub protocol_version: u32,
    pub sender_name: String,
    pub sender_quac_id: u32,
    pub destination_quac_id: u32,
    pub file_name: String,
    pub file_size: u64,
}

impl TransferHandshakeRequest {
    pub const MSG_TYPE: &'static str = "transfer_handshake";

    pub fn new(
        sender_name: impl Into<String>,
        sender_quac_id: u32,
        destination_quac_id: u32,
        file_name: impl Into<String>,
        file_size: u64,
    ) -> Self {
        Self {
            msg_type: Self::MSG_TYPE.to_string(),
            protocol_version: CURRENT_PROTOCOL_VERSION,
            sender_name: sender_name.into(),
            sender_quac_id,
            destination_quac_id,
            file_name: file_name.into(),
            file_size,
        }
    }

    /// Validação estrita do ID Quac de destino
    pub fn validate_destination(&self, own_quac_id: u32) -> Result<(), DuckerError> {
        if self.protocol_version != CURRENT_PROTOCOL_VERSION {
            return Err(DuckerError::ProtocolVersionMismatch {
                expected: CURRENT_PROTOCOL_VERSION,
                received: self.protocol_version,
            });
        }

        if self.destination_quac_id != own_quac_id {
            return Err(DuckerError::DestinationIdMismatch);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HandshakeStatus {
    Accepted,
    Rejected,
}

/// Resposta do receptor para a solicitação de handshake
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferHandshakeResponse {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub status: HandshakeStatus,
    pub reason: Option<ErrorCode>,
    pub message: Option<String>,
}

impl TransferHandshakeResponse {
    pub const MSG_TYPE: &'static str = "handshake_response";

    pub fn accepted() -> Self {
        Self {
            msg_type: Self::MSG_TYPE.to_string(),
            status: HandshakeStatus::Accepted,
            reason: None,
            message: None,
        }
    }

    pub fn rejected(error: &DuckerError) -> Self {
        Self {
            msg_type: Self::MSG_TYPE.to_string(),
            status: HandshakeStatus::Rejected,
            reason: Some(error.code()),
            message: Some(error.user_friendly_message()),
        }
    }
}

/// Confirmação final da transferência
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferResult {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub success: bool,
    pub error_code: Option<ErrorCode>,
    pub message: String,
}

impl TransferResult {
    pub const MSG_TYPE: &'static str = "transfer_result";

    pub fn ok() -> Self {
        Self {
            msg_type: Self::MSG_TYPE.to_string(),
            success: true,
            error_code: None,
            message: "Transferência concluída com sucesso.".to_string(),
        }
    }

    pub fn failed(error: &DuckerError) -> Self {
        Self {
            msg_type: Self::MSG_TYPE.to_string(),
            success: false,
            error_code: Some(error.code()),
            message: error.user_friendly_message(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_destination_validation_success() {
        let req = TransferHandshakeRequest::new("Remetente", 11112222, 88889999, "foto.png", 1024);
        assert!(req.validate_destination(88889999).is_ok());
    }

    #[test]
    fn test_destination_validation_mismatch() {
        let req = TransferHandshakeRequest::new("Remetente", 11112222, 12345678, "foto.png", 1024);
        let result = req.validate_destination(88889999);
        assert!(result.is_err());
        match result.unwrap_err() {
            DuckerError::DestinationIdMismatch => (),
            other => panic!("Esperado DestinationIdMismatch, recebido: {:?}", other),
        }
    }

    #[test]
    fn test_protocol_version_mismatch() {
        let mut req = TransferHandshakeRequest::new("Remetente", 11112222, 88889999, "foto.png", 1024);
        req.protocol_version = 999;
        assert!(matches!(
            req.validate_destination(88889999),
            Err(DuckerError::ProtocolVersionMismatch { expected: 1, received: 999 })
        ));
    }
}
