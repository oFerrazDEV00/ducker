use std::time::Duration;
use console::style;
use dialoguer::Select;
use ducker_core::{default_save_dir, Identity, Node, NodeConfig, Peer, SendOptions};
use crate::ui::{ERROR, SEARCH, SUCCESS};

pub async fn execute(
    identity: &Identity,
    text: String,
    to_query: Option<String>,
) {
    if text.trim().is_empty() {
        eprintln!("{} Mensagem de texto não pode ser vazia.", ERROR);
        return;
    }

    let mut config = NodeConfig::new(identity.clone(), default_save_dir());
    config.port = 0;
    config.enable_discovery = true;

    let (node, _events) = match Node::start(config).await {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{} Falha ao iniciar cliente: {}", ERROR, e);
            return;
        }
    };

    println!("\n{} {}", SEARCH, style("Localizando destinatário para mensagem...").bold());

    let target_peer: Peer = if let Some(ref q) = to_query {
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
            tokio::time::sleep(Duration::from_millis(1500)).await;
            match node.find_peer(q) {
                Some(p) => p,
                None => {
                    node.scan_subnet().await;
                    match node.find_peer(q) {
                        Some(p) => p,
                        None => {
                            eprintln!("{} Destinatário '{}' não encontrado na rede local.", ERROR, q);
                            node.shutdown();
                            return;
                        }
                    }
                }
            }
        }
    } else {
        tokio::time::sleep(Duration::from_millis(1800)).await;
        let mut peers = node.peers();
        if peers.is_empty() {
            node.scan_subnet().await;
            peers = node.peers();
        }

        if peers.is_empty() {
            eprintln!("\n{} Nenhum dispositivo encontrado na rede local.", ERROR);
            node.shutdown();
            return;
        }

        let items: Vec<String> = peers
            .iter()
            .map(|p| {
                let quac = p.info.quac_id.map(|q| format!(" [Quac: {}]", q)).unwrap_or_default();
                format!("{} {}{}", p.info.alias, quac, format!(" ({}:{})", p.ip, p.port))
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
        "\nEnviando texto para {}...",
        style(&target_peer.info.alias).cyan().bold()
    );

    let opts = SendOptions {
        pin: None,
        expected_quac: target_peer.info.quac_id,
    };

    let result = node.send_text(&target_peer, &text, opts).await;
    node.shutdown();

    match result {
        Ok(()) => {
            println!("\n{} {}", SUCCESS, style("Mensagem enviada com sucesso!").green().bold());
        }
        Err(e) => {
            eprintln!("\n{} Falha no envio da mensagem: {}", ERROR, style(e.to_string()).red().bold());
        }
    }
}
