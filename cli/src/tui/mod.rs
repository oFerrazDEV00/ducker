pub mod state;
pub mod theme;
pub mod ui;

use std::io::{self, stdout};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, EventStream, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures_util::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use ducker_core::{default_save_dir, Identity, Node, NodeConfig, NodeEvent, SendOptions};

use self::state::{ActiveTab, LogKind, ModalMode, TransferInfo, TuiApp};

pub async fn run(identity: &Identity, config_dir: &Path) -> io::Result<()> {
    // 1. Configurar Terminal e Panic Hook para garantir restauração impecável
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
        default_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Inicializar Node do Core
    let save_dir = default_save_dir();
    let _ = std::fs::create_dir_all(&save_dir);
    let config = NodeConfig::new(identity.clone(), save_dir);

    let (node, mut events) = match Node::start(config).await {
        Ok(res) => res,
        Err(e) => {
            let _ = disable_raw_mode();
            let _ = execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                crossterm::cursor::Show
            );
            eprintln!("Erro ao inicializar o nó de rede do Ducker: {}", e);
            return Ok(());
        }
    };

    let node = Arc::new(node);
    let mut app = TuiApp::new(identity.clone(), config_dir.to_path_buf(), node.clone());

    // Carrega peers iniciais e dispara anúncio de rede
    app.update_peers(node.peers());
    let _ = node.refresh().await;

    // 3. Loop Principal Assíncrono com EventStream e Tokio Select
    let mut ticker = tokio::time::interval(Duration::from_millis(200));
    let mut event_stream = EventStream::new();

    loop {
        // Desenha a interface
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if app.should_quit {
            break;
        }

        tokio::select! {
            _ = ticker.tick() => {
                app.tick = app.tick.wrapping_add(1);

                // A cada ~10 segundos (50 ticks), atualiza a lista de peers
                if app.tick % 50 == 0 {
                    app.update_peers(node.peers());
                }
            }

            Some(Ok(event)) = event_stream.next() => {
                if let Event::Key(key) = event {
                    if key.kind == KeyEventKind::Press {
                        handle_key_event(&mut app, key).await;
                    }
                }
            }

            Ok(event) = events.recv() => {
                handle_node_event(&mut app, event).await;
            }
        }
    }

    // 4. Restauração do Terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show
    )?;
    println!("\n🦆 Ducker encerrado com sucesso. Be simple, be duck.\n");

    Ok(())
}

async fn handle_node_event(app: &mut TuiApp, event: NodeEvent) {
    match event {
        NodeEvent::PeerDiscovered { peer } => {
            app.add_log(
                format!("Dispositivo '{}' encontrado ({})", peer.info.alias, peer.ip),
                LogKind::Info,
            );
            app.update_peers(app.node.peers());
        }
        NodeEvent::IncomingRequest {
            session_id,
            sender,
            files,
            auto_accepted,
        } => {
            let count = files.len();
            let total_size: u64 = files.iter().map(|f| f.size).sum();
            let names: Vec<String> = files.iter().take(3).map(|f| f.file_name.clone()).collect();
            let summary = if count > 3 {
                format!("{} e mais {} arquivo(s)", names.join(", "), count - 3)
            } else {
                names.join(", ")
            };

            if auto_accepted {
                app.add_log(
                    format!(
                        "Recebimento de {} aceito automaticamente ({} arquivo(s))",
                        sender.alias, count
                    ),
                    LogKind::Success,
                );
            } else {
                app.modal = ModalMode::IncomingRequest {
                    session_id,
                    sender_name: format!(
                        "{} ({})",
                        sender.alias,
                        sender.device_model.as_deref().unwrap_or("Dispositivo")
                    ),
                    files_summary: summary,
                    total_size,
                };
            }
        }
        NodeEvent::ReceiveProgress {
            received, total, ..
        } => {
            let pct = if total > 0 {
                ((received as f64 / total as f64) * 100.0).clamp(0.0, 100.0) as u32
            } else {
                0
            };

            let now = Instant::now();
            if let Some(ref mut t) = app.active_transfer {
                let duration = now.duration_since(t.start_time).as_secs_f64().max(0.1);
                t.transferred_bytes = received;
                t.pct = pct;
                t.speed_mbps = (received as f64 / 1_048_576.0) / duration;
            } else {
                app.active_transfer = Some(TransferInfo {
                    file_name: "Arquivo recebido".to_string(),
                    is_incoming: true,
                    total_bytes: total,
                    transferred_bytes: received,
                    start_time: now,
                    pct,
                    speed_mbps: 0.0,
                });
            }
        }
        NodeEvent::FileReceived {
            file_name, path, ..
        } => {
            app.active_transfer = None;
            app.add_log(
                format!("Arquivo '{}' salvo com sucesso!", file_name),
                LogKind::Success,
            );

            let bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            app.duckive.record_files_received(1, bytes);
            let _ = app.duckive.save(&app.config_dir);
        }
        NodeEvent::TextReceived { sender, text } => {
            app.add_log(format!("{}: {}", sender.alias, text), LogKind::Message);
        }
        NodeEvent::SessionFinished { .. } | NodeEvent::SessionCancelled { .. } => {
            app.active_transfer = None;
        }
    }
}

fn autocomplete_path(input: &str) -> Option<String> {
    let clean = input.trim().trim_matches('"');
    let path = Path::new(clean);

    let (dir, prefix) = if clean.is_empty() {
        (Path::new("."), "")
    } else if clean.ends_with('/') || clean.ends_with('\\') {
        (path, "")
    } else {
        match path.parent() {
            Some(p) if !p.as_os_str().is_empty() => {
                (p, path.file_name().and_then(|f| f.to_str()).unwrap_or(""))
            }
            _ => (Path::new("."), clean),
        }
    };

    let entries = std::fs::read_dir(dir).ok()?;
    let mut matches = Vec::new();
    for entry in entries.flatten() {
        if let Ok(name) = entry.file_name().into_string() {
            if name.to_lowercase().starts_with(&prefix.to_lowercase()) {
                let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                let full = if dir == Path::new(".") {
                    if is_dir {
                        format!("{}/", name)
                    } else {
                        name
                    }
                } else {
                    let dir_str = dir.to_string_lossy();
                    let sep = if dir_str.contains('\\') { "\\" } else { "/" };
                    if is_dir {
                        format!("{}{}{}{}", dir_str, sep, name, sep)
                    } else {
                        format!("{}{}{}", dir_str, sep, name)
                    }
                };
                matches.push(full);
            }
        }
    }

    matches.sort();
    matches.into_iter().next()
}

async fn handle_key_event(app: &mut TuiApp, key: event::KeyEvent) {
    // 1. Teclas quando um modal está ativo
    match &mut app.modal {
        ModalMode::IncomingRequest {
            session_id,
            sender_name,
            ..
        } => {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    let sid = session_id.clone();
                    let sname = sender_name.clone();
                    app.node.respond(&sid, true);
                    app.add_log(
                        format!("Transferência aceita de {}.", sname),
                        LogKind::Success,
                    );
                    app.modal = ModalMode::None;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    let sid = session_id.clone();
                    let sname = sender_name.clone();
                    app.node.respond(&sid, false);
                    app.add_log(
                        format!("Transferência recusada de {}.", sname),
                        LogKind::Warning,
                    );
                    app.modal = ModalMode::None;
                }
                _ => {}
            }
            return;
        }
        ModalMode::RenameDuckive { value } => {
            match key.code {
                KeyCode::Esc => {
                    app.modal = ModalMode::None;
                }
                KeyCode::Enter => {
                    let trimmed = value.trim().to_string();
                    app.modal = ModalMode::None;
                    if !trimmed.is_empty() {
                        app.duckive.name = trimmed;
                        let _ = app.duckive.save(&app.config_dir);
                        app.add_log(
                            format!("Duckive renomeado para '{}'! 🦆", app.duckive.name),
                            LogKind::Success,
                        );
                    }
                }
                KeyCode::Backspace => {
                    value.pop();
                }
                KeyCode::Char(c) => {
                    value.push(c);
                }
                _ => {}
            }
            return;
        }
        ModalMode::InputText {
            value, target_peer, ..
        } => {
            match key.code {
                KeyCode::Esc => {
                    app.modal = ModalMode::None;
                }
                KeyCode::Enter => {
                    let msg = value.trim().to_string();
                    let peer = target_peer.clone();
                    app.modal = ModalMode::None;

                    if !msg.is_empty() {
                        app.add_log(
                            format!("Enviando mensagem para {}...", peer.info.alias),
                            LogKind::Info,
                        );
                        let node = app.node.clone();
                        let alias = peer.info.alias.clone();
                        tokio::spawn(async move {
                            match node.send_text(&peer, &msg, SendOptions::default()).await {
                                Ok(()) => {}
                                Err(e) => eprintln!("Erro ao enviar texto: {}", e),
                            }
                        });
                        app.add_log(
                            format!("Mensagem enviada para {}! 💬", alias),
                            LogKind::Success,
                        );

                        // Registra mensagem de texto no Duckive (desbloqueia conquista whatsapp_green)
                        let _ = app.duckive.record_transfer(0, 0, false, 0.0, true);
                        let _ = app.duckive.save(&app.config_dir);
                    }
                }
                KeyCode::Backspace => {
                    value.pop();
                }
                KeyCode::Char(c) => {
                    value.push(c);
                }
                _ => {}
            }
            return;
        }
        ModalMode::InputFilePath {
            value, target_peer, ..
        } => {
            match key.code {
                KeyCode::Esc => {
                    app.modal = ModalMode::None;
                }
                KeyCode::Tab => {
                    if let Some(completed) = autocomplete_path(value) {
                        *value = completed;
                    }
                }
                KeyCode::Enter => {
                    let input = value.trim().trim_matches('"').to_string();
                    let peer = target_peer.clone();
                    app.modal = ModalMode::None;

                    if !input.is_empty() {
                        let path = PathBuf::from(&input);
                        if !path.exists() {
                            app.add_log(
                                format!("Caminho '{}' não encontrado no disco.", input),
                                LogKind::Error,
                            );
                            return;
                        }

                        let paths = vec![path];
                        let (files_count, total_bytes) = ducker_core::count_files_and_bytes(&paths)
                            .unwrap_or((paths.len() as u64, 0));

                        app.add_log(
                            format!(
                                "Iniciando envio de {} arquivo(s) para {}...",
                                files_count, peer.info.alias
                            ),
                            LogKind::Info,
                        );

                        let node = app.node.clone();
                        let start_time = Instant::now();
                        let my_os = std::env::consts::OS.to_lowercase();
                        let peer_model = peer
                            .info
                            .device_model
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase();
                        let is_cross_platform =
                            !peer_model.is_empty() && !peer_model.contains(&my_os);

                        let send_res = node
                            .send_files(&peer, paths, SendOptions::default(), |_| {})
                            .await;

                        match send_res {
                            Ok(()) => {
                                let duration = start_time.elapsed().as_secs_f64().max(0.1);
                                let speed_mbps = (total_bytes as f64 / 1_048_576.0) / duration;
                                app.add_log(
                                    format!(
                                        "Envio para {} concluído com sucesso! 🦆",
                                        peer.info.alias
                                    ),
                                    LogKind::Success,
                                );

                                let summary = app.duckive.record_transfer(
                                    files_count,
                                    total_bytes,
                                    is_cross_platform,
                                    speed_mbps,
                                    false,
                                );
                                let _ = app.duckive.save(&app.config_dir);

                                if summary.level_up {
                                    app.add_log(
                                        format!(
                                            "⭐ LEVEL UP! Seu Duckive subiu para o Nível {}!",
                                            summary.current_level
                                        ),
                                        LogKind::Success,
                                    );
                                }
                                for it in summary.newly_unlocked_items {
                                    app.add_log(
                                        format!(
                                            "🎉 Novo item cosmético desbloqueado: {}!",
                                            it.name
                                        ),
                                        LogKind::Success,
                                    );
                                }
                            }
                            Err(e) => {
                                app.add_log(
                                    format!("Falha no envio para {}: {}", peer.info.alias, e),
                                    LogKind::Error,
                                );
                            }
                        }
                    }
                }
                KeyCode::Backspace => {
                    value.pop();
                }
                KeyCode::Char(c) => {
                    value.push(c);
                }
                _ => {}
            }
            return;
        }
        ModalMode::None => {}
    }

    // 2. Se a barra de pesquisa de dispositivos estiver ativa
    if app.is_filtering {
        match key.code {
            KeyCode::Esc => {
                app.is_filtering = false;
                app.filter_query.clear();
            }
            KeyCode::Enter => {
                app.is_filtering = false;
            }
            KeyCode::Backspace => {
                app.filter_query.pop();
            }
            KeyCode::Up => {
                app.select_prev_filtered_peer();
            }
            KeyCode::Down => {
                app.select_next_filtered_peer();
            }
            KeyCode::Char(c) => {
                app.filter_query.push(c);
            }
            _ => {}
        }
        return;
    }

    // 3. Teclas no modo normal (sem modal aberto e sem pesquisa ativa)
    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Tab => {
            app.active_tab = match app.active_tab {
                ActiveTab::Radar => ActiveTab::Duckive,
                ActiveTab::Duckive => ActiveTab::Help,
                ActiveTab::Help => ActiveTab::Radar,
            };
        }
        KeyCode::Char('1') => app.active_tab = ActiveTab::Radar,
        KeyCode::Char('2') | KeyCode::Char('d') => app.active_tab = ActiveTab::Duckive,
        KeyCode::Char('3') | KeyCode::Char('?') => app.active_tab = ActiveTab::Help,

        // Ativação da pesquisa / filtro de dispositivos
        KeyCode::Char('/') => {
            app.is_filtering = true;
            app.active_tab = ActiveTab::Radar;
        }

        // Navegação de acordo com a aba ativa
        KeyCode::Up | KeyCode::Char('k') => match app.active_tab {
            ActiveTab::Radar => app.select_prev_filtered_peer(),
            ActiveTab::Duckive => app.select_prev_cosmetic(),
            ActiveTab::Help => {}
        },
        KeyCode::Down | KeyCode::Char('j') => match app.active_tab {
            ActiveTab::Radar => app.select_next_filtered_peer(),
            ActiveTab::Duckive => app.select_next_cosmetic(),
            ActiveTab::Help => {}
        },

        // Ações da aba Duckive (equipar / renomear)
        KeyCode::Enter | KeyCode::Char('e') | KeyCode::Char(' ')
            if app.active_tab == ActiveTab::Duckive =>
        {
            match app.toggle_equip_selected_cosmetic() {
                Ok(msg) => app.add_log(msg, LogKind::Success),
                Err(msg) => app.add_log(msg, LogKind::Warning),
            }
        }
        KeyCode::Char('n') if app.active_tab == ActiveTab::Duckive => {
            app.modal = ModalMode::RenameDuckive {
                value: app.duckive.name.clone(),
            };
        }

        // Ações de rede
        KeyCode::Char('r') => {
            app.set_status("Buscando dispositivos...".to_string());
            app.add_log(
                "Varredura de rede local disparada...".to_string(),
                LogKind::Info,
            );
            let node = app.node.clone();
            tokio::spawn(async move {
                node.refresh().await;
            });
            app.update_peers(app.node.peers());
        }
        KeyCode::Char('s') => {
            if let Some(peer) = app.selected_peer().cloned() {
                app.modal = ModalMode::InputFilePath {
                    title: format!("Enviar Arquivos para {}", peer.info.alias),
                    prompt: "Digite o caminho do arquivo ou pasta (Tab completa)".to_string(),
                    value: String::new(),
                    target_peer: peer,
                };
            } else {
                app.set_status("Selecione um dispositivo primeiro.".to_string());
            }
        }
        KeyCode::Char('t') => {
            if let Some(peer) = app.selected_peer().cloned() {
                app.modal = ModalMode::InputText {
                    title: format!("Enviar Mensagem para {}", peer.info.alias),
                    prompt: "Digite o texto da mensagem".to_string(),
                    value: String::new(),
                    target_peer: peer,
                };
            } else {
                app.set_status("Selecione um dispositivo primeiro.".to_string());
            }
        }
        _ => {}
    }
}
