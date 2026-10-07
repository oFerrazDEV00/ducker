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
    // Pequena espera para coleta de pacotes de anúncio
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    Ok(state.node.peers())
}

#[tauri::command]
async fn scan_network(state: State<'_, Arc<AppState>>) -> Result<usize, String> {
    Ok(state.node.scan_subnet().await)
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
        .peers()
        .into_iter()
        .find(|p| p.key() == payload.peer_key || p.info.alias == payload.peer_key)
        .ok_or_else(|| "Destinatário não encontrado".to_string())?;

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
        .peers()
        .into_iter()
        .find(|p| p.key() == payload.peer_key || p.info.alias == payload.peer_key)
        .ok_or_else(|| "Destinatário não encontrado".to_string())?;

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

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| Identity::default_config_dir().unwrap_or_else(|_| PathBuf::from(".")));

            let def_name = default_alias();
            let identity = Identity::load_or_create(&config_dir, &def_name)
                .expect("Falha ao inicializar identidade do nó");

            let save_dir = default_save_dir();
            let _ = std::fs::create_dir_all(&save_dir);

            let config = NodeConfig::new(identity, save_dir);

            let app_handle = app.handle().clone();

            // Iniciar o nó em um runtime async do tokio
            tauri::async_runtime::spawn(async move {
                let (node, mut events) = match Node::start(config).await {
                    Ok(res) => res,
                    Err(e) => {
                        eprintln!("Erro ao iniciar Ducker Node: {e}");
                        return;
                    }
                };

                let state = Arc::new(AppState {
                    node,
                    config_dir,
                });
                app_handle.manage(state);

                // Encaminhar todos os eventos do Node para a interface web via Tauri
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
            send_files,
            send_text,
            respond_request,
            open_save_dir
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao executar aplicativo Tauri");
}
