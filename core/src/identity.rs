use std::path::{Path, PathBuf};
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::error::DuckerError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceIdentity {
    pub device_name: String,
    pub quac_id: u32,
}

impl DeviceIdentity {
    pub fn new(device_name: impl Into<String>) -> Self {
        let mut rng = rand::thread_rng();
        let quac_id: u32 = rng.gen_range(10_000_000..=99_999_999);
        Self {
            device_name: device_name.into(),
            quac_id,
        }
    }

    pub fn with_id(device_name: impl Into<String>, quac_id: u32) -> Self {
        Self {
            device_name: device_name.into(),
            quac_id,
        }
    }

    pub fn default_config_path() -> Result<PathBuf, DuckerError> {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            DuckerError::Custom("Não foi possível localizar o diretório do usuário.".to_string())
        })?;
        let config_dir = home_dir.join(".ducker");
        std::fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join("identity.json"))
    }

    pub fn load_from_path(path: &Path) -> Result<Option<Self>, DuckerError> {
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(path)?;
        let clean_content = content.strip_prefix('\u{feff}').unwrap_or(&content);
        let identity: Self = serde_json::from_str(clean_content)?;
        Ok(Some(identity))
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), DuckerError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn load_default() -> Result<Option<Self>, DuckerError> {
        let path = Self::default_config_path()?;
        Self::load_from_path(&path)
    }

    pub fn save_default(&self) -> Result<(), DuckerError> {
        let path = Self::default_config_path()?;
        self.save_to_path(&path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quac_id_range() {
        let identity = DeviceIdentity::new("Teste");
        assert!(identity.quac_id >= 10_000_000 && identity.quac_id <= 99_999_999);
        assert_eq!(identity.device_name, "Teste");
    }

    #[test]
    fn test_identity_serialization() {
        let identity = DeviceIdentity::with_id("Notebook do Fael", 84726193);
        let serialized = serde_json::to_string(&identity).unwrap();
        let deserialized: DeviceIdentity = serde_json::from_str(&serialized).unwrap();
        assert_eq!(identity, deserialized);
    }

    #[test]
    fn test_identity_file_roundtrip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("identity.json");

        let identity = DeviceIdentity::new("Celular do Fael");
        identity.save_to_path(&path).unwrap();

        let loaded = DeviceIdentity::load_from_path(&path).unwrap().unwrap();
        assert_eq!(identity, loaded);
    }

    #[test]
    fn test_identity_with_utf8_bom() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("identity_bom.json");

        // Simula arquivo salvo pelo Windows PowerShell com BOM
        let content = "\u{feff}{\"device_name\": \"Notebook do Fael\", \"quac_id\": 84726193}";
        std::fs::write(&path, content).unwrap();

        let loaded = DeviceIdentity::load_from_path(&path).unwrap().unwrap();
        assert_eq!(loaded.device_name, "Notebook do Fael");
        assert_eq!(loaded.quac_id, 84726193);
    }
}
