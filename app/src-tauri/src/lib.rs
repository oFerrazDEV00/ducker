use std::path::PathBuf;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, State};
use ducker_core::{default_alias, default_save_dir, Identity, Node, NodeConfig, Peer, SendOptions};

pub struct AppState {
    pub node: Node,
    pub config_dir: PathBuf,
}

#[derive(Serialize)]
pub struct IdentityDto {
    pub alias: String,
    pub quac_id: u32,
    pub fingerprint: String,
    pub port: u16,
    pub save_dir: String,
}

#[tauri::command]
async fn get_identity(state: State<'_, Arc<AppState>>) -> Result<IdentityDto, String> {
    let id = state.node.identity();
    Ok(IdentityDto {
        alias: id.alias,
        quac_id: id.quac_id,
        fingerprint: id.fingerprint,
        port: state.node.port(),
        save_dir: state.node.save_dir().to_string_lossy().to_string(),
    })
}

#[tauri::command]
async fn set_alias(alias: String, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let trimmed = alias.trim().to_string();
    if trimmed.is_empty() {
        return Err("O apelido não pode ser vazio".into());
    }
    state.node.set_alias(&trimmed).await;
    let mut id = state.node.identity();
    id.alias = trimmed;
    id.save(&state.config_dir).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn list_peers(state: State<'_, Arc<AppState>>) -> Result<Vec<Peer>, String> {
    Ok(state.node.peers())
}

#[tauri::command]
async fn refresh(state: State<'_, Arc<AppState>>) -> Result<Vec<Peer>, String> {
    state.node.refresh().await;
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    Ok(state.node.peers())
}

#[tauri::command]
async fn scan_network(state: State<'_, Arc<AppState>>) -> Result<usize, String> {
    Ok(state.node.scan_subnet().await)
}

#[tauri::command]
async fn pick_files() -> Result<Vec<String>, String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let files = rfd::AsyncFileDialog::new()
            .set_title("Ducker — Selecione arquivos para enviar")
            .pick_files()
            .await;

        match files {
            Some(handles) => Ok(handles
                .into_iter()
                .map(|h| h.path().to_string_lossy().to_string())
                .collect()),
            None => Ok(vec![]),
        }
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        Ok(vec![])
    }
}

#[derive(Deserialize)]
pub struct StageChunkPayload {
    pub stage_id: String,
    pub file_name: String,
    pub data: Vec<u8>,
    pub is_first: bool,
}

#[tauri::command]
async fn stage_file_chunk(
    payload: StageChunkPayload,
    app: tauri::AppHandle,
) -> Result<String, String> {
    use std::io::Write;
    let cache_dir = app.path().app_cache_dir().unwrap_or_else(|_| std::env::temp_dir());
    let staging_dir = cache_dir.join("ducker_staging");
    std::fs::create_dir_all(&staging_dir).map_err(|e| e.to_string())?;

    let clean_name = std::path::Path::new(&payload.file_name)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "file.bin".to_string());

    let file_path = staging_dir.join(format!("{}_{}", payload.stage_id, clean_name));

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(payload.is_first)
        .append(!payload.is_first)
        .open(&file_path)
        .map_err(|e| format!("Falha ao abrir arquivo temporário: {e}"))?;

    file.write_all(&payload.data)
        .map_err(|e| format!("Falha ao gravar arquivo temporário: {e}"))?;
    file.flush()
        .map_err(|e| format!("Falha ao descarregar buffer temporário: {e}"))?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn clear_staged_files(app: tauri::AppHandle) -> Result<(), String> {
    let cache_dir = app.path().app_cache_dir().unwrap_or_else(|_| std::env::temp_dir());
    let staging_dir = cache_dir.join("ducker_staging");
    if staging_dir.exists() {
        let _ = std::fs::remove_dir_all(&staging_dir);
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct SendFilesPayload {
    pub peer_key: String,
    pub paths: Vec<String>,
}

#[tauri::command]
async fn send_files(
    payload: SendFilesPayload,
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let peer = state
        .node
        .find_peer(&payload.peer_key)
        .or_else(|| {
            state.node.peers().into_iter().find(|p| {
                p.key().eq_ignore_ascii_case(&payload.peer_key)
                    || p.info.fingerprint.eq_ignore_ascii_case(&payload.peer_key)
                    || p.info.alias.eq_ignore_ascii_case(&payload.peer_key)
                    || format!("{}:{}", p.ip, p.port) == payload.peer_key
                    || p.ip.to_string() == payload.peer_key
            })
        })
        .ok_or_else(|| format!("Destinatário '{}' não encontrado na rede", payload.peer_key))?;

    let path_bufs: Vec<PathBuf> = payload.paths.into_iter().map(PathBuf::from).collect();
    let app_clone = app.clone();

    state
        .node
        .send_files(&peer, path_bufs, SendOptions::default(), move |progress| {
            let _ = app_clone.emit("ducker://send-progress", progress);
        })
        .await
        .map_err(|e| e.to_string())
}

#[derive(Deserialize)]
pub struct SendTextPayload {
    pub peer_key: String,
    pub text: String,
}

#[tauri::command]
async fn send_text(
    payload: SendTextPayload,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let peer = state
        .node
        .find_peer(&payload.peer_key)
        .or_else(|| {
            state.node.peers().into_iter().find(|p| {
                p.key().eq_ignore_ascii_case(&payload.peer_key)
                    || p.info.fingerprint.eq_ignore_ascii_case(&payload.peer_key)
                    || p.info.alias.eq_ignore_ascii_case(&payload.peer_key)
                    || format!("{}:{}", p.ip, p.port) == payload.peer_key
                    || p.ip.to_string() == payload.peer_key
            })
        })
        .ok_or_else(|| format!("Destinatário '{}' não encontrado na rede", payload.peer_key))?;

    state
        .node
        .send_text(&peer, &payload.text, SendOptions::default())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn respond_request(
    session_id: String,
    accept: bool,
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    Ok(state.node.respond(&session_id, accept))
}

#[tauri::command]
async fn open_save_dir(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let dir = state.node.save_dir();
    let _ = std::fs::create_dir_all(dir);

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer.exe").arg(dir).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(dir).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = dir;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init());

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }));
    }

    builder
        .setup(|app| {
            let config_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| Identity::default_config_dir().unwrap_or_else(|_| PathBuf::from(".")));

            let def_name = default_alias();
            let identity = Identity::load_or_create(&config_dir, &def_name)
                .expect("Falha ao inicializar identidade do nó");

            let save_dir = app
                .path()
                .download_dir()
                .or_else(|_| app.path().document_dir())
                .or_else(|_| app.path().app_data_dir())
                .map(|p| p.join("Ducker"))
                .unwrap_or_else(|_| default_save_dir());
            let _ = std::fs::create_dir_all(&save_dir);

            let config = NodeConfig::new(identity, save_dir);

            // Iniciar o nó e registrar o estado sincronicamente no setup
            let (node, mut events) = tauri::async_runtime::block_on(async {
                Node::start(config).await
            }).expect("Falha ao inicializar o nó de rede do Ducker");

            let state = Arc::new(AppState {
                node,
                config_dir,
            });
            app.manage(state);

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                while let Ok(event) = events.recv().await {
                    let _ = app_handle.emit("ducker://event", event);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_identity,
            set_alias,
            list_peers,
            refresh,
            scan_network,
            pick_files,
            stage_file_chunk,
            clear_staged_files,
            send_files,
            send_text,
            respond_request,
            open_save_dir
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao executar aplicativo Tauri");
}
