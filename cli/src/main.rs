use std::path::PathBuf;
use clap::{Parser, Subcommand};
use console::style;
use dialoguer::Input;
use ducker_core::DeviceIdentity;

mod commands;
mod ui;

#[derive(Parser)]
#[command(name = "ducker")]
#[command(about = "Ducker — Be simple, be duck. 🦆 Local-first file transfer", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Mostra a identidade do dispositivo e o ID Quac atual
    Id,
    /// Busca e lista os dispositivos Ducker ativos na mesma rede local
    Devices,
    /// Envia um arquivo para outro dispositivo na rede
    Send {
        /// Caminho do arquivo a ser enviado
        file: PathBuf,
        /// (Opcional) Endereço direto no formato IP:PORTA
        #[arg(long)]
        target: Option<String>,
        /// (Opcional) ID Quac específico do destinatário para validação
        #[arg(long)]
        to: Option<u32>,
    },
    /// Inicia o serviço de escuta e recebimento de arquivos
    Receive {
        /// Diretório personalizado para salvar os arquivos recebidos
        #[arg(short, long)]
        save_dir: Option<PathBuf>,
    },
    /// Executa o diagnóstico de rede, portas e permissões do Ducker
    Doctor,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Carregar ou inicializar identidade na primeira execução
    let mut identity = match DeviceIdentity::load_default() {
        Ok(Some(id)) => Some(id),
        Ok(None) => None,
        Err(e) => {
            eprintln!("Aviso ao carregar identidade existente: {}", e);
            None
        }
    };

    // Se não houver identidade e não for apenas um doctor, executa o assistente de primeira execução
    if identity.is_none() && !matches!(cli.command, Some(Commands::Doctor)) {
        ui::print_banner();
        println!("\nBem-vindo ao {}!\n", style("Ducker").yellow().bold());

        let name: String = Input::new()
            .with_prompt("Nome do dispositivo")
            .interact_text()
            .unwrap_or_else(|_| "Meu Dispositivo Ducker".to_string());

        let new_id = DeviceIdentity::new(name.trim());
        if let Err(e) = new_id.save_default() {
            eprintln!("Erro ao salvar identidade: {}", e);
        } else {
            println!("\nSeu ID Quac: {}\n", style(new_id.quac_id).green().bold());
        }
        identity = Some(new_id);
    }

    match cli.command {
        Some(Commands::Id) => {
            if let Some(id) = identity.as_ref() {
                commands::id::execute(id);
            }
        }
        Some(Commands::Devices) => {
            if let Some(id) = identity.as_ref() {
                commands::devices::execute(id).await;
            }
        }
        Some(Commands::Send { file, target, to }) => {
            if let Some(id) = identity.as_ref() {
                commands::send::execute(id, file, target, to).await;
            }
        }
        Some(Commands::Receive { save_dir }) => {
            if let Some(id) = identity.as_ref() {
                commands::receive::execute(id, save_dir).await;
            }
        }
        Some(Commands::Doctor) => {
            commands::doctor::execute(identity.as_ref());
        }
        None => {
            if let Some(id) = identity.as_ref() {
                commands::receive::execute(id, None).await;
            }
        }
    }
}
