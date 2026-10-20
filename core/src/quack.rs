//! Formato .quack — Automação Declarativa de Envio de Arquivos
//!
//! Especificação Versão 1.
//! Permite descrever tarefas de envio de arquivos e pastas de forma legível,
//! reprodutível, validável e segura, sem execução de código arbitrário.

use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{DuckerError, Result};
use crate::model::Protocol;
use crate::node::Peer;

/// Estrutura raiz de um arquivo `.quack` de automação.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuackFile {
    /// Versão do esquema de automação (atualmente "1").
    pub schema: String,
    /// Nome da tarefa de automação.
    pub name: String,
    /// Descrição opcional do objetivo da tarefa.
    #[serde(default)]
    pub description: Option<String>,
    /// Destinatário da automação.
    pub target: QuackTarget,
    /// Origens dos arquivos ou pastas a serem enviados.
    pub sources: Vec<QuackSource>,
    /// Opções de execução e limites de segurança.
    #[serde(default)]
    pub options: QuackOptions,
}

/// Identificação do destinatário no arquivo `.quack`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct QuackTarget {
    #[serde(default)]
    pub alias: Option<String>,
    #[serde(default)]
    pub quac_id: Option<String>,
    #[serde(default)]
    pub fingerprint: Option<String>,
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    53317
}

/// Declaração de origem de arquivos no arquivo `.quack`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuackSource {
    /// Caminho do arquivo ou pasta (relativo ao arquivo .quack ou absoluto).
    pub path: String,
    /// Padrões de inclusão glob (ex: ["*.pdf", "*.xlsx"]).
    #[serde(default)]
    pub include: Vec<String>,
    /// Padrões de exclusão glob (ex: ["*.tmp", "~*", ".git/*"]).
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Se verdadeiro, varre subpastas recursivamente.
    #[serde(default = "default_true")]
    pub recursive: bool,
}

fn default_true() -> bool {
    true
}

/// Opções de execução de uma automação `.quack`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuackOptions {
    /// Se verdadeiro, simula a execução sem transferir bytes na rede.
    #[serde(default)]
    pub dry_run: bool,
    /// Política de conflito: "rename" | "skip" | "overwrite".
    #[serde(default = "default_conflict")]
    pub conflict: String,
    /// Limite máximo de bytes permitidos para envio (padrão: 1 GB).
    #[serde(default = "default_max_bytes")]
    pub max_total_bytes: u64,
    /// Quantidade máxima de arquivos permitidos para envio (padrão: 1000).
    #[serde(default = "default_max_files")]
    pub max_file_count: usize,
    /// Se verdadeiro, exige confirmação interativa antes de iniciar a transferência.
    #[serde(default)]
    pub require_confirmation: bool,
}

impl Default for QuackOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            conflict: default_conflict(),
            max_total_bytes: default_max_bytes(),
            max_file_count: default_max_files(),
            require_confirmation: false,
        }
    }
}

fn default_conflict() -> String {
    "rename".to_string()
}

fn default_max_bytes() -> u64 {
    1_073_741_824 // 1 GB
}

fn default_max_files() -> usize {
    1000
}

/// Arquivo individual resolvido e pronto para transferência.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolvedQuackFile {
    pub source_path: PathBuf,
    pub relative_dest_name: String,
    pub size_bytes: u64,
}

/// Plano de execução consolidado de uma tarefa `.quack`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuackPlan {
    pub name: String,
    pub target: QuackTarget,
    pub files: Vec<ResolvedQuackFile>,
    pub total_bytes: u64,
    pub file_count: usize,
    pub dry_run: bool,
    pub conflict: String,
}

impl QuackFile {
    /// Analisa uma string contendo a definição `.quack` em YAML ou JSON.
    pub fn parse_str(content: &str) -> Result<Self> {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Err(DuckerError::Quack(
                "O arquivo .quack está vazio.".to_string(),
            ));
        }

        // Tenta interpretar como YAML primeiro, com fallback para JSON
        let parsed: QuackFile = match serde_yaml::from_str(trimmed) {
            Ok(val) => val,
            Err(yaml_err) => match serde_json::from_str(trimmed) {
                Ok(val) => val,
                Err(_) => {
                    return Err(DuckerError::Quack(format!(
                        "Sintaxe inválida no arquivo .quack: {}",
                        yaml_err
                    )));
                }
            },
        };

        // Validação da versão do esquema
        if parsed.schema.trim() != "1" {
            return Err(DuckerError::Quack(format!(
                "Versão de esquema .quack '{}' não suportada. Esta versão do Ducker suporta schema: '1'.",
                parsed.schema
            )));
        }

        // Validação de destinatário
        if parsed.target.alias.is_none()
            && parsed.target.quac_id.is_none()
            && parsed.target.fingerprint.is_none()
            && parsed.target.ip.is_none()
        {
            return Err(DuckerError::Quack(
                "O destinatário (target) deve especificar ao menos um dos campos: alias, quac_id, fingerprint ou ip."
                    .to_string(),
            ));
        }

        // Validação de origens
        if parsed.sources.is_empty() {
            return Err(DuckerError::Quack(
                "A automação deve conter ao menos uma fonte em 'sources'.".to_string(),
            ));
        }

        Ok(parsed)
    }

    /// Carrega e analisa um arquivo `.quack` a partir do disco com limite de segurança de tamanho.
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let metadata = std::fs::metadata(path).map_err(|e| {
            DuckerError::Quack(format!(
                "Não foi possível acessar '{}': {}",
                path.display(),
                e
            ))
        })?;

        if metadata.len() > 2 * 1024 * 1024 {
            return Err(DuckerError::Quack(
                "O arquivo .quack excede o limite máximo permitido de 2 MB.".to_string(),
            ));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| DuckerError::Quack(format!("Erro ao ler '{}': {}", path.display(), e)))?;

        Self::parse_str(&content)
    }

    /// Resolve o plano de envio inspecionando o sistema de arquivos a partir de um diretório base.
    /// Aplica proteções estritas contra path traversal e valida limites configurados.
    pub fn resolve_plan(&self, base_dir: &Path) -> Result<QuackPlan> {
        let mut resolved_files = Vec::new();
        let mut total_bytes: u64 = 0;

        for src in &self.sources {
            let raw_path = src.path.trim();
            if raw_path.is_empty() {
                continue;
            }

            // Proteção contra path traversal: não permitir ".." saindo do escopo
            if raw_path.contains("..") {
                let comp_has_parent = Path::new(raw_path)
                    .components()
                    .any(|c| c == Component::ParentDir);
                if comp_has_parent {
                    return Err(DuckerError::Quack(format!(
                        "Caminho inseguro detectado em sources ('{}'). O uso de '..' não é permitido.",
                        raw_path
                    )));
                }
            }

            let source_path = if Path::new(raw_path).is_absolute() {
                PathBuf::from(raw_path)
            } else {
                base_dir.join(raw_path)
            };

            if !source_path.exists() {
                return Err(DuckerError::Quack(format!(
                    "Caminho de origem não encontrado no disco: '{}'",
                    source_path.display()
                )));
            }

            if source_path.is_file() {
                let meta = std::fs::metadata(&source_path).map_err(DuckerError::Io)?;
                let file_name = source_path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("arquivo")
                    .to_string();

                if is_allowed_by_filters(&file_name, &src.include, &src.exclude) {
                    total_bytes += meta.len();
                    resolved_files.push(ResolvedQuackFile {
                        source_path: source_path.clone(),
                        relative_dest_name: file_name,
                        size_bytes: meta.len(),
                    });
                }
            } else if source_path.is_dir() {
                let base_root = source_path.clone();
                collect_dir_files(
                    &source_path,
                    &base_root,
                    src.recursive,
                    &src.include,
                    &src.exclude,
                    &mut resolved_files,
                    &mut total_bytes,
                )?;
            }
        }

        // Valida limites de segurança
        if resolved_files.len() > self.options.max_file_count {
            return Err(DuckerError::Quack(format!(
                "A automação encontrou {} arquivos, excedendo o limite configurado de {}.",
                resolved_files.len(),
                self.options.max_file_count
            )));
        }

        if total_bytes > self.options.max_total_bytes {
            let mb_total = (total_bytes as f64) / 1_048_576.0;
            let mb_max = (self.options.max_total_bytes as f64) / 1_048_576.0;
            return Err(DuckerError::Quack(format!(
                "Tamanho total dos arquivos ({:.2} MB) excede o limite configurado ({:.2} MB).",
                mb_total, mb_max
            )));
        }

        Ok(QuackPlan {
            name: self.name.clone(),
            target: self.target.clone(),
            file_count: resolved_files.len(),
            files: resolved_files,
            total_bytes,
            dry_run: self.options.dry_run,
            conflict: self.options.conflict.clone(),
        })
    }
}

impl QuackTarget {
    /// Localiza o peer correspondente na lista de dispositivos descobertos pelo Ducker.
    pub fn resolve_peer<'a>(&self, peers: &'a [Peer]) -> Option<&'a Peer> {
        // 1. Busca por Quac ID
        if let Some(ref qid_str) = self.quac_id {
            if let Ok(qid) = qid_str.trim().parse::<u32>() {
                if let Some(found) = peers.iter().find(|p| p.info.quac_id == Some(qid)) {
                    return Some(found);
                }
            }
        }

        // 2. Busca por Fingerprint SHA-256
        if let Some(ref fp) = self.fingerprint {
            let clean_fp = fp.trim();
            if let Some(found) = peers.iter().find(|p| {
                p.info.fingerprint.trim().eq_ignore_ascii_case(clean_fp)
                    || p.info.fingerprint.starts_with(clean_fp)
            }) {
                return Some(found);
            }
        }

        // 3. Busca por Apelido (Alias)
        if let Some(ref alias) = self.alias {
            let clean_alias = alias.trim().to_lowercase();
            if let Some(found) = peers
                .iter()
                .find(|p| p.info.alias.trim().to_lowercase() == clean_alias)
            {
                return Some(found);
            }
        }

        // 4. Busca por IP
        if let Some(ref ip) = self.ip {
            if let Some(found) = peers.iter().find(|p| p.ip.to_string() == ip.trim()) {
                return Some(found);
            }
        }

        None
    }

    /// Cria um Peer sintético diretamente a partir de IP e porta se especificados.
    pub fn to_synthetic_peer(&self) -> Option<Peer> {
        let ip_str = self.ip.as_ref()?;
        let ip: std::net::IpAddr = ip_str.trim().parse().ok()?;
        let alias = self
            .alias
            .clone()
            .unwrap_or_else(|| format!("Dispositivo {}", ip));
        let quac_id = self
            .quac_id
            .as_ref()
            .and_then(|q| q.trim().parse::<u32>().ok());
        let fingerprint = self.fingerprint.clone().unwrap_or_default();

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Some(Peer {
            key: format!("{}:{}", ip, self.port),
            ip,
            port: self.port,
            protocol: Protocol::Https,
            info: crate::model::DeviceInfo {
                alias,
                version: crate::model::PROTOCOL_VERSION.to_string(),
                device_model: Some("Peer Quack".to_string()),
                device_type: Some(crate::model::DeviceType::Desktop),
                fingerprint,
                port: Some(self.port),
                protocol: Some(Protocol::Https),
                download: true,
                quac_id,
            },
            last_seen_ms: now_ms,
        })
    }
}

fn collect_dir_files(
    current_dir: &Path,
    base_root: &Path,
    recursive: bool,
    include: &[String],
    exclude: &[String],
    out_files: &mut Vec<ResolvedQuackFile>,
    total_bytes: &mut u64,
) -> Result<()> {
    let entries = match std::fs::read_dir(current_dir) {
        Ok(e) => e,
        Err(e) => return Err(DuckerError::Io(e)),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if path.is_file() {
            if is_allowed_by_filters(&name, include, exclude) {
                let meta = std::fs::metadata(&path).map_err(DuckerError::Io)?;
                let rel = path
                    .strip_prefix(base_root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");

                *total_bytes += meta.len();
                out_files.push(ResolvedQuackFile {
                    source_path: path,
                    relative_dest_name: rel,
                    size_bytes: meta.len(),
                });
            }
        } else if path.is_dir() && recursive {
            collect_dir_files(
                &path,
                base_root,
                recursive,
                include,
                exclude,
                out_files,
                total_bytes,
            )?;
        }
    }

    Ok(())
}

fn is_allowed_by_filters(name: &str, include: &[String], exclude: &[String]) -> bool {
    // 1. Checa exclusões primeiro
    for pattern in exclude {
        if matches_pattern(name, pattern) {
            return false;
        }
    }

    // 2. Se houver inclusões específicas, deve atender a pelo menos uma
    if !include.is_empty() {
        return include.iter().any(|pattern| matches_pattern(name, pattern));
    }

    true
}

fn matches_pattern(name: &str, pattern: &str) -> bool {
    let pat = pattern.trim().to_lowercase();
    let n = name.to_lowercase();

    if pat == "*" {
        return true;
    }

    if pat.starts_with("*.") {
        let ext = &pat[1..]; // ".pdf"
        return n.ends_with(ext);
    }

    if pat.ends_with('*') {
        let prefix = &pat[..pat.len() - 1];
        return n.starts_with(prefix);
    }

    n == pat
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_yaml_quack() {
        let yaml = r#"
schema: "1"
name: "Backup Documentos"
description: "Envio de relatórios semanais"
target:
  alias: "Notebook Trabalho"
  quac_id: "12345678"
sources:
  - path: "."
    include: ["*.txt"]
options:
  dry_run: true
  conflict: "skip"
  max_total_bytes: 10485760
  max_file_count: 50
"#;
        let quack = QuackFile::parse_str(yaml).unwrap();
        assert_eq!(quack.schema, "1");
        assert_eq!(quack.name, "Backup Documentos");
        assert_eq!(quack.target.alias.as_deref(), Some("Notebook Trabalho"));
        assert_eq!(quack.target.quac_id.as_deref(), Some("12345678"));
        assert!(quack.options.dry_run);
        assert_eq!(quack.options.conflict, "skip");
    }

    #[test]
    fn test_rejects_unsupported_schema() {
        let yaml = r#"
schema: "99"
name: "Teste Inválido"
target:
  alias: "Outro"
sources:
  - path: "."
"#;
        let err = QuackFile::parse_str(yaml).unwrap_err();
        assert!(err.to_string().contains("não suportada"));
    }

    #[test]
    fn test_rejects_traversal_path() {
        let yaml = r#"
schema: "1"
name: "Traversal Test"
target:
  alias: "Outro"
sources:
  - path: "../../etc/passwd"
"#;
        let quack = QuackFile::parse_str(yaml).unwrap();
        let temp_dir = std::env::temp_dir();
        let err = quack.resolve_plan(&temp_dir).unwrap_err();
        assert!(err.to_string().contains("Caminho inseguro"));
    }

    #[test]
    fn test_resolves_plan_with_filters() {
        let temp_dir = std::env::temp_dir().join(format!("quack-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let file_a = temp_dir.join("relatorio.pdf");
        let file_b = temp_dir.join("temp.tmp");
        std::fs::write(&file_a, b"PDF CONTENT").unwrap();
        std::fs::write(&file_b, b"TMP CONTENT").unwrap();

        let yaml = format!(
            r#"
schema: "1"
name: "Teste Filtros"
target:
  alias: "Meu PC"
sources:
  - path: "{}"
    include: ["*.pdf"]
    exclude: ["*.tmp"]
options:
  dry_run: true
"#,
            temp_dir.display().to_string().replace('\\', "/")
        );

        let quack = QuackFile::parse_str(&yaml).unwrap();
        let plan = quack.resolve_plan(&temp_dir).unwrap();
        assert_eq!(plan.file_count, 1);
        assert_eq!(plan.files[0].relative_dest_name, "relatorio.pdf");
        assert_eq!(plan.total_bytes, 11);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
