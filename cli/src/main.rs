use clap::{Parser, Subcommand};
use console::style;
use dialoguer::Input;
use ducker_core::{default_alias, Identity};
use std::path::PathBuf;

mod commands;
mod tui;
mod ui;

#[derive(Parser)]
#[command(name = "ducker")]
#[command(about = "Ducker — Be simple, be duck. 🦆 Local-first file transfer (LocalSend v2 protocol)", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Mostra a identidade deste dispositivo, ID Quac e fingerprint SHA-256
    Id,
    /// Busca e lista os dispositivos ativos na mesma rede local
    Devices {
        /// Forçar varredura completa da sub-rede local (útil quando multicast estiver desabilitado no Wi-Fi)
        #[arg(long)]
        scan: bool,
    },
    /// Envia arquivos ou pastas para outro dispositivo na rede
    Send {
        /// Caminhos dos arquivos ou pastas a serem enviados
        #[arg(required = true)]
        files: Vec<PathBuf>,
        /// Destinatário: Apelido, ID Quac, prefixo do fingerprint, ou IP:Porta
        #[arg(long)]
        to: Option<String>,
        /// PIN de segurança opcional se exigido pelo destinatário
        #[arg(long)]
        pin: Option<String>,
    },
    /// Envia uma mensagem de texto diretamente para outro dispositivo
    Text {
        /// Texto da mensagem a ser enviada
        message: String,
        /// Destinatário: Apelido, ID Quac, prefixo do fingerprint, ou IP:Porta
        #[arg(long)]
        to: Option<String>,
    },
    /// Inicia o serviço de escuta e recebimento de arquivos (protocolo LocalSend v2)
    Serve {
        /// Diretório personalizado para salvar os arquivos recebidos
        #[arg(short, long)]
        save_dir: Option<PathBuf>,
        /// Porta TCP do servidor (padrão: 53317)
        #[arg(short, long)]
        port: Option<u16>,
        /// Aceitar todas as transferências automaticamente sem confirmação no terminal
        #[arg(short = 'y', long)]
        yes: bool,
        /// Exigir PIN para aceitar transferências
        #[arg(long)]
        pin: Option<String>,
    },
    /// Alias para 'serve': escuta e recebe arquivos na rede local
    Receive {
        /// Diretório personalizado para salvar os arquivos recebidos
        #[arg(short, long)]
        save_dir: Option<PathBuf>,
        /// Porta TCP do servidor (padrão: 53317)
        #[arg(short, long)]
        port: Option<u16>,
        /// Aceitar todas as transferências automaticamente sem confirmação no terminal
        #[arg(short = 'y', long)]
        yes: bool,
        /// Exigir PIN para aceitar transferências
        #[arg(long)]
        pin: Option<String>,
    },
    /// Executa o receptor em segundo plano (você pode fechar o terminal)
    Background,
    /// Encerra processos do Ducker rodando em segundo plano
    Stop,
    /// Configura o Ducker para iniciar com o Windows (enable | disable)
    Autostart {
        /// Ação a ser executada: enable (padrão) ou disable
        #[arg(default_value = "enable")]
        action: String,
    },
    /// Executa o diagnóstico de rede, portas e certificados (Debugger do Pato)
    Doctor {
        /// Exportar relatório de diagnóstico para arquivo Markdown
        #[arg(long)]
        export: Option<PathBuf>,
    },
    /// Alias para 'doctor': diagnóstico do sistema e laboratório de rede
    Debug {
        /// Exportar relatório de diagnóstico para arquivo Markdown
        #[arg(long)]
        export: Option<PathBuf>,
    },
    /// Gerencia e executa automações declarativas no formato .quack
    Quack {
        #[command(subcommand)]
        action: QuackCommands,
    },
    /// Visualiza o seu Duckive, progresso, inventário de cosméticos e conquistas
    Duckive {
        /// Equipar um item cosmético pelo ID
        #[arg(long)]
        equip: Option<String>,
        /// Renomear o seu Duckive
        #[arg(long)]
        name: Option<String>,
    },
    /// Alias para 'duckive': veja seu mascote e conquistas
    Profile {
        /// Equipar um item cosmético pelo ID
        #[arg(long)]
        equip: Option<String>,
        /// Renomear o seu Duckive
        #[arg(long)]
        name: Option<String>,
    },
    /// Abre a interface gráfica desktop (Ducker App)
    Open,
}

#[derive(Subcommand)]
enum QuackCommands {
    /// Executa uma automação declarativa .quack
    Run {
        /// Caminho para o arquivo .quack
        file: PathBuf,
        /// Simular a automação sem transferir dados na rede
        #[arg(long)]
        dry_run: bool,
        /// Executar sem solicitar confirmação interativa
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Valida a sintaxe e os arquivos de uma automação .quack
    Validate {
        /// Caminho para o arquivo .quack
        file: PathBuf,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config_dir = Identity::default_config_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Carregar identidade existente ou criar interativamente
    let mut identity = match Identity::load(&config_dir) {
        Ok(Some(id)) => Some(id),
        Ok(None) => None,
        Err(e) => {
            eprintln!("Aviso ao carregar identidade: {}", e);
            None
        }
    };

    if identity.is_none()
        && !matches!(
            cli.command,
            Some(Commands::Doctor { .. }) | Some(Commands::Debug { .. }) | Some(Commands::Open)
        )
    {
        ui::print_banner();
        println!("\nBem-vindo ao {}!\n", style("Ducker").yellow().bold());

        let def_name = default_alias();
        let name: String = Input::new()
            .with_prompt("Nome / Apelido deste dispositivo na rede")
            .default(def_name)
            .interact_text()
            .unwrap_or_else(|_| "Meu Dispositivo Ducker".to_string());

        match Identity::load_or_create(&config_dir, name.trim()) {
            Ok(new_id) => {
                println!("\nDispositivo configurado com sucesso!");
                println!("  Apelido:     {}", style(&new_id.alias).cyan().bold());
                println!("  ID Quac:     {}", style(new_id.quac_id).green().bold());
                println!("  Fingerprint: {}\n", style(&new_id.fingerprint).yellow());
                identity = Some(new_id);
            }
            Err(e) => {
                eprintln!("Erro ao inicializar identidade: {}", e);
            }
        }
    }

    match cli.command {
        Some(Commands::Id) => {
            if let Some(id) = identity.as_ref() {
                commands::id::execute(id);
            }
        }
        Some(Commands::Devices { scan }) => {
            if let Some(id) = identity.as_ref() {
                commands::devices::execute(id, scan).await;
            }
        }
        Some(Commands::Send { files, to, pin }) => {
            if let Some(id) = identity.as_ref() {
                commands::send::execute(id, files, to, pin).await;
            }
        }
        Some(Commands::Text { message, to }) => {
            if let Some(id) = identity.as_ref() {
                commands::text::execute(id, message, to).await;
            }
        }
        Some(Commands::Serve {
            save_dir,
            port,
            yes,
            pin,
        })
        | Some(Commands::Receive {
            save_dir,
            port,
            yes,
            pin,
        }) => {
            if let Some(id) = identity.as_ref() {
                commands::receive::execute(id, save_dir, port, yes, pin).await;
            }
        }
        Some(Commands::Background) => {
            commands::service::start_background();
        }
        Some(Commands::Stop) => {
            commands::service::stop_background();
        }
        Some(Commands::Autostart { action }) => {
            let enable = !action.eq_ignore_ascii_case("disable");
            commands::service::configure_autostart(enable);
        }
        Some(Commands::Doctor { export }) | Some(Commands::Debug { export }) => {
            commands::doctor::execute(identity.as_ref(), export.as_deref()).await;
        }
        Some(Commands::Quack { action }) => match action {
            QuackCommands::Validate { file } => {
                commands::quack::validate(&file).await;
            }
            QuackCommands::Run { file, dry_run, yes } => {
                if let Some(id) = identity.as_ref() {
                    commands::quack::run(id, &file, dry_run, yes).await;
                }
            }
        },
        Some(Commands::Duckive { equip, name }) | Some(Commands::Profile { equip, name }) => {
            if let Some(id) = identity.as_ref() {
                commands::duckive::execute(id, &config_dir, equip, name);
            }
        }
        Some(Commands::Open) => {
            if let Err(e) = commands::open::run() {
                eprintln!("Erro ao abrir interface visual: {}", e);
            }
        }
        None => {
            // Sem argumentos: se for um terminal interativo (TTY), abre a TUI interativa completa do Ducker;
            // caso contrário (pipes, redirecionamento ou automação), executa o receptor em linha de comando.
            if let Some(id) = identity.as_ref() {
                use std::io::IsTerminal;
                if std::io::stdout().is_terminal() {
                    if let Err(e) = tui::run(id, &config_dir).await {
                        eprintln!("Erro ao executar interface de terminal: {}", e);
                    }
                } else {
                    commands::receive::execute(id, None, None, false, None).await;
                }
            }
        }
    }
}
