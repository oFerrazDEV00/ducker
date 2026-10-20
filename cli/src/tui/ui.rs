use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph, Tabs, Wrap,
};
use ratatui::Frame;

use super::state::{ActiveTab, LogKind, ModalMode, TuiApp};
use super::theme::*;

pub fn draw(f: &mut Frame, app: &mut TuiApp) {
    let size = f.area();

    // Proteção para terminais pequenos
    if size.width < 65 || size.height < 16 {
        render_small_terminal_warning(f, size);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header + Info bar
            Constraint::Length(3), // Navigation Tabs
            Constraint::Min(8),    // Main workspace
            Constraint::Length(1), // Footer / Shortcuts bar
        ])
        .split(size);

    render_header(f, chunks[0], app);
    render_tabs(f, chunks[1], app);

    match app.active_tab {
        ActiveTab::Radar => render_radar_view(f, chunks[2], app),
        ActiveTab::Duckive => render_duckive_view(f, chunks[2], app),
        ActiveTab::Help => render_help_view(f, chunks[2], app),
    }

    render_footer(f, chunks[3], app);

    // Renderiza modais se houver algum ativo
    match &app.modal {
        ModalMode::IncomingRequest {
            session_id: _,
            sender_name,
            files_summary,
            total_size,
        } => {
            render_incoming_modal(f, size, sender_name, files_summary, *total_size);
        }
        ModalMode::InputText {
            title,
            prompt,
            value,
            target_peer,
        } => {
            render_input_modal(
                f,
                size,
                title,
                prompt,
                value,
                &target_peer.info.alias,
                app.tick,
            );
        }
        ModalMode::InputFilePath {
            title,
            prompt,
            value,
            target_peer,
        } => {
            render_input_modal(
                f,
                size,
                title,
                prompt,
                value,
                &target_peer.info.alias,
                app.tick,
            );
        }
        ModalMode::RenameDuckive { value } => {
            render_rename_modal(f, size, value, app.tick);
        }
        ModalMode::None => {}
    }
}

fn render_small_terminal_warning(f: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(vec![
            Span::styled("🦆 DUCKER", Style::default().fg(DUCK_YELLOW).bold()),
            Span::raw(" — Janela do terminal muito pequena"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tamanho mínimo exigido: ", Style::default().fg(TEXT_MUTED)),
            Span::styled(
                "65 colunas x 16 linhas",
                Style::default().fg(DUCK_YELLOW).bold(),
            ),
        ]),
        Line::from(vec![Span::styled(
            format!(
                "Tamanho atual: {} colunas x {} linhas",
                area.width, area.height
            ),
            Style::default().fg(TEXT_DIM),
        )]),
        Line::from(""),
        Line::from(Span::styled(
            "Por favor, amplie o terminal para continuar. 🦆",
            Style::default().fg(SUCCESS_GREEN),
        )),
    ];

    let p = Paragraph::new(text).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(DUCK_YELLOW))
            .title(" Atenção "),
    );
    f.render_widget(p, area);
}

fn render_header(f: &mut Frame, area: Rect, app: &TuiApp) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DUCK_YELLOW));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let header_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    // Linha 1: Marca + Slogan + Status
    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner = spinner_frames[(app.tick as usize / 2) % spinner_frames.len()];

    let line1 = Line::from(vec![
        Span::styled(" 🦆 DUCKER", Style::default().fg(DUCK_YELLOW).bold()),
        Span::styled(" • ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            "Be simple, be duck.",
            Style::default().fg(TEXT_MUTED).italic(),
        ),
        Span::styled(" [v1.0.0]", Style::default().fg(DUCK_YELLOW_DARK)),
        Span::raw("   "),
        Span::styled("● ONLINE", Style::default().fg(SUCCESS_GREEN).bold()),
        Span::styled(
            format!(
                " (Porta {} / {})",
                app.node.port(),
                app.node.config().protocol.scheme()
            ),
            Style::default().fg(TEXT_DIM),
        ),
        Span::styled(
            format!("  {} radar ativo", spinner),
            Style::default().fg(DUCK_YELLOW),
        ),
    ]);

    // Linha 2: Identidade do dispositivo local
    let short_fp = if app.identity.fingerprint.len() >= 12 {
        &app.identity.fingerprint[..12]
    } else {
        &app.identity.fingerprint
    };

    let peers_count = app.peers.len();
    let peers_text = if peers_count == 1 {
        "1 dispositivo próximo".to_string()
    } else {
        format!("{} dispositivos próximos", peers_count)
    };

    let line2 = Line::from(vec![
        Span::styled(" Dispositivo: ", Style::default().fg(TEXT_MUTED)),
        Span::styled(&app.identity.alias, Style::default().fg(DUCK_YELLOW).bold()),
        Span::styled(" • ID Quac: ", Style::default().fg(TEXT_MUTED)),
        Span::styled(
            format!("#{}", app.identity.quac_id),
            Style::default().fg(SUCCESS_GREEN).bold(),
        ),
        Span::styled(" • Fingerprint: ", Style::default().fg(TEXT_MUTED)),
        Span::styled(short_fp, Style::default().fg(TEXT_DIM)),
        Span::styled("  │  ", Style::default().fg(TEXT_DIM)),
        Span::styled(peers_text, Style::default().fg(INFO_BLUE).bold()),
    ]);

    f.render_widget(Paragraph::new(line1), header_rows[0]);
    f.render_widget(Paragraph::new(line2), header_rows[1]);
}

fn render_tabs(f: &mut Frame, area: Rect, app: &TuiApp) {
    let titles = vec![
        " [1] Radar & Transferências ",
        " [2] Duckive Companion ",
        " [3] Ajuda & Comandos ",
    ];

    let selected_idx = match app.active_tab {
        ActiveTab::Radar => 0,
        ActiveTab::Duckive => 1,
        ActiveTab::Help => 2,
    };

    let tabs = Tabs::new(titles)
        .select(selected_idx)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(TEXT_DIM)),
        )
        .highlight_style(
            Style::default()
                .fg(DUCK_YELLOW)
                .bold()
                .add_modifier(Modifier::UNDERLINED),
        )
        .style(Style::default().fg(TEXT_MUTED));

    f.render_widget(tabs, area);
}

fn render_radar_view(f: &mut Frame, area: Rect, app: &mut TuiApp) {
    let main_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(46), Constraint::Percentage(54)])
        .split(area);

    render_peers_list(f, main_cols[0], app);

    // Coluna direita: Transferência Ativa + Log de Atividade + Faixa Duckive
    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Transferência ativa
            Constraint::Min(6),    // Log de atividade
            Constraint::Length(3), // Companion Duckive mini banner
        ])
        .split(main_cols[1]);

    render_active_transfer(f, right_rows[0], app);
    render_activity_log(f, right_rows[1], app);
    render_duckive_strip(f, right_rows[2], app);
}

fn render_peers_list(f: &mut Frame, area: Rect, app: &TuiApp) {
    let filtered = app.filtered_peers();
    let title = if !app.filter_query.is_empty() {
        format!(
            " Dispositivos ({}/{} filtrados) ",
            filtered.len(),
            app.peers.len()
        )
    } else {
        format!(" Dispositivos na Rede ({}) ", app.peers.len())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DUCK_YELLOW))
        .title(Span::styled(title, Style::default().fg(DUCK_YELLOW).bold()));

    if app.peers.is_empty() {
        let spinner_chars = ["|", "/", "-", "\\"];
        let sp = spinner_chars[(app.tick as usize / 3) % 4];

        let empty_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(format!(" [ {} ] ", sp), Style::default().fg(DUCK_YELLOW)),
                Span::styled(
                    "Procurando dispositivos na rede local...",
                    Style::default().fg(TEXT_MUTED),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                " • Certifique-se de estar conectado ao mesmo Wi-Fi.",
                Style::default().fg(TEXT_DIM),
            )),
            Line::from(Span::styled(
                " • Pressione [r] para forçar nova varredura.",
                Style::default().fg(TEXT_DIM),
            )),
            Line::from(Span::styled(
                " • Abra o Ducker ou LocalSend em outro aparelho.",
                Style::default().fg(TEXT_DIM),
            )),
        ];
        let p = Paragraph::new(empty_text)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(p, area);
        return;
    }

    let (list_area, search_area) = if app.is_filtering || !app.filter_query.is_empty() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(4)])
            .split(area);
        (chunks[1], Some(chunks[0]))
    } else {
        (area, None)
    };

    if let Some(s_area) = search_area {
        let cursor_char = if app.is_filtering && (app.tick / 2) % 2 == 0 {
            "█"
        } else {
            " "
        };
        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if app.is_filtering {
                Style::default().fg(DUCK_YELLOW).bold()
            } else {
                Style::default().fg(TEXT_DIM)
            })
            .title(Span::styled(
                " 🔍 Filtrar Dispositivos [/] ",
                Style::default().fg(DUCK_YELLOW),
            ));

        let search_text = Line::from(vec![
            Span::styled(" Buscar: ", Style::default().fg(TEXT_MUTED)),
            Span::styled(&app.filter_query, Style::default().fg(TEXT_MAIN).bold()),
            Span::styled(cursor_char, Style::default().fg(DUCK_YELLOW)),
            Span::styled(
                format!("  ({} de {} dispositivos)", filtered.len(), app.peers.len()),
                Style::default().fg(TEXT_DIM),
            ),
        ]);
        f.render_widget(Paragraph::new(search_text).block(search_block), s_area);
    }

    if filtered.is_empty() {
        let not_found = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("Nenhum dispositivo encontrado para '{}'", app.filter_query),
                Style::default().fg(TEXT_MUTED),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Pressione [Esc] para limpar o filtro.",
                Style::default().fg(TEXT_DIM),
            )),
        ])
        .alignment(Alignment::Center)
        .block(block);
        f.render_widget(not_found, list_area);
        return;
    }

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|(original_idx, peer)| {
            let is_selected = *original_idx == app.selected_peer_idx;

            let icon = match peer.info.device_type {
                Some(ducker_core::DeviceType::Mobile) => "📱",
                Some(ducker_core::DeviceType::Headless | ducker_core::DeviceType::Server) => "🍓",
                _ => "💻",
            };

            let prefix = if is_selected { "► " } else { "  " };

            let model = peer.info.device_model.as_deref().unwrap_or("Dispositivo");
            let protocol_badge = if peer.protocol == ducker_core::Protocol::Https {
                "TLS"
            } else {
                "HTTP"
            };

            let line1 = Line::from(vec![
                Span::styled(prefix, Style::default().fg(DUCK_YELLOW).bold()),
                Span::raw(format!("{} ", icon)),
                Span::styled(
                    &peer.info.alias,
                    if is_selected {
                        Style::default().fg(DUCK_YELLOW).bold()
                    } else {
                        Style::default().fg(TEXT_MAIN).bold()
                    },
                ),
                Span::raw(" "),
                Span::styled(format!("({})", model), Style::default().fg(TEXT_DIM)),
            ]);

            let short_fp = if peer.info.fingerprint.len() >= 8 {
                &peer.info.fingerprint[..8]
            } else {
                "n/a"
            };

            let line2 = Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    format!("{}:{}", peer.ip, peer.port),
                    Style::default().fg(TEXT_MUTED),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("[{}]", protocol_badge),
                    Style::default().fg(INFO_BLUE),
                ),
                Span::raw("  "),
                Span::styled(format!("fp:{}", short_fp), Style::default().fg(TEXT_DIM)),
            ]);

            let item = ListItem::new(vec![line1, line2, Line::from("")]);
            if is_selected {
                item.style(Style::default().bg(BG_CARD_HIGHLIGHT))
            } else {
                item
            }
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, list_area);
}

fn render_active_transfer(f: &mut Frame, area: Rect, app: &TuiApp) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(TEXT_DIM))
        .title(" Transferência Ativa ");

    if let Some(ref t) = app.active_transfer {
        let inner = block.inner(area);
        f.render_widget(block, area);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner);

        let dir_badge = if t.is_incoming {
            Span::styled(" [RECEBENDO] ", Style::default().fg(SUCCESS_GREEN).bold())
        } else {
            Span::styled(" [ENVIANDO] ", Style::default().fg(DUCK_YELLOW).bold())
        };

        let title_line = Line::from(vec![
            dir_badge,
            Span::styled(&t.file_name, Style::default().fg(TEXT_MAIN).bold()),
        ]);

        let gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(Style::default().fg(DUCK_YELLOW).bg(BG_CARD))
            .percent(t.pct as u16)
            .label(format!("{}%", t.pct));

        let mb_transferred = (t.transferred_bytes as f64) / 1_048_576.0;
        let mb_total = (t.total_bytes as f64) / 1_048_576.0;
        let meta_line = Line::from(vec![
            Span::styled(
                format!("{:.2} MB / {:.2} MB", mb_transferred, mb_total),
                Style::default().fg(TEXT_MUTED),
            ),
            Span::raw("  •  "),
            Span::styled(
                format!("{:.1} MB/s", t.speed_mbps),
                Style::default().fg(INFO_BLUE).bold(),
            ),
        ]);

        f.render_widget(Paragraph::new(title_line), rows[0]);
        f.render_widget(gauge, rows[1]);
        f.render_widget(Paragraph::new(meta_line), rows[2]);
    } else {
        let p = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("● ", Style::default().fg(SUCCESS_GREEN)),
                Span::styled(
                    "Nenhuma transferência ativa no momento.",
                    Style::default().fg(TEXT_MUTED),
                ),
            ]),
        ])
        .block(block)
        .alignment(Alignment::Center);
        f.render_widget(p, area);
    }
}

fn render_activity_log(f: &mut Frame, area: Rect, app: &TuiApp) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(TEXT_DIM))
        .title(" Atividade & Mensagens ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    let max_lines = inner.height as usize;
    let items_to_show: Vec<ListItem> = app
        .logs
        .iter()
        .take(max_lines)
        .map(|entry| {
            let (prefix_char, style) = match entry.kind {
                LogKind::Success => ("✓", Style::default().fg(SUCCESS_GREEN).bold()),
                LogKind::Info => ("ℹ", Style::default().fg(INFO_BLUE)),
                LogKind::Warning => ("⚠", Style::default().fg(DUCK_YELLOW)),
                LogKind::Error => ("✗", Style::default().fg(ERROR_RED).bold()),
                LogKind::Message => ("💬", Style::default().fg(WHATSAPP_GREEN).bold()),
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", entry.timestamp),
                    Style::default().fg(TEXT_DIM),
                ),
                Span::styled(format!("{} ", prefix_char), style),
                Span::styled(&entry.text, Style::default().fg(TEXT_MAIN)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items_to_show);
    f.render_widget(list, inner);
}

fn render_duckive_strip(f: &mut Frame, area: Rect, app: &TuiApp) {
    let view = app.duckive.to_view();
    let lvl = view.level.level;
    let title = view.level.title;
    let next_reward = view
        .level
        .next_reward_item
        .map(|i| format!("{} {}", i.icon_emoji, i.name))
        .unwrap_or_else(|| "Nenhum".to_string());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DUCK_YELLOW_DARK));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let line = Line::from(vec![
        Span::styled(" 🦆 Duckive: ", Style::default().fg(DUCK_YELLOW).bold()),
        Span::styled(
            format!("Nv. {} ({})", lvl, title),
            Style::default().fg(TEXT_MAIN).bold(),
        ),
        Span::styled("  │  ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            format!("Próximo: {} ", next_reward),
            Style::default().fg(TEXT_MUTED),
        ),
        Span::styled(
            format!("({} arq)", view.files_sent_count),
            Style::default().fg(INFO_BLUE),
        ),
        Span::styled("  │  ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            "50.000 arq: Pato Verde WA 🟢",
            Style::default().fg(WHATSAPP_GREEN).bold(),
        ),
    ]);

    f.render_widget(Paragraph::new(line), inner);
}

fn render_duckive_view(f: &mut Frame, area: Rect, app: &TuiApp) {
    let view = app.duckive.to_view();

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);

    // Painel Esquerdo: Perfil e Estatísticas
    let left_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DUCK_YELLOW))
        .title(" Perfil Duckive ");

    let inner_left = left_block.inner(cols[0]);
    f.render_widget(left_block, cols[0]);

    let mascot_art = r#"
         __
       <(o )___
        ( ._> /
         `---'
    "#;

    let total_mb_sent = (view.total_bytes_sent as f64) / 1_048_576.0;
    let total_mb_rec = (view.total_bytes_received as f64) / 1_048_576.0;

    let left_text = vec![
        Line::from(Span::styled(
            mascot_art,
            Style::default().fg(DUCK_YELLOW).bold(),
        )),
        Line::from(vec![
            Span::styled(" Nome do Mascote: ", Style::default().fg(TEXT_MUTED)),
            Span::styled(&view.name, Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled("  [n: renomear]", Style::default().fg(TEXT_DIM)),
        ]),
        Line::from(vec![
            Span::styled(" Nível Atual:     ", Style::default().fg(TEXT_MUTED)),
            Span::styled(
                format!("Nível {} • {}", view.level.level, view.level.title),
                Style::default().fg(SUCCESS_GREEN).bold(),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            " ── Estatísticas de Rede ──",
            Style::default().fg(TEXT_DIM),
        )),
        Line::from(vec![
            Span::styled(" Arquivos Enviados:    ", Style::default().fg(TEXT_MUTED)),
            Span::styled(
                format!(
                    "{} arquivos ({:.2} MB)",
                    view.files_sent_count, total_mb_sent
                ),
                Style::default().fg(TEXT_MAIN),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Arquivos Recebidos:   ", Style::default().fg(TEXT_MUTED)),
            Span::styled(
                format!(
                    "{} arquivos ({:.2} MB)",
                    view.files_received_count, total_mb_rec
                ),
                Style::default().fg(TEXT_MAIN),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Transferências Reais: ", Style::default().fg(TEXT_MUTED)),
            Span::styled(
                format!("{} sessões concluídas", view.transfers_count),
                Style::default().fg(TEXT_MAIN),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " 🟢 Meta Lendária WhatsApp: ",
                Style::default().fg(WHATSAPP_GREEN).bold(),
            ),
            Span::styled(
                format!("{}/50.000 arquivos", view.files_sent_count),
                Style::default().fg(TEXT_MAIN),
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(left_text), inner_left);

    // Painel Direito: Cosméticos & Conquistas
    let right_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(TEXT_DIM))
        .title(" Conquistas & Cosméticos ");

    let inner_right = right_block.inner(cols[1]);
    f.render_widget(right_block, cols[1]);

    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner_right);

    // Itens Cosméticos
    let items_list: Vec<ListItem> = view
        .items
        .iter()
        .enumerate()
        .map(|(idx, it)| {
            let is_selected = idx == app.selected_cosmetic_idx;
            let prefix = if is_selected { "► " } else { "  " };

            let status = if it.equipped {
                Span::styled(" [EQUIPADO] ", Style::default().fg(SUCCESS_GREEN).bold())
            } else if it.unlocked {
                Span::styled(" [LIBERADO] ", Style::default().fg(DUCK_YELLOW))
            } else {
                Span::styled(
                    format!(" [{}%] ", it.progress_pct),
                    Style::default().fg(TEXT_DIM),
                )
            };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(DUCK_YELLOW).bold()),
                Span::raw(format!("{} ", it.item.icon_emoji)),
                Span::styled(
                    &it.item.name,
                    if is_selected {
                        Style::default().fg(DUCK_YELLOW).bold()
                    } else {
                        Style::default().fg(TEXT_MAIN).bold()
                    },
                ),
                status,
                Span::styled(
                    format!("({})", it.item.description),
                    Style::default().fg(TEXT_DIM),
                ),
            ]);

            let item = ListItem::new(line);
            if is_selected {
                item.style(Style::default().bg(BG_CARD_HIGHLIGHT))
            } else {
                item
            }
        })
        .collect();

    let cosmetics_widget = List::new(items_list).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(TEXT_DIM))
            .title(" Cosméticos [↑/↓ Selecionar • Enter Equipar] "),
    );
    f.render_widget(cosmetics_widget, right_rows[0]);

    // Conquistas
    let ach_list: Vec<ListItem> = view
        .achievements
        .iter()
        .map(|ach| {
            let badge = if ach.unlocked {
                Span::styled(" ✓ CONCLUÍDO ", Style::default().fg(SUCCESS_GREEN).bold())
            } else {
                Span::styled(" 🔒 BLOQUEADO ", Style::default().fg(TEXT_DIM))
            };

            Line::from(vec![
                Span::raw(format!("{} ", ach.icon_emoji)),
                Span::styled(&ach.title, Style::default().fg(TEXT_MAIN).bold()),
                badge,
                Span::styled(
                    format!("- {}", ach.description),
                    Style::default().fg(TEXT_MUTED),
                ),
            ])
            .into()
        })
        .collect();

    let ach_widget =
        List::new(ach_list).block(Block::default().title(" Conquistas Desbloqueáveis "));
    f.render_widget(ach_widget, right_rows[1]);
}

fn render_help_view(f: &mut Frame, area: Rect, _app: &TuiApp) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DUCK_YELLOW))
        .title(" Ajuda & Atalhos ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    let help_text = vec![
        Line::from(Span::styled(" Atalhos e Navegação do Terminal Ducker", Style::default().fg(DUCK_YELLOW).bold())),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [↑] ou [k]        ", Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled("Navega na lista de dispositivos ou cosméticos", Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("  [↓] ou [j]        ", Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled("Navega na lista de dispositivos ou cosméticos", Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("  [/]               ", Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled("Filtrar dispositivos por nome, IP ou modelo", Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("  [s]               ", Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled("Enviar arquivo ou pasta para o dispositivo selecionado", Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("  [Tab] no envio    ", Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled("Autocompletar nomes de arquivos e pastas no modal de envio", Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("  [t]               ", Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled("Enviar mensagem de texto direta (chat P2P seguro)", Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("  [r]               ", Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled("Atualizar radar e forçar busca ativa de dispositivos na rede", Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("  [Enter] / [e]     ", Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled("Equipar / desequipar cosmético na aba Duckive", Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("  [n]               ", Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled("Renomear mascote na aba Duckive", Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("  [d]               ", Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled("Abrir aba do Duckive Companion e ver cosméticos", Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("  [Tab]             ", Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled("Alternar entre abas (Radar / Duckive / Ajuda)", Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("  [1, 2, 3]         ", Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled("Ir diretamente para a aba correspondente", Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("  [Esc]             ", Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled("Fechar janelas modais, limpar filtros ou cancelar ações", Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled("  [q] ou [Ctrl+C]   ", Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled("Sair do Ducker e restaurar o terminal", Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(""),
        Line::from(Span::styled(" Compatibilidade LocalSend v2.1", Style::default().fg(INFO_BLUE).bold())),
        Line::from(Span::styled("  O Ducker opera em conformidade total com o protocolo aberto LocalSend v2.", Style::default().fg(TEXT_MUTED))),
        Line::from(Span::styled("  Você pode trocar arquivos livremente com smartphones Android, iPhones, iPads e computadores.", Style::default().fg(TEXT_MUTED))),
    ];

    f.render_widget(Paragraph::new(help_text).wrap(Wrap { trim: false }), inner);
}

fn render_footer(f: &mut Frame, area: Rect, app: &TuiApp) {
    let status_span = if let Some((ref msg, time)) = app.status_message {
        if time.elapsed().as_secs() < 4 {
            Span::styled(
                format!(" [ {} ] ", msg),
                Style::default().fg(SUCCESS_GREEN).bold(),
            )
        } else {
            Span::raw("")
        }
    } else {
        Span::raw("")
    };

    let footer_spans = if app.is_filtering {
        vec![
            Span::styled(" [Busca] ", Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled("Digite para filtrar  ", Style::default().fg(TEXT_MAIN)),
            Span::styled("[↑/↓] Navegar  ", Style::default().fg(TEXT_DIM)),
            Span::styled("[Enter] Fixar  ", Style::default().fg(INFO_BLUE)),
            Span::styled("[Esc] Limpar e fechar", Style::default().fg(ERROR_RED)),
            status_span,
        ]
    } else {
        match app.active_tab {
            ActiveTab::Radar => vec![
                Span::styled(" [↑/↓] Navegar ", Style::default().fg(TEXT_DIM)),
                Span::styled("[/] Filtrar ", Style::default().fg(INFO_BLUE)),
                Span::styled("[s] Enviar ", Style::default().fg(DUCK_YELLOW).bold()),
                Span::styled("[t] Texto ", Style::default().fg(DUCK_YELLOW).bold()),
                Span::styled("[r] Buscar ", Style::default().fg(INFO_BLUE)),
                Span::styled("[d] Duckive ", Style::default().fg(DUCK_YELLOW_DARK)),
                Span::styled("[Tab] Abas ", Style::default().fg(TEXT_DIM)),
                Span::styled("[?] Ajuda ", Style::default().fg(TEXT_DIM)),
                Span::styled("[q] Sair ", Style::default().fg(ERROR_RED)),
                status_span,
            ],
            ActiveTab::Duckive => vec![
                Span::styled(" [↑/↓] Navegar cosméticos ", Style::default().fg(TEXT_DIM)),
                Span::styled(
                    "[Enter/e] Equipar ",
                    Style::default().fg(SUCCESS_GREEN).bold(),
                ),
                Span::styled(
                    "[n] Renomear mascote ",
                    Style::default().fg(DUCK_YELLOW).bold(),
                ),
                Span::styled("[Tab] Abas ", Style::default().fg(TEXT_DIM)),
                Span::styled("[1] Radar ", Style::default().fg(TEXT_MUTED)),
                Span::styled("[q] Sair ", Style::default().fg(ERROR_RED)),
                status_span,
            ],
            ActiveTab::Help => vec![
                Span::styled(" [1] Radar ", Style::default().fg(DUCK_YELLOW).bold()),
                Span::styled("[2] Duckive ", Style::default().fg(DUCK_YELLOW_DARK)),
                Span::styled("[Tab] Próxima Aba ", Style::default().fg(TEXT_DIM)),
                Span::styled("[q] Sair ", Style::default().fg(ERROR_RED)),
                status_span,
            ],
        }
    };

    f.render_widget(Paragraph::new(Line::from(footer_spans)), area);
}

fn render_incoming_modal(
    f: &mut Frame,
    area: Rect,
    sender_name: &str,
    files_summary: &str,
    total_size: u64,
) {
    let modal_area = centered_rect(60, 40, area);
    f.render_widget(Clear, modal_area);

    let mb = (total_size as f64) / 1_048_576.0;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(DUCK_YELLOW).bold())
        .title(" 🔔 Pedido de Transferência Recebido ");

    let inner = block.inner(modal_area);
    f.render_widget(block, modal_area);

    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" Remetente: ", Style::default().fg(TEXT_MUTED)),
            Span::styled(sender_name, Style::default().fg(DUCK_YELLOW).bold()),
        ]),
        Line::from(vec![
            Span::styled(" Arquivos:  ", Style::default().fg(TEXT_MUTED)),
            Span::styled(files_summary, Style::default().fg(TEXT_MAIN)),
        ]),
        Line::from(vec![
            Span::styled(" Tamanho:   ", Style::default().fg(TEXT_MUTED)),
            Span::styled(
                format!("{:.2} MB", mb),
                Style::default().fg(INFO_BLUE).bold(),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Deseja aceitar o recebimento dos arquivos?",
            Style::default().fg(TEXT_MAIN).bold(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [Y] ACEITAR   ",
                Style::default()
                    .fg(SUCCESS_GREEN)
                    .bold()
                    .add_modifier(Modifier::REVERSED),
            ),
            Span::raw("     "),
            Span::styled(
                "  [N] RECUSAR   ",
                Style::default()
                    .fg(ERROR_RED)
                    .bold()
                    .add_modifier(Modifier::REVERSED),
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(content).alignment(Alignment::Center), inner);
}

fn render_input_modal(
    f: &mut Frame,
    area: Rect,
    title: &str,
    prompt: &str,
    value: &str,
    target_alias: &str,
    tick: u64,
) {
    let modal_area = centered_rect(65, 35, area);
    f.render_widget(Clear, modal_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DUCK_YELLOW).bold())
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(DUCK_YELLOW).bold(),
        ));

    let inner = block.inner(modal_area);
    f.render_widget(block, modal_area);

    let cursor = if (tick / 2) % 2 == 0 { "█" } else { " " };

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" Destinatário: ", Style::default().fg(TEXT_MUTED)),
            Span::styled(target_alias, Style::default().fg(INFO_BLUE).bold()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!(" {}:", prompt),
            Style::default().fg(TEXT_MAIN),
        )),
        Line::from(vec![
            Span::styled(" > ", Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled(value, Style::default().fg(TEXT_MAIN).bold()),
            Span::styled(cursor, Style::default().fg(DUCK_YELLOW)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " [Enter] Confirmar   [Esc] Cancelar",
            Style::default().fg(TEXT_DIM),
        )]),
    ];

    f.render_widget(Paragraph::new(text), inner);
}

fn render_rename_modal(f: &mut Frame, area: Rect, value: &str, tick: u64) {
    let modal_area = centered_rect(60, 30, area);
    f.render_widget(Clear, modal_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DUCK_YELLOW).bold())
        .title(Span::styled(
            " 🦆 Renomear Mascote Duckive ",
            Style::default().fg(DUCK_YELLOW).bold(),
        ));

    let inner = block.inner(modal_area);
    f.render_widget(block, modal_area);

    let cursor = if (tick / 2) % 2 == 0 { "█" } else { " " };

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Digite o novo nome para o seu companheiro:",
            Style::default().fg(TEXT_MAIN),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(" > ", Style::default().fg(DUCK_YELLOW).bold()),
            Span::styled(value, Style::default().fg(TEXT_MAIN).bold()),
            Span::styled(cursor, Style::default().fg(DUCK_YELLOW)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " [Enter] Salvar Nome   [Esc] Cancelar",
            Style::default().fg(TEXT_DIM),
        )]),
    ];

    f.render_widget(Paragraph::new(text).alignment(Alignment::Center), inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
