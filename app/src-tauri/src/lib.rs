use ducker_core::{
    default_alias, default_save_dir, AdminPermissions, AdminSession, CosmeticCategory,
    DuckiveProfile, DuckiveView, Identity, Node, NodeConfig, NodeEvent, Peer, SendOptions,
    UnlockSummary,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Emitter, Manager, State};

mod updater;

pub struct AppState {
    pub node: Node,
    pub config_dir: PathBuf,
    pub duckive: tokio::sync::Mutex<DuckiveProfile>,
    pub admin_session: tokio::sync::Mutex<Option<AdminSession>>,
    pub update_cancel_flag: Arc<std::sync::atomic::AtomicBool>,
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
    let cache_dir = app
        .path()
        .app_cache_dir()
        .unwrap_or_else(|_| std::env::temp_dir());
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
    let cache_dir = app
        .path()
        .app_cache_dir()
        .unwrap_or_else(|_| std::env::temp_dir());
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
    let (real_files_count, _) =
        ducker_core::count_files_and_bytes(&path_bufs).unwrap_or((path_bufs.len() as u64, 0));
    let app_clone = app.clone();

    let start_time = std::time::Instant::now();
    let total_sent_bytes = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let total_clone = total_sent_bytes.clone();

    let res = state
        .node
        .send_files(&peer, path_bufs, SendOptions::default(), move |progress| {
            total_clone.store(progress.total, std::sync::atomic::Ordering::Relaxed);
            let _ = app_clone.emit("ducker://send-progress", progress);
        })
        .await;

    match res {
        Ok(()) => {
            let bytes = total_sent_bytes.load(std::sync::atomic::Ordering::Relaxed);
            let duration = start_time.elapsed().as_secs_f64().max(0.01);
            let speed_mbps = (bytes as f64 / 1_048_576.0) / duration;

            let my_os = std::env::consts::OS.to_lowercase();
            let peer_model = peer
                .info
                .device_model
                .as_deref()
                .unwrap_or("")
                .to_lowercase();
            let is_cross_platform = !peer_model.is_empty() && !peer_model.contains(&my_os);

            let mut duckive = state.duckive.lock().await;
            let summary = duckive.record_transfer(
                real_files_count,
                bytes,
                is_cross_platform,
                speed_mbps,
                false,
            );
            let _ = duckive.save(&state.config_dir);

            if !summary.newly_unlocked_items.is_empty()
                || !summary.newly_unlocked_achievements.is_empty()
                || summary.level_up
            {
                let _ = app.emit("ducker://duckive-unlock", &summary);
            }

            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}

#[derive(Deserialize)]
pub struct SendTextPayload {
    pub peer_key: String,
    pub text: String,
}

#[tauri::command]
async fn send_text(
    payload: SendTextPayload,
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

    let res = state
        .node
        .send_text(&peer, &payload.text, SendOptions::default())
        .await;

    match res {
        Ok(()) => {
            let mut duckive = state.duckive.lock().await;
            let summary = duckive.record_transfer(1, payload.text.len() as u64, false, 0.0, true);
            let _ = duckive.save(&state.config_dir);

            if !summary.newly_unlocked_items.is_empty()
                || !summary.newly_unlocked_achievements.is_empty()
            {
                let _ = app.emit("ducker://duckive-unlock", &summary);
            }

            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn get_duckive_view(state: State<'_, Arc<AppState>>) -> Result<DuckiveView, String> {
    let duckive = state.duckive.lock().await;
    Ok(duckive.to_view())
}

#[tauri::command]
async fn equip_duckive_item(
    item_id: String,
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<DuckiveView, String> {
    let mut duckive = state.duckive.lock().await;
    duckive.equip_item(&item_id).map_err(|e| e.to_string())?;
    duckive.save(&state.config_dir).map_err(|e| e.to_string())?;
    let view = duckive.to_view();
    let _ = app.emit("ducker://duckive-updated", &view);
    Ok(view)
}

#[tauri::command]
async fn unequip_duckive_slot(
    category: String,
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<DuckiveView, String> {
    let mut duckive = state.duckive.lock().await;
    match category.to_lowercase().as_str() {
        "head" => duckive.unequip_slot(CosmeticCategory::Head),
        "outfit" => duckive.unequip_slot(CosmeticCategory::Outfit),
        "accessory" => duckive.unequip_slot(CosmeticCategory::Accessory),
        "effect" => duckive.unequip_slot(CosmeticCategory::Effect),
        _ => return Err("Categoria inválida".into()),
    }
    duckive.save(&state.config_dir).map_err(|e| e.to_string())?;
    let view = duckive.to_view();
    let _ = app.emit("ducker://duckive-updated", &view);
    Ok(view)
}

#[tauri::command]
async fn set_duckive_name(
    name: String,
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<DuckiveView, String> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err("O nome do Duckive não pode ser vazio".into());
    }
    let mut duckive = state.duckive.lock().await;
    duckive.name = trimmed;
    duckive.save(&state.config_dir).map_err(|e| e.to_string())?;
    let view = duckive.to_view();
    let _ = app.emit("ducker://duckive-updated", &view);
    Ok(view)
}

#[tauri::command]
async fn record_transfer_progress(
    files_count: u64,
    bytes: u64,
    is_cross_platform: bool,
    speed_mbps: f64,
    is_text: bool,
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<UnlockSummary, String> {
    let mut duckive = state.duckive.lock().await;
    let summary =
        duckive.record_transfer(files_count, bytes, is_cross_platform, speed_mbps, is_text);
    duckive.save(&state.config_dir).map_err(|e| e.to_string())?;
    if !summary.newly_unlocked_items.is_empty()
        || !summary.newly_unlocked_achievements.is_empty()
        || summary.level_up
    {
        let _ = app.emit("ducker://duckive-unlock", &summary);
    }
    Ok(summary)
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

#[tauri::command]
async fn get_admin_session(
    state: State<'_, Arc<AppState>>,
) -> Result<Option<AdminSession>, String> {
    let session = state.admin_session.lock().await;
    Ok(session.clone())
}

#[tauri::command]
async fn auth_admin(
    code: String,
    state: State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
) -> Result<AdminSession, String> {
    #[cfg(debug_assertions)]
    {
        if code.trim() == "83578989b" {
            let session = AdminSession {
                is_admin: true,
                admin_name: "Faelindo".to_string(),
                role: "Administrador / Faelindo".to_string(),
                is_dev_mode: true,
                permissions: AdminPermissions::default(),
            };
            *state.admin_session.lock().await = Some(session.clone());
            let _ = app.emit("ducker://admin-changed", &session);
            return Ok(session);
        }
        Err("Código de acesso de desenvolvedor inválido".into())
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = code;
        let _ = state;
        let _ = app;
        Err("Autenticação de administrador desabilitada nesta versão de produção".into())
    }
}

#[tauri::command]
async fn logout_admin(
    state: State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    *state.admin_session.lock().await = None;
    let _ = app.emit("ducker://admin-changed", None::<AdminSession>);
    Ok(())
}

#[tauri::command]
async fn admin_test_unlock(
    item_id: String,
    state: State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
) -> Result<DuckiveView, String> {
    {
        let session = state.admin_session.lock().await;
        if session.as_ref().map(|s| s.is_admin) != Some(true) {
            return Err("Acesso negado: Requer privilégios de Administrador Faelindo".into());
        }
    }
    let mut duckive = state.duckive.lock().await;
    duckive.admin_unlock_item(&item_id);
    duckive.save(&state.config_dir).map_err(|e| e.to_string())?;
    let view = duckive.to_view();
    let _ = app.emit("ducker://duckive-updated", &view);
    Ok(view)
}

#[tauri::command]
async fn admin_simulate_transfer(
    files_count: u64,
    bytes: u64,
    state: State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
) -> Result<UnlockSummary, String> {
    {
        let session = state.admin_session.lock().await;
        if session.as_ref().map(|s| s.is_admin) != Some(true) {
            return Err("Acesso negado: Requer privilégios de Administrador Faelindo".into());
        }
    }
    let mut duckive = state.duckive.lock().await;
    let summary = duckive.record_transfer(files_count, bytes, true, 48.0, false);
    duckive.save(&state.config_dir).map_err(|e| e.to_string())?;
    let view = duckive.to_view();
    let _ = app.emit("ducker://duckive-updated", &view);
    if !summary.newly_unlocked_items.is_empty()
        || !summary.newly_unlocked_achievements.is_empty()
        || summary.level_up
    {
        let _ = app.emit("ducker://duckive-unlock", &summary);
    }
    Ok(summary)
}

#[derive(Serialize)]
pub struct AdminStatsDto {
    pub is_debug: bool,
    pub peer_count: usize,
    pub port: u16,
    pub save_dir: String,
    pub total_files_sent: u64,
    pub total_bytes_sent: u64,
    pub total_files_received: u64,
    pub total_bytes_received: u64,
    pub transfers_count: u64,
    pub duckive_level: u32,
    pub duckive_level_title: String,
    pub items_unlocked_count: usize,
    pub total_items_count: usize,
}

#[tauri::command]
async fn admin_get_stats(state: State<'_, Arc<AppState>>) -> Result<AdminStatsDto, String> {
    {
        let session = state.admin_session.lock().await;
        if session.as_ref().map(|s| s.is_admin) != Some(true) {
            return Err("Acesso negado: Requer privilégios de Administrador Faelindo".into());
        }
    }
    let peers = state.node.peers().len();
    let port = state.node.port();
    let save_dir = state.node.save_dir().to_string_lossy().to_string();
    let duckive = state.duckive.lock().await;
    let view = duckive.to_view();

    Ok(AdminStatsDto {
        #[cfg(debug_assertions)]
        is_debug: true,
        #[cfg(not(debug_assertions))]
        is_debug: false,
        peer_count: peers,
        port,
        save_dir,
        total_files_sent: view.files_sent_count,
        total_bytes_sent: view.total_bytes_sent,
        total_files_received: view.files_received_count,
        total_bytes_received: view.total_bytes_received,
        transfers_count: view.transfers_count,
        duckive_level: view.level.level,
        duckive_level_title: view.level.title,
        items_unlocked_count: view.items.iter().filter(|i| i.unlocked).count(),
        total_items_count: view.items.len(),
    })
}

#[tauri::command]
async fn check_for_updates() -> Result<updater::UpdateCheckResult, String> {
    let current_version = env!("CARGO_PKG_VERSION");
    updater::check_github_update(current_version).await
}

#[tauri::command]
async fn download_update(
    asset_url: String,
    asset_name: String,
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    use std::sync::atomic::Ordering;
    state.update_cancel_flag.store(false, Ordering::Relaxed);
    let cancel_flag = state.update_cancel_flag.clone();

    let save_dir = std::env::temp_dir().join("ducker-update");
    let app_handle = app.clone();

    let downloaded_path = updater::download_installer(
        &asset_url,
        &asset_name,
        &save_dir,
        move |progress| {
            let _ = app_handle.emit("ducker://update-download-progress", progress);
        },
        cancel_flag,
    )
    .await?;

    Ok(downloaded_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn cancel_update_download(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    use std::sync::atomic::Ordering;
    state.update_cancel_flag.store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
async fn install_update(file_path: String) -> Result<(), String> {
    let path = PathBuf::from(file_path);
    updater::run_installer(&path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());

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
            let config_dir = app.path().app_data_dir().unwrap_or_else(|_| {
                Identity::default_config_dir().unwrap_or_else(|_| PathBuf::from("."))
            });

            let def_name = default_alias();
            let identity = Identity::load_or_create(&config_dir, &def_name)
                .expect("Falha ao inicializar identidade do nó");

            let duckive_profile = DuckiveProfile::load_or_create(&config_dir, &identity.alias)
                .expect("Falha ao inicializar perfil Duckive");

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
            let (node, mut events) =
                tauri::async_runtime::block_on(async { Node::start(config).await })
                    .expect("Falha ao inicializar o nó de rede do Ducker");

            let state = Arc::new(AppState {
                node,
                config_dir,
                duckive: tokio::sync::Mutex::new(duckive_profile),
                admin_session: tokio::sync::Mutex::new(None),
                update_cancel_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            });
            app.manage(state.clone());

            let app_handle = app.handle().clone();
            let state_events = state.clone();
            tauri::async_runtime::spawn(async move {
                while let Ok(event) = events.recv().await {
                    if let NodeEvent::FileReceived { path, .. } = &event {
                        let file_bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                        let mut duckive = state_events.duckive.lock().await;
                        duckive.record_files_received(1, file_bytes);
                        let _ = duckive.save(&state_events.config_dir);
                    }
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
            open_save_dir,
            get_duckive_view,
            equip_duckive_item,
            unequip_duckive_slot,
            set_duckive_name,
            record_transfer_progress,
            get_admin_session,
            auth_admin,
            logout_admin,
            admin_test_unlock,
            admin_simulate_transfer,
            admin_get_stats,
            check_for_updates,
            download_update,
            cancel_update_download,
            install_update
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao executar aplicativo Tauri");
}
