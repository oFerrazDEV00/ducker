use std::time::Duration;
use console::style;
use ducker_core::{default_save_dir, Identity, Node, NodeConfig};
use crate::ui::SEARCH;

pub async fn execute(identity: &Identity, run_scan: bool) {
    println!("\n{} {}", SEARCH, style("Buscando dispositivos na rede local (LocalSend / Ducker)...").bold());

    let mut config = NodeConfig::new(identity.clone(), default_save_dir());
    config.port = 0; // porta efêmera para listagem rápida
    config.enable_discovery = true;

    let (node, _events) = match Node::start(config).await {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Erro ao iniciar busca: {}", e);
            return;
        }
    };

    // Aguardar anúncios e respostas multicast
    tokio::time::sleep(Duration::from_millis(1500)).await;

    if run_scan {
        println!("{}", style("Executando scan na sub-rede local...").dim());
        node.scan_subnet().await;
    }

    let peers = node.peers();
    node.shutdown();

    if peers.is_empty() {
        println!("\nNenhum outro dispositivo encontrado no momento.");
        println!("{}", style("Dica: Certifique-se de que os outros dispositivos estão na mesma rede Wi-Fi/LAN com o Ducker ou LocalSend aberto.").dim());
        if !run_scan {
            println!("{}", style("Você pode tentar buscar forçando scan com: ducker devices --scan").cyan());
        }
    } else {
        println!("\n{}", style("Dispositivos encontrados:").bold());
        for (idx, peer) in peers.iter().enumerate() {
            let model = peer.info.device_model.as_deref().unwrap_or("Dispositivo");
            let dev_type = peer.info.device_type.map(|t| format!("{:?}", t)).unwrap_or_else(|| "Desktop".into());
            let quac_str = if let Some(quac) = peer.info.quac_id {
                format!(" [Quac: {}]", style(quac).green().bold())
            } else {
                String::new()
            };

            let fp_short = if peer.info.fingerprint.len() >= 12 {
                format!(" (fp: {}...)", &peer.info.fingerprint[..8])
            } else {
                String::new()
            };

            println!(
                "\n{}. {}{} — {} ({})",
                style(idx + 1).bold(),
                style(&peer.info.alias).cyan().bold(),
                quac_str,
                model,
                dev_type
            );
            println!(
                "   Endereço: {}://{}:{}{}",
                peer.protocol.scheme(),
                peer.ip,
                peer.port,
                style(fp_short).dim()
            );
        }
    }
    println!();
}
