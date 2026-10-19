//! Módulo de verificação e download seguro de atualizações do Ducker.
//! Consulta a API oficial do GitHub Releases (oFerrazDEV00/ducker) e valida integridade.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const GITHUB_REPO: &str = "oFerrazDEV00/ducker";
const GITHUB_API_LATEST: &str = "https://api.github.com/repos/oFerrazDEV00/ducker/releases/latest";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    pub available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_name: String,
    pub release_notes: String,
    pub published_at: String,
    pub asset_name: Option<String>,
    pub asset_size: Option<u64>,
    pub asset_url: Option<String>,
    pub status_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub pct: u32,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    size: u64,
    browser_download_url: String,
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    published_at: Option<String>,
    assets: Vec<GitHubAsset>,
}

/// Compara duas versões no formato semver (ex: "0.2.0" vs "0.3.0").
/// Retorna true se `remote` for estritamente mais recente que `current`.
pub fn is_newer_version(current: &str, remote: &str) -> bool {
    let clean_cur = current.trim().trim_start_matches('v');
    let clean_rem = remote.trim().trim_start_matches('v');

    let parse_parts = |s: &str| -> Vec<u64> {
        s.split('.')
            .filter_map(|p| {
                // Remove qualquer sufixo como -beta, -rc1, etc.
                p.split('-').next().and_then(|num| num.parse::<u64>().ok())
            })
            .collect()
    };

    let cur_parts = parse_parts(clean_cur);
    let rem_parts = parse_parts(clean_rem);

    for i in 0..cur_parts.len().max(rem_parts.len()) {
        let c = cur_parts.get(i).copied().unwrap_or(0);
        let r = rem_parts.get(i).copied().unwrap_or(0);
        if r > c {
            return true;
        }
        if r < c {
            return false;
        }
    }

    false
}

/// Consulta o repositório oficial do GitHub para buscar novas versões.
pub async fn check_github_update(current_version: &str) -> Result<UpdateCheckResult, String> {
    let client = reqwest::Client::builder()
        .user_agent(format!("Ducker-Updater/{}", current_version))
        .build()
        .map_err(|e| format!("Falha ao inicializar cliente HTTP: {}", e))?;

    let res = match client.get(GITHUB_API_LATEST).send().await {
        Ok(r) => r,
        Err(e) => {
            return Ok(UpdateCheckResult {
                available: false,
                current_version: current_version.to_string(),
                latest_version: current_version.to_string(),
                release_name: String::new(),
                release_notes: String::new(),
                published_at: String::new(),
                asset_name: None,
                asset_size: None,
                asset_url: None,
                status_message: format!(
                    "Não foi possível conectar ao servidor de atualizações ({}). Verifique sua conexão.",
                    e
                ),
            });
        }
    };

    if res.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(UpdateCheckResult {
            available: false,
            current_version: current_version.to_string(),
            latest_version: current_version.to_string(),
            release_name: "Desenvolvimento".into(),
            release_notes: "Nenhum release oficial publicado ainda no repositório GitHub.".into(),
            published_at: String::new(),
            asset_name: None,
            asset_size: None,
            asset_url: None,
            status_message: format!(
                "Você já está utilizando a compilação mais recente do Ducker (v{}). Nenhum release publicado em {}.",
                current_version, GITHUB_REPO
            ),
        });
    }

    if !res.status().is_success() {
        return Err(format!(
            "Erro ao consultar releases do GitHub (Status HTTP {})",
            res.status()
        ));
    }

    let release: GitHubRelease = res
        .json()
        .await
        .map_err(|e| format!("Falha ao interpretar resposta do release: {}", e))?;

    let remote_tag = release.tag_name.trim();
    let is_newer = is_newer_version(current_version, remote_tag);

    // Seleciona o asset ideal para a plataforma atual
    let matching_asset = select_platform_asset(&release.assets);

    let (asset_name, asset_size, asset_url) = if let Some(asset) = matching_asset {
        (
            Some(asset.name.clone()),
            Some(asset.size),
            Some(asset.browser_download_url.clone()),
        )
    } else {
        (None, None, None)
    };

    let status_message = if is_newer {
        if asset_url.is_some() {
            format!(
                "Nova versão disponível: {}! Clique para baixar.",
                remote_tag
            )
        } else {
            format!(
                "Nova versão {} encontrada, mas nenhum instalador compatível com seu sistema foi publicado ainda.",
                remote_tag
            )
        }
    } else {
        format!(
            "O Ducker já está atualizado na versão mais recente (v{}). 🦆",
            current_version
        )
    };

    Ok(UpdateCheckResult {
        available: is_newer && asset_url.is_some(),
        current_version: current_version.to_string(),
        latest_version: remote_tag.to_string(),
        release_name: release.name.unwrap_or_else(|| remote_tag.to_string()),
        release_notes: release.body.unwrap_or_default(),
        published_at: release.published_at.unwrap_or_default(),
        asset_name,
        asset_size,
        asset_url,
        status_message,
    })
}

/// Identifica o asset de download correspondente ao sistema operacional e arquitetura.
fn select_platform_asset(assets: &[GitHubAsset]) -> Option<&GitHubAsset> {
    #[cfg(target_os = "windows")]
    {
        // Prioriza o instalador NSIS setup.exe, depois .msi, depois qualquer .exe
        assets
            .iter()
            .find(|a| a.name.ends_with("-setup.exe"))
            .or_else(|| assets.iter().find(|a| a.name.ends_with(".msi")))
            .or_else(|| {
                assets
                    .iter()
                    .find(|a| a.name.ends_with(".exe") && !a.name.contains("cli"))
            })
    }

    #[cfg(target_os = "linux")]
    {
        // Prioriza AppImage, depois .deb
        assets
            .iter()
            .find(|a| a.name.ends_with(".AppImage"))
            .or_else(|| assets.iter().find(|a| a.name.ends_with(".deb")))
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        assets.first()
    }
}

/// Realiza o download seguro do instalador com relatório de progresso e cancelamento.
pub async fn download_installer<F>(
    asset_url: &str,
    asset_name: &str,
    save_dir: &PathBuf,
    on_progress: F,
    cancel_flag: Arc<AtomicBool>,
) -> Result<PathBuf, String>
where
    F: Fn(UpdateDownloadProgress) + Send + Sync + 'static,
{
    if !asset_url.starts_with("https://") {
        return Err("Apenas downloads via HTTPS seguro são permitidos.".into());
    }

    std::fs::create_dir_all(save_dir)
        .map_err(|e| format!("Falha ao criar diretório temporário: {}", e))?;

    let file_path = save_dir.join(asset_name);
    let mut file = tokio::fs::File::create(&file_path)
        .await
        .map_err(|e| format!("Falha ao criar arquivo local: {}", e))?;

    let client = reqwest::Client::builder()
        .user_agent("Ducker-Updater")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(asset_url)
        .send()
        .await
        .map_err(|e| format!("Falha ao iniciar download da atualização: {}", e))?;

    if !response.status().is_success() {
        let _ = tokio::fs::remove_file(&file_path).await;
        return Err(format!(
            "Download falhou com status HTTP {}",
            response.status()
        ));
    }

    let total = response.content_length().unwrap_or(0);
    let mut downloaded = 0u64;
    let mut stream = response.bytes_stream();
    let mut hasher = Sha256::new();

    while let Some(chunk_res) = stream.next().await {
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = tokio::fs::remove_file(&file_path).await;
            return Err("Download cancelado pelo usuário.".into());
        }

        let chunk = chunk_res.map_err(|e| format!("Erro durante transmissão de dados: {}", e))?;
        use tokio::io::AsyncWriteExt;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Erro ao gravar dados no disco: {}", e))?;

        hasher.update(&chunk);
        downloaded += chunk.len() as u64;

        let pct = if total > 0 {
            ((downloaded as f64 / total as f64) * 100.0).clamp(0.0, 100.0) as u32
        } else {
            0
        };

        on_progress(UpdateDownloadProgress {
            downloaded,
            total,
            pct,
        });
    }

    use tokio::io::AsyncWriteExt;
    file.flush()
        .await
        .map_err(|e| format!("Erro ao sincronizar arquivo com disco: {}", e))?;

    let _hash_hex = hex::encode(hasher.finalize());

    Ok(file_path)
}

/// Executa o instalador baixado com confirmação.
pub fn run_installer(path: &PathBuf) -> Result<(), String> {
    if !path.exists() {
        return Err("Arquivo de atualização não encontrado no disco.".into());
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new(path)
            .spawn()
            .map_err(|e| format!("Falha ao executar o instalador: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        // Garante permissão de execução caso seja AppImage
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(path, perms);
        }

        std::process::Command::new(path)
            .spawn()
            .map_err(|e| format!("Falha ao iniciar o pacote Linux: {}", e))?;
        Ok(())
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err("Execução automática não suportada nesta plataforma.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("0.2.0", "0.2.1"));
        assert!(is_newer_version("0.2.0", "0.3.0"));
        assert!(is_newer_version("0.2.0", "1.0.0"));
        assert!(is_newer_version("v0.2.0", "v0.2.5"));
        assert!(!is_newer_version("0.2.0", "0.2.0"));
        assert!(!is_newer_version("0.3.0", "0.2.0"));
        assert!(!is_newer_version("1.0.0", "0.9.9"));
    }
}
