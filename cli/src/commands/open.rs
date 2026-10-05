use std::io::{self, Write};
use std::path::PathBuf;
use console::style;
use ducker_core::{DeviceIdentity, DEFAULT_MOBILE_BRIDGE_PORT};
use crate::commands;
use crate::ui::{DUCK, ERROR, SUCCESS};

pub async fn execute(identity: &DeviceIdentity) {
    println!("\n{} {}", DUCK, style("Ducker — Painel de Controle").bold().yellow());
    println!("  Dispositivo: {}", style(&identity.device_name).cyan().bold());
    println!("  ID Quac:     {}", style(identity.quac_id).green().bold());

    // 1. Garante que o serviço de segundo plano está rodando na porta 7876
    let is_running = reqwest_or_tcp_check(DEFAULT_MOBILE_BRIDGE_PORT).await;
    if !is_running {
        println!("\n  Iniciando serviço do Ducker em segundo plano...");
        commands::service::start_background();
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
    }

    // 2. Abre a interface Web no navegador padrão
    let url = format!("http://localhost:{}/", DEFAULT_MOBILE_BRIDGE_PORT);
    println!("  🌐 Interface Web aberta no seu navegador: {}\n", style(&url).cyan().underlined());
    open_browser(&url);

    loop {
        println!("\n{}", style("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").dim());
        println!("{}", style("Escolha uma opção digitando o número:").bold());
        println!("  {} 🌐 Abrir Interface Web no Navegador", style("[1]").yellow().bold());
        println!("  {} 📤 Enviar Arquivo para Celular ou PC", style("[2]").cyan().bold());
        println!("  {} 💬 Enviar Mensagem Curta para o Celular", style("[3]").cyan().bold());
        println!("  {} 📂 Abrir Pasta de Downloads (Ducker)", style("[4]").green().bold());
        println!("  {} 🔍 Buscar Dispositivos Ativos na Rede", style("[5]").white().bold());
        println!("  {} 🪪 Ver Meu ID Quac e Identidade", style("[6]").white().bold());
        println!("  {} ⚙️  Configurar Inicialização com o Windows", style("[7]").white().bold());
        println!("  {} 🩺 Executar Diagnóstico de Rede (Doctor)", style("[8]").white().bold());
        println!("  {} 🛑 Parar Serviço em Segundo Plano", style("[9]").red().bold());
        println!("  {} 🚪 Sair do Menu", style("[0]").dim().bold());

        print!("\n{} ", style("Digite o número (0-9):").bold());
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let choice = input.trim();

        match choice {
            "1" | "" => {
                open_browser(&url);
                println!("\n{} Painel aberto no navegador: {}", SUCCESS, style(&url).cyan());
            }
            "2" => {
                print!("\n{} ", style("Caminho do arquivo a enviar (ou arraste aqui):").bold());
                let _ = io::stdout().flush();
                let mut path_str = String::new();
                if io::stdin().read_line(&mut path_str).is_ok() {
                    let cleaned = path_str.trim().trim_matches('"').trim_matches('\'');
                    let file_path = PathBuf::from(cleaned);
                    if file_path.is_file() {
                        commands::mobile::execute(identity, Some(file_path), None, None).await;
                    } else {
                        eprintln!("{} Arquivo não encontrado: {}", ERROR, cleaned);
                    }
                }
            }
            "3" => {
                print!("\n{} ", style("Digite a mensagem para o celular:").bold());
                let _ = io::stdout().flush();
                let mut msg = String::new();
                if io::stdin().read_line(&mut msg).is_ok() {
                    let text = msg.trim().to_string();
                    if !text.is_empty() {
                        commands::mobile::execute(identity, None, Some(text), None).await;
                    }
                }
            }
            "4" => {
                let save_dir = dirs::download_dir()
                    .map(|d| d.join("Ducker"))
                    .unwrap_or_else(|| PathBuf::from("."));
                let _ = std::fs::create_dir_all(&save_dir);
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("explorer.exe")
                        .arg(&save_dir)
                        .spawn();
                }
                println!("\n{} Pasta aberta: {}", SUCCESS, save_dir.display());
            }
            "5" => {
                commands::devices::execute(identity).await;
            }
            "6" => {
                commands::id::execute(identity);
            }
            "7" => {
                println!("\n[1] Ativar inicialização automática (ligar com o Windows)");
                println!("[2] Desativar inicialização automática");
                print!("Escolha (1 ou 2): ");
                let _ = io::stdout().flush();
                let mut sub = String::new();
                if io::stdin().read_line(&mut sub).is_ok() {
                    match sub.trim() {
                        "1" => commands::service::configure_autostart(true),
                        "2" => commands::service::configure_autostart(false),
                        _ => println!("Opção cancelada."),
                    }
                }
            }
            "8" => {
                commands::doctor::execute(Some(identity));
            }
            "9" => {
                commands::service::stop_background();
            }
            "0" | "sair" | "exit" | "q" => {
                println!("\nAté logo! 🦆 O Ducker continua pronto para receber em segundo plano.\n");
                break;
            }
            _ => {
                println!("\nOpção inválida '{}'. Por favor, digite um número de 0 a 9.", choice);
            }
        }
    }
}

async fn reqwest_or_tcp_check(port: u16) -> bool {
    tokio::net::TcpStream::connect(("127.0.0.1", port)).await.is_ok()
}

fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = std::process::Command::new("cmd")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["/c", "start", "", url])
            .spawn();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = url;
    }
}
