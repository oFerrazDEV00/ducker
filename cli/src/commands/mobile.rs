use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use console::style;
use dialoguer::Select;
use ducker_core::{
    ConnectedMobile, DeviceIdentity, DiscoveryManager, MobileBridge, MobileSendEvent,
    DEFAULT_MOBILE_BRIDGE_PORT,
};
use crate::ui::{create_transfer_progress_bar, DUCK, ERROR, SUCCESS};

/// Envia um arquivo ou mensagem curta do PC para o app Ducker Mobile (Expo Go / iOS / Android).
///
/// Sobe o Mobile Bridge (HTTP + WebSocket) na porta 7876, aguarda o celular
/// conectar e entrega o arquivo pelo fluxo handshake -> download -> confirmação.
pub async fn execute(
    identity: &DeviceIdentity,
    file_path: Option<PathBuf>,
    msg: Option<String>,
    to: Option<u32>,
) {
    let (file_path, is_temp) = match (file_path, msg) {
        (Some(path), _) => {
            if !path.is_file() {
                eprintln!("{} Arquivo não encontrado: {}", ERROR, path.display());
                return;
            }
            (path, false)
        }
        (None, Some(text)) => {
            let temp_dir = std::env::temp_dir();
            let temp_path = temp_dir.join("mensagem.txt");
            if let Err(e) = std::fs::write(&temp_path, text.as_bytes()) {
                eprintln!("{} Falha ao criar mensagem temporária: {}", ERROR, e);
                return;
            }
            (temp_path, true)
        }
        (None, None) => {
            eprintln!("{} Informe um arquivo ou use a flag --msg \"sua mensagem\".", ERROR);
            eprintln!("Exemplos:");
            eprintln!("  ducker mobile teste.txt");
            eprintln!("  ducker mobile --msg \"Olá do PC!\"");
            return;
        }
    };

    let file_name = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("arquivo")
        .to_string();
    let file_size = std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);

    let discovery = Arc::new(DiscoveryManager::new(identity.clone()));
    let bridge = MobileBridge::new(identity.clone(), discovery);
    if let Err(e) = bridge.start().await {
        eprintln!("\n{} {}", ERROR, style(e.user_friendly_message()).red().bold());
        eprintln!(
            "Dica: feche outros processos do Ducker que estejam usando a porta {}.",
            DEFAULT_MOBILE_BRIDGE_PORT
        );
        return;
    }
    let state = bridge.state();

    let ip = DiscoveryManager::get_local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| "<IP do PC — veja com ipconfig>".to_string());

    println!("\n{} {}", DUCK, style("Envio para o celular").bold().yellow());
    println!("  Arquivo:  {} ({:.2} MB)", style(&file_name).bold(), file_size as f64 / 1024.0 / 1024.0);
    println!("  PC:       {} [Quac: {}]", style(&identity.device_name).cyan(), style(identity.quac_id).green());
    println!();
    println!("  No app Ducker do celular, digite este IP e toque em Conectar:");
    println!("      {}", style(&ip).green().bold());
    println!("  (porta {})", DEFAULT_MOBILE_BRIDGE_PORT);
    println!();
    println!("{}", style("Aguardando o celular conectar... (Ctrl+C para cancelar)").dim());

    let target: ConnectedMobile = loop {
        let mut mobiles = state.connected_mobiles().await;
        if let Some(wanted) = to {
            if let Some(m) = mobiles.into_iter().find(|m| m.quac_id == wanted) {
                break m;
            }
        } else if mobiles.len() == 1 {
            break mobiles.remove(0);
        } else if mobiles.len() > 1 {
            let items: Vec<String> = mobiles
                .iter()
                .map(|m| format!("{} [{}]", m.device_name, m.quac_id))
                .collect();
            match Select::new()
                .with_prompt("Mais de um celular conectado. Para qual enviar?")
                .items(&items)
                .default(0)
                .interact()
            {
                Ok(idx) => break mobiles.remove(idx),
                Err(_) => {
                    println!("Envio cancelado.");
                    return;
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    };

    println!(
        "\n{} Celular conectado: {} [Quac: {}]",
        SUCCESS,
        style(&target.device_name).cyan().bold(),
        style(target.quac_id).green().bold()
    );

    let pb = create_transfer_progress_bar(file_size, &file_name);
    pb.set_message(format!("Aguardando o celular aceitar {}...", file_name));
    let pb_clone = pb.clone();
    let name_clone = file_name.clone();

    let result = state
        .send_to_mobile(target.quac_id, file_path.clone(), move |event| match event {
            MobileSendEvent::WaitingAcceptance => {}
            MobileSendEvent::Accepted => pb_clone.set_message(format!("Enviando {}", name_clone)),
            MobileSendEvent::Progress { sent, total } => {
                if pb_clone.length().unwrap_or(0) != total {
                    pb_clone.set_length(total);
                }
                pb_clone.set_position(sent);
            }
        })
        .await;

    pb.finish_and_clear();

    match result {
        Ok(()) => println!(
            "\n{} {} {}",
            SUCCESS,
            style(&file_name).bold(),
            style("entregue e salvo no celular!").green().bold()
        ),
        Err(err) => eprintln!("\n{} {}", ERROR, style(err.user_friendly_message()).red().bold()),
    }

    // Pequena pausa para as últimas mensagens WS saírem antes de encerrar
    tokio::time::sleep(Duration::from_millis(300)).await;

    if is_temp {
        let _ = std::fs::remove_file(&file_path);
    }
}
