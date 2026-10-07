//! Identidade persistente do dispositivo: apelido, ID Quac, certificado TLS e fingerprint.
//! Salva em `<config_dir>/identity.json`. No desktop o padrão é `~/.ducker`; no mobile
//! o app Tauri passa o seu `app_data_dir`.

use std::path::{Path, PathBuf};

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::error::{DuckerError, Result};
use crate::tls;

pub const IDENTITY_FILE: &str = "identity.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Identity {
    /// Nome exibido para outros dispositivos (`alias` no LocalSend).
    /// `device_name` é aceito para compatibilidade com o identity.json antigo.
    #[serde(alias = "device_name")]
    pub alias: String,
    pub quac_id: u32,
    #[serde(default)]
    pub cert_pem: String,
    #[serde(default)]
    pub key_pem: String,
    #[serde(default)]
    pub fingerprint: String,
}

impl Identity {
    /// Cria uma identidade nova (gera ID Quac e certificado).
    pub fn generate(alias: impl Into<String>) -> Result<Self> {
        let mut id = Self {
            alias: alias.into(),
            quac_id: rand::thread_rng().gen_range(10_000_000..=99_999_999),
            cert_pem: String::new(),
            key_pem: String::new(),
            fingerprint: String::new(),
        };
        id.ensure_certificate()?;
        Ok(id)
    }

    /// Garante que exista certificado + fingerprint (migra identity.json antigo).
    /// Retorna `true` se algo foi gerado.
    pub fn ensure_certificate(&mut self) -> Result<bool> {
        if !self.cert_pem.is_empty() && !self.key_pem.is_empty() {
            let fp = tls::fingerprint_pem(&self.cert_pem)?;
            let changed = fp != self.fingerprint;
            self.fingerprint = fp;
            return Ok(changed);
        }
        let (cert, key) = tls::generate_self_signed()?;
        self.fingerprint = tls::fingerprint_pem(&cert)?;
        self.cert_pem = cert;
        self.key_pem = key;
        Ok(true)
    }

    pub fn default_config_dir() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| DuckerError::Custom("Não foi possível localizar o diretório do usuário.".into()))?;
        Ok(home.join(".ducker"))
    }

    pub fn load(dir: &Path) -> Result<Option<Self>> {
        let path = dir.join(IDENTITY_FILE);
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        // Arquivos salvos pelo PowerShell podem ter BOM UTF-8.
        let clean = content.strip_prefix('\u{feff}').unwrap_or(&content);
        let mut id: Self = serde_json::from_str(clean)?;
        if id.ensure_certificate()? {
            id.save(dir)?;
        }
        Ok(Some(id))
    }

    pub fn save(&self, dir: &Path) -> Result<()> {
        std::fs::create_dir_all(dir)?;
        std::fs::write(dir.join(IDENTITY_FILE), serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Carrega do diretório ou cria uma identidade nova com `default_alias`.
    pub fn load_or_create(dir: &Path, default_alias: &str) -> Result<Self> {
        if let Some(id) = Self::load(dir)? {
            return Ok(id);
        }
        let id = Self::generate(default_alias)?;
        id.save(dir)?;
        Ok(id)
    }
}

/// Apelido padrão sugerido: nome do computador ou "Ducker".
pub fn default_alias() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "Ducker".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_reload() {
        crate::tls::install_crypto_provider();
        let dir = tempfile::tempdir().unwrap();
        let a = Identity::load_or_create(dir.path(), "Teste").unwrap();
        assert!(a.quac_id >= 10_000_000 && a.quac_id <= 99_999_999);
        assert_eq!(a.fingerprint.len(), 64);
        let b = Identity::load_or_create(dir.path(), "Outro").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn migrates_legacy_file_with_bom() {
        crate::tls::install_crypto_provider();
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(IDENTITY_FILE),
            "\u{feff}{\"device_name\": \"Notebook do Fael\", \"quac_id\": 84726193}",
        )
        .unwrap();
        let id = Identity::load(dir.path()).unwrap().unwrap();
        assert_eq!(id.alias, "Notebook do Fael");
        assert_eq!(id.quac_id, 84726193);
        assert!(!id.cert_pem.is_empty());
    }
}
