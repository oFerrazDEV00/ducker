//! Estado de sessões de recebimento (lado servidor) e utilitários de caminho.

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use tokio::sync::oneshot;

use crate::model::{DeviceInfo, FileDto};

pub(crate) struct SessionFile {
    pub dto: FileDto,
    pub token: String,
    pub done: bool,
}

pub(crate) struct ReceiveSession {
    pub id: String,
    #[allow(dead_code)]
    pub sender: DeviceInfo,
    pub sender_ip: IpAddr,
    pub files: HashMap<String, SessionFile>,
    pub cancelled: Arc<AtomicBool>,
}

/// Apenas UMA sessão de recebimento ativa por vez (semântica do LocalSend: 409 se ocupado).
#[derive(Default)]
pub(crate) struct SessionStore {
    pub active: Mutex<Option<ReceiveSession>>,
    /// Pedidos aguardando decisão do usuário: session_id -> canal de resposta.
    pub pending: Mutex<HashMap<String, oneshot::Sender<bool>>>,
}

/// Converte o `fileName` recebido (pode conter subpastas, ex: "fotos/a.jpg")
/// num caminho relativo seguro, removendo `..`, raízes e prefixos de drive.
pub fn sanitize_relative_path(name: &str) -> PathBuf {
    let normalized = name.replace('\\', "/");
    let mut out = PathBuf::new();
    for comp in Path::new(&normalized).components() {
        if let Component::Normal(part) = comp {
            let s = part.to_string_lossy();
            let cleaned: String = s
                .chars()
                .map(|c| if matches!(c, ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control() { '_' } else { c })
                .collect();
            let cleaned = cleaned.trim().trim_end_matches('.').to_string();
            if !cleaned.is_empty() {
                out.push(cleaned);
            }
        }
    }
    if out.as_os_str().is_empty() {
        out.push("arquivo_recebido");
    }
    out
}

/// Caminho que não sobrescreve arquivos existentes: "a.txt" -> "a (1).txt" -> "a (2).txt".
pub fn unique_path(dir: &Path, relative: &Path) -> PathBuf {
    let candidate = dir.join(relative);
    if !candidate.exists() {
        return candidate;
    }
    let parent = candidate.parent().map(Path::to_path_buf).unwrap_or_else(|| dir.to_path_buf());
    let stem = candidate.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let ext = candidate.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    for i in 1.. {
        let p = parent.join(format!("{stem} ({i}){ext}"));
        if !p.exists() {
            return p;
        }
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_traversal() {
        assert_eq!(sanitize_relative_path("../../etc/passwd"), PathBuf::from("etc").join("passwd"));
        #[cfg(windows)]
        assert_eq!(sanitize_relative_path("C:\\Windows\\x.dll"), PathBuf::from("Windows").join("x.dll"));
        #[cfg(not(windows))]
        assert_eq!(sanitize_relative_path("C:\\Windows\\x.dll"), PathBuf::from("C_").join("Windows").join("x.dll"));
        assert_eq!(sanitize_relative_path("fotos/a.jpg"), PathBuf::from("fotos").join("a.jpg"));
        assert_eq!(sanitize_relative_path(".."), PathBuf::from("arquivo_recebido"));
    }

    #[test]
    fn unique_names() {
        let dir = tempfile::tempdir().unwrap();
        let first = unique_path(dir.path(), Path::new("a.txt"));
        std::fs::write(&first, b"1").unwrap();
        let second = unique_path(dir.path(), Path::new("a.txt"));
        assert_eq!(second.file_name().unwrap(), "a (1).txt");
    }
}
