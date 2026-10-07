//! DTOs do protocolo LocalSend v2 (ver `ducker/LOCALSEND_PROTOCOL.md`).
//! Todos os campos usam camelCase no JSON. Campos desconhecidos são ignorados pelo serde,
//! o que garante compatibilidade com versões futuras do LocalSend.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: &str = "2.1";
pub const DEFAULT_PORT: u16 = 53317;
pub const DEFAULT_MULTICAST_ADDR: std::net::Ipv4Addr = std::net::Ipv4Addr::new(224, 0, 0, 167);
pub const API_PREFIX: &str = "/api/localsend/v2";

fn default_version() -> String {
    PROTOCOL_VERSION.to_string()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Mobile,
    #[default]
    Desktop,
    Web,
    Headless,
    Server,
    /// Valor desconhecido recebido de outra implementação (tratado como desktop na UI).
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Http,
    #[default]
    Https,
}

impl Protocol {
    pub fn scheme(&self) -> &'static str {
        match self {
            Protocol::Http => "http",
            Protocol::Https => "https",
        }
    }
}

/// Informações de um dispositivo. Usado em `register`, `info`, `prepare-upload.info`
/// e (achatado) no anúncio multicast.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub alias: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub device_model: Option<String>,
    #[serde(default)]
    pub device_type: Option<DeviceType>,
    #[serde(default)]
    pub fingerprint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<Protocol>,
    #[serde(default)]
    pub download: bool,
    /// Extensão Ducker: ID Quac de 8 dígitos. Ignorado por clientes LocalSend oficiais.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quac_id: Option<u32>,
}

/// Mensagem UDP multicast (anúncio ou resposta).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MulticastDto {
    #[serde(flatten)]
    pub info: DeviceInfo,
    /// v2
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub announce: Option<bool>,
    /// v1 (legado, enviado junto por compatibilidade)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub announcement: Option<bool>,
}

impl MulticastDto {
    pub fn new(info: DeviceInfo, announce: bool) -> Self {
        Self { info, announce: Some(announce), announcement: Some(announce) }
    }

    pub fn is_announce(&self) -> bool {
        self.announce.or(self.announcement).unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accessed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileDto {
    pub id: String,
    pub file_name: String,
    pub size: u64,
    pub file_type: String,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub preview: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<FileMetadata>,
}

impl FileDto {
    /// Mensagem de texto pura (LocalSend envia texto como arquivo text/plain com `preview`).
    pub fn is_text_message(&self) -> bool {
        self.file_type.starts_with("text/") && self.preview.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareUploadRequest {
    pub info: DeviceInfo,
    pub files: HashMap<String, FileDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareUploadResponse {
    pub session_id: String,
    /// fileId -> token
    pub files: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_official_announcement() {
        let json = r#"{"alias":"Nice Orange","version":"2.0","deviceModel":"Samsung","deviceType":"mobile",
            "fingerprint":"abc","port":53317,"protocol":"https","download":true,"announce":true}"#;
        let dto: MulticastDto = serde_json::from_str(json).unwrap();
        assert_eq!(dto.info.alias, "Nice Orange");
        assert_eq!(dto.info.device_type, Some(DeviceType::Mobile));
        assert_eq!(dto.info.protocol, Some(Protocol::Https));
        assert!(dto.is_announce());
        assert_eq!(dto.info.quac_id, None);
    }

    #[test]
    fn unknown_device_type_and_missing_fields() {
        let json = r#"{"alias":"X","deviceType":"toaster","fingerprint":"f"}"#;
        let info: DeviceInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.device_type, Some(DeviceType::Other));
        assert_eq!(info.version, PROTOCOL_VERSION);
        assert!(!info.download);
    }

    #[test]
    fn serializes_camel_case_with_quac() {
        let info = DeviceInfo {
            alias: "PC".into(),
            version: PROTOCOL_VERSION.into(),
            device_model: Some("Windows".into()),
            device_type: Some(DeviceType::Desktop),
            fingerprint: "F".into(),
            port: Some(53317),
            protocol: Some(Protocol::Https),
            download: false,
            quac_id: Some(12345678),
        };
        let v = serde_json::to_value(MulticastDto::new(info, true)).unwrap();
        assert_eq!(v["deviceModel"], "Windows");
        assert_eq!(v["quacId"], 12345678);
        assert_eq!(v["announce"], true);
        assert_eq!(v["announcement"], true);
    }
}
