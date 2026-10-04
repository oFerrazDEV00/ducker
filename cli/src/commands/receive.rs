use std::path::PathBuf;
use console::style;
use ducker_core::{
    DeviceIdentity, DiscoveryManager, FileReceiver, TransferEvent, DEFAULT_TRANSFER_PORT,
};
use crate::ui::{create_transfer_progress_bar, DUCK, ERROR, SUCCESS};

pub async fn execute(identity: &DeviceIdentity, custom_save_dir: Option<PathBuf>) {
    let save_dir = custom_save_dir.unwrap_or_else(FileReceiver::default_save_dir);

    println!("\n{} {}", DUCK, style("Ducker está pronto para receber arquivos!").bold().yellow());
    println!("  Dispositivo: {}", style(&identity.device_name).cyan().bold());
    println!("  ID Quac:     {}", style(identity.quac_id).green().bold());
    println!("  Destino:     {}", style(save_dir.display()).dim());
    println!("  Status:      Aguardando conexões na rede local (Porta {})...\n", DEFAULT_TRANSFER_PORT);

    // Iniciar anúncio e escuta de descoberta
    let discovery = DiscoveryManager::new(identity.clone());
    if let Err(e) = discovery.start().await {
        eprintln!("Aviso: Falha ao iniciar serviço de descoberta: {}", e);
    }

    // Iniciar servidor receptor
    let (receiver, mut events) = FileReceiver::new(identity.clone(), save_dir.clone());
    if let Err(e) = receiver.start().await {
        eprintln!("{} Falha ao iniciar servidor receptor: {}", ERROR, e);
        return;
    }

    let mut current_pb: Option<indicatif::ProgressBar> = None;

    while let Ok(event) = events.recv().await {
        match event {
            TransferEvent::IncomingRequest {
                sender_name,
                sender_quac_id,
                destination_quac_id,
                file_name,
                file_size,
            } => {
                println!(
                    "\nRecebendo solicitação de transferência:\n  Arquivo:  {} ({:.2} MB)\n  De:       {} [Quac: {}]\n  Para ID:  {}",
                    style(&file_name).bold(),
                    (file_size as f64) / 1024.0 / 1024.0,
                    style(&sender_name).cyan(),
                    style(sender_quac_id).green(),
                    style(destination_quac_id).yellow()
                );
                current_pb = Some(create_transfer_progress_bar(file_size, &file_name));
            }
            TransferEvent::Progress {
                bytes_received,
                total_bytes,
                ..
            } => {
                if let Some(ref pb) = current_pb {
                    pb.set_length(total_bytes);
                    pb.set_position(bytes_received);
                }
            }
            TransferEvent::Completed { file_name, saved_path } => {
                if let Some(pb) = current_pb.take() {
                    pb.finish_and_clear();
                }
                println!(
                    "{} {} {}",
                    SUCCESS,
                    style(&file_name).bold(),
                    style("recebido com sucesso!").green().bold()
                );
                println!("  Salvo em: {}\n", style(saved_path.display()).dim());
            }
            TransferEvent::Rejected { reason, message } => {
                if let Some(pb) = current_pb.take() {
                    pb.finish_and_clear();
                }
                eprintln!(
                    "\n{} Transferência rejeitada [{}]:\n  {}",
                    ERROR,
                    style(reason.as_str()).red().bold(),
                    style(message).red()
                );
            }
            TransferEvent::Failed { file_name, error } => {
                if let Some(pb) = current_pb.take() {
                    pb.finish_and_clear();
                }
                eprintln!("\n{} Falha ao receber {}: {}", ERROR, file_name, error);
            }
        }
    }
}
