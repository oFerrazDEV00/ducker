use crate::ui::{create_transfer_progress_bar, ERROR, SEARCH, SUCCESS};
use console::style;
use dialoguer::Select;
use ducker_core::{default_save_dir, Identity, Node, NodeConfig, Peer, SendOptions};
use std::path::PathBuf;
use std::time::Duration;

pub async fn execute(
    identity: &Identity,
    paths: Vec<PathBuf>,
    to_query: Option<String>,
    pin: Option<String>,
) {
    if paths.is_empty() {
        eprintln!("{} Nenhum arquivo ou pasta informado para envio.", ERROR);
        return;
    }

    for p in &paths {
        if !p.exists() {
            eprintln!("{} Caminho não encontrado: {}", ERROR, p.display());
            return;
        }
    }

    let mut config = NodeConfig::new(identity.clone(), default_save_dir());
    config.port = 0;
    config.enable_discovery = true;

    let (node, _events) = match Node::start(config).await {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{} Falha ao iniciar cliente de envio: {}", ERROR, e);
            return;
        }
    };

    println!(
        "\n{} {}",
        SEARCH,
        style("Localizando destinatário...").bold()
    );

    // Se o usuário passou um IP ou IP:Porta direto
    let target_peer: Peer = if let Some(ref q) = to_query {
        // Tentar parsing de SocketAddr ou IP
        if let Ok(addr) = q.parse::<std::net::SocketAddr>() {
            match node.connect(addr.ip(), addr.port()).await {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{} Falha ao conectar ao endereço {}: {}", ERROR, q, e);
                    node.shutdown();
                    return;
                }
            }
        } else if let Ok(ip) = q.parse::<std::net::IpAddr>() {
            match node.connect(ip, ducker_core::DEFAULT_PORT).await {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{} Falha ao conectar ao IP {}: {}", ERROR, q, e);
                    node.shutdown();
                    return;
                }
            }
        } else {
            // Aguardar descoberta por multicast
            tokio::time::sleep(Duration::from_millis(1500)).await;
            match node.find_peer(q) {
                Some(p) => p,
                None => {
                    // Tentar scan na sub-rede
                    println!("{}", style("Dispositivo não encontrado no anúncio inicial; tentando scan na rede...").dim());
                    node.scan_subnet().await;
                    match node.find_peer(q) {
                        Some(p) => p,
                        None => {
                            eprintln!(
                                "{} Destinatário '{}' não encontrado na rede local.",
                                ERROR, q
                            );
                            node.shutdown();
                            return;
                        }
                    }
                }
            }
        }
    } else {
        // Sem --to, fazer descoberta interativa
        tokio::time::sleep(Duration::from_millis(1800)).await;
        let mut peers = node.peers();
        if peers.is_empty() {
            println!(
                "{}",
                style("Nenhum dispositivo anunciou via multicast; executando scan rápido...").dim()
            );
            node.scan_subnet().await;
            peers = node.peers();
        }

        if peers.is_empty() {
            eprintln!(
                "\n{} Nenhum dispositivo Ducker ou LocalSend encontrado na rede.",
                ERROR
            );
            eprintln!(
                "Certifique-se de que o destinatário está na mesma rede e executando o aplicativo."
            );
            node.shutdown();
            return;
        }

        let items: Vec<String> = peers
            .iter()
            .map(|p| {
                let quac = p
                    .info
                    .quac_id
                    .map(|q| format!(" [Quac: {}]", q))
                    .unwrap_or_default();
                format!(
                    "{} {}{}",
                    p.info.alias,
                    quac,
                    format!(" ({}:{})", p.ip, p.port)
                )
            })
            .collect();

        let selection = match Select::new()
            .with_prompt("Escolha o destinatário")
            .items(&items)
            .default(0)
            .interact()
        {
            Ok(idx) => idx,
            Err(_) => {
                println!("Envio cancelado.");
                node.shutdown();
                return;
            }
        };

        peers[selection].clone()
    };

    println!(
        "\nEnviando para {} ({}://{}:{})...",
        style(&target_peer.info.alias).cyan().bold(),
        target_peer.protocol.scheme(),
        target_peer.ip,
        target_peer.port
    );

    let pb = create_transfer_progress_bar(0, "arquivos");
    let pb_clone = pb.clone();

    let opts = SendOptions {
        pin,
        expected_quac: target_peer.info.quac_id,
    };

    let start_time = std::time::Instant::now();
    let total_bytes_sent = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let total_bytes_clone = total_bytes_sent.clone();

    let result = node
        .send_files(&target_peer, paths.clone(), opts, move |progress| {
            if pb_clone.length().unwrap_or(0) != progress.total {
                pb_clone.set_length(progress.total);
            }
            total_bytes_clone.store(progress.total, std::sync::atomic::Ordering::Relaxed);
            pb_clone.set_position(progress.total_sent);
            pb_clone.set_message(format!("Enviando {}", progress.file_name));
        })
        .await;

    pb.finish_and_clear();
    node.shutdown();

    match result {
        Ok(()) => {
            println!(
                "\n{} {}",
                SUCCESS,
                style("Transferência concluída com sucesso!").green().bold()
            );

            // Atualiza estatísticas do Duckive
            let config_dir = Identity::default_config_dir().unwrap_or_else(|_| PathBuf::from("."));
            if let Ok(mut profile) =
                ducker_core::DuckiveProfile::load_or_create(&config_dir, &identity.alias)
            {
                let bytes = total_bytes_sent.load(std::sync::atomic::Ordering::Relaxed);
                let duration_sec = start_time.elapsed().as_secs_f64().max(0.01);
                let speed_mbps = (bytes as f64 / 1_048_576.0) / duration_sec;

                // Verifica se a plataforma do destino difere da local
                let my_os = std::env::consts::OS.to_lowercase();
                let peer_model = target_peer
                    .info
                    .device_model
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase();
                let is_cross_platform = !peer_model.is_empty() && !peer_model.contains(&my_os);

                let (real_files_count, real_bytes) = ducker_core::count_files_and_bytes(&paths)
                    .unwrap_or((paths.len() as u64, bytes));
                let actual_bytes = if bytes > 0 { bytes } else { real_bytes };
                let summary = profile.record_transfer(
                    real_files_count,
                    actual_bytes,
                    is_cross_platform,
                    speed_mbps,
                    false,
                );
                let _ = profile.save(&config_dir);

                for item in summary.newly_unlocked_items {
                    println!(
                        "\n{} {} Novo Cosmético Desbloqueado: {}!",
                        crate::ui::DUCK,
                        style("NOVO ITEM!").yellow().bold(),
                        style(&item.name).cyan().bold()
                    );
                }
                for ach in summary.newly_unlocked_achievements {
                    println!(
                        "🏆 {} Conquista Desbloqueada: {}!",
                        style("CONQUISTA!").yellow().bold(),
                        style(&ach.title).green().bold()
                    );
                }
                if summary.level_up {
                    println!(
                        "⭐ {} Seu Duckive subiu para o Nível {}!",
                        style("LEVEL UP!").yellow().bold(),
                        style(summary.current_level).bold()
                    );
                }
            }
        }
        Err(e) => {
            eprintln!(
                "\n{} Falha no envio: {}",
                ERROR,
                style(e.to_string()).red().bold()
            );
        }
    }
}
