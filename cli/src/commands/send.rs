use std::path::PathBuf;
use std::time::Duration;
use console::style;
use dialoguer::Select;
use ducker_core::{DeviceIdentity, DiscoveryManager, FileSender};
use crate::ui::{create_transfer_progress_bar, ERROR, SEARCH, SUCCESS};

pub async fn execute(
    identity: &DeviceIdentity,
    file_path: PathBuf,
    explicit_target: Option<String>,
    explicit_quac_id: Option<u32>,
) {
    if !file_path.exists() {
        eprintln!("{} Arquivo não encontrado: {}", ERROR, file_path.display());
        return;
    }

    let file_name = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("arquivo")
        .to_string();

    let (target_addr, target_quac_id) = if let (Some(addr), Some(quac_id)) = (explicit_target, explicit_quac_id) {
        (addr, quac_id)
    } else {
        println!("\n{} {}", SEARCH, style("Procurando dispositivos para envio...").bold());

        let discovery = DiscoveryManager::new(identity.clone());
        if let Err(e) = discovery.start().await {
            eprintln!("{} Falha ao iniciar descoberta: {}", ERROR, e);
            return;
        }

        tokio::time::sleep(Duration::from_millis(2500)).await;

        let devices = discovery.list_devices().await;
        if devices.is_empty() {
            eprintln!("\n{} Nenhum dispositivo Ducker disponível na rede local.", ERROR);
            eprintln!("Certifique-se de que o destinatário está na mesma rede e executando o Ducker.");
            return;
        }

        println!("\n{}", style("Para quem enviar?").bold());
        let items: Vec<String> = devices
            .iter()
            .map(|d| format!("{} [{}] ({}:{})", d.device_name, d.quac_id, d.address, d.port))
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
                return;
            }
        };

        let selected = &devices[selection];
        (format!("{}:{}", selected.address, selected.port), selected.quac_id)
    };

    println!("\nIniciando transferência para destino com ID Quac {}...", style(target_quac_id).green().bold());

    let pb = create_transfer_progress_bar(0, &file_name);
    let pb_clone = pb.clone();

    let result = FileSender::send_file(
        &target_addr,
        target_quac_id,
        identity,
        &file_path,
        move |sent, total| {
            if pb_clone.length().unwrap_or(0) != total {
                pb_clone.set_length(total);
            }
            pb_clone.set_position(sent);
        },
    ).await;

    pb.finish_and_clear();

    match result {
        Ok(()) => {
            println!(
                "\n{} {} {}",
                SUCCESS,
                style(&file_name).bold(),
                style("transferido com sucesso!").green().bold()
            );
        }
        Err(err) => {
            eprintln!("\n{} {}", ERROR, style(err.user_friendly_message()).red().bold());
        }
    }
}
