use std::path::PathBuf;
use console::style;
use dialoguer::Confirm;
use ducker_core::{default_save_dir, Identity, Node, NodeConfig, NodeEvent};
use crate::ui::{create_transfer_progress_bar, DUCK, ERROR, SUCCESS};

pub async fn execute(
    identity: &Identity,
    custom_save_dir: Option<PathBuf>,
    port: Option<u16>,
    auto_accept: bool,
    pin: Option<String>,
) {
    let save_dir = custom_save_dir.unwrap_or_else(default_save_dir);
    let _ = std::fs::create_dir_all(&save_dir);

    let mut config = NodeConfig::new(identity.clone(), save_dir.clone());
    if let Some(p) = port {
        config.port = p;
    }
    config.auto_accept = auto_accept;
    config.pin = pin;

    let (node, mut events) = match Node::start(config).await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{} Falha ao iniciar receptor: {}", ERROR, e);
            return;
        }
    };

    println!("\n{} {}", DUCK, style("Ducker está pronto e ouvindo na rede local!").bold().yellow());
    println!("  Dispositivo:  {}", style(&identity.alias).cyan().bold());
    println!("  ID Quac:      {}", style(identity.quac_id).green().bold());
    println!("  Fingerprint:  {}", style(&identity.fingerprint[..12]).dim());
    println!("  Porta:        {} ({})", style(node.port()).bold(), node.config().protocol.scheme());
    println!("  Pasta Destino:{}", style(save_dir.display()).dim());
    if auto_accept {
        println!("  Modo:         {}", style("Aceite automático ATIVADO (--yes)").green());
    } else {
        println!("  Modo:         {}", style("Confirmação interativa no terminal").yellow());
    }
    println!("\n{}", style("Aguardando conexões... Pressione Ctrl+C para encerrar.").dim());

    let mut current_pb: Option<indicatif::ProgressBar> = None;

    while let Ok(event) = events.recv().await {
        match event {
            NodeEvent::IncomingRequest { session_id, sender, files, auto_accepted } => {
                let total_size: u64 = files.iter().map(|f| f.size).sum();
                let count = files.len();
                let file_names: Vec<&str> = files.iter().map(|f| f.file_name.as_str()).collect();

                println!(
                    "\n{} Pedido de transferência de {} [fp: {}]:",
                    style("➤").cyan().bold(),
                    style(&sender.alias).cyan().bold(),
                    style(if sender.fingerprint.len() >= 8 { &sender.fingerprint[..8] } else { "n/a" }).dim()
                );
                println!(
                    "  {} arquivo(s) ({:.2} MB): {}",
                    count,
                    (total_size as f64) / 1024.0 / 1024.0,
                    style(file_names.join(", ")).italic()
                );

                if !auto_accepted {
                    let prompt = format!("Deseja aceitar o recebimento de {}?", sender.alias);
                    let accepted = Confirm::new()
                        .with_prompt(prompt)
                        .default(true)
                        .interact()
                        .unwrap_or(false);

                    if accepted {
                        println!("  Aceitando transferência...");
                        node.respond(&session_id, true);
                    } else {
                        println!("  Transferência recusada.");
                        node.respond(&session_id, false);
                    }
                }
            }
            NodeEvent::ReceiveProgress { file_id: _, received, total, .. } => {
                let pb = current_pb.get_or_insert_with(|| create_transfer_progress_bar(total, "arquivo"));
                pb.set_length(total);
                pb.set_position(received);
            }
            NodeEvent::FileReceived { file_name, path, .. } => {
                if let Some(pb) = current_pb.take() {
                    pb.finish_and_clear();
                }
                println!(
                    "{} Arquivo '{}' salvo com sucesso em:\n  {}",
                    SUCCESS,
                    style(&file_name).bold(),
                    style(path).dim()
                );
            }
            NodeEvent::TextReceived { sender, text } => {
                println!(
                    "\n{} Mensagem recebida de {}:\n  {}",
                    style("✉").yellow().bold(),
                    style(&sender.alias).cyan().bold(),
                    style(text).white().bold()
                );
            }
            NodeEvent::SessionFinished { session_id: _ } => {
                if let Some(pb) = current_pb.take() {
                    pb.finish_and_clear();
                }
                println!("\n{} Transferência concluída!\n", SUCCESS);
            }
            NodeEvent::SessionCancelled { session_id: _ } => {
                if let Some(pb) = current_pb.take() {
                    pb.finish_and_clear();
                }
                println!("\n{} Sessão cancelada ou recusada.\n", style("!").yellow());
            }
            NodeEvent::PeerDiscovered { .. } => {}
        }
    }
}
