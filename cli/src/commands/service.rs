use std::path::PathBuf;
use console::style;
use crate::ui::{DUCK, ERROR, SUCCESS};

/// Inicia o serviço Ducker em segundo plano (background) no Windows
pub fn start_background() {
    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{} Falha ao obter caminho do executável: {}", ERROR, e);
            return;
        }
    };

    println!("\n{} {}", DUCK, style("Iniciando Ducker em segundo plano...").bold().yellow());

    #[cfg(target_os = "windows")]
    {
        use std::process::{Command, Stdio};
        use std::os::windows::process::CommandExt;

        const CREATE_NO_WINDOW: u32 = 0x08000000;
        const DETACHED_PROCESS: u32 = 0x00000008;

        let log_dir = dirs::home_dir().map(|h| h.join(".ducker")).unwrap_or_else(|| PathBuf::from("."));
        let _ = std::fs::create_dir_all(&log_dir);
        let log_path = log_dir.join("ducker.log");

        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path);

        let (stdout_cfg, stderr_cfg) = match log_file {
            Ok(file) => {
                let err_file = file.try_clone().ok();
                (Stdio::from(file), err_file.map(Stdio::from).unwrap_or_else(Stdio::null))
            }
            Err(_) => (Stdio::null(), Stdio::null()),
        };

        match Command::new(&exe_path)
            .args(["serve", "--yes"])
            .stdin(Stdio::null())
            .stdout(stdout_cfg)
            .stderr(stderr_cfg)
            .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
            .spawn()
        {
            Ok(_) => {
                println!("{} {}", SUCCESS, style("Ducker está rodando em segundo plano!").green().bold());
                println!("  • O terminal pode ser fechado livremente.");
                println!("  • O dispositivo está ouvindo no protocolo LocalSend v2 (porta 53317).");
                println!("  • Aceite automático ativado (--yes) para execução sem terminal.");
                println!("  • Logs salvos em: {}", style(log_path.display()).dim());
                println!("  • Para parar o serviço quando quiser: {}", style("ducker stop").cyan());
            }
            Err(e) => {
                eprintln!("{} Falha ao iniciar processo em segundo plano: {}", ERROR, e);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("Modo segundo plano atualmente otimizado para Windows.");
    }
}

/// Encerra processos do Ducker rodando em segundo plano
pub fn stop_background() {
    println!("\n{} {}", DUCK, style("Parando Ducker em segundo plano...").bold().yellow());

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        use std::os::windows::process::CommandExt;

        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let my_pid = std::process::id();

        let script = format!(
            "Get-Process -Name ducker -ErrorAction SilentlyContinue | Where-Object {{ $_.Id -ne {} }} | Stop-Process -Force",
            my_pid
        );

        let _ = Command::new("powershell")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["-NoProfile", "-Command", &script])
            .status();

        println!("{} {}", SUCCESS, style("Ducker em segundo plano foi finalizado.").green().bold());
    }

    #[cfg(not(target_os = "windows"))]
    {
        println!("Comando stop executado.");
    }
}

/// Configura o Ducker para iniciar automaticamente ao ligar o Windows
pub fn configure_autostart(enable: bool) {
    #[cfg(target_os = "windows")]
    {
        let startup_dir = dirs::config_dir()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup"));

        let startup_dir = match startup_dir {
            Some(d) if d.exists() => d,
            _ => {
                // Fallback via variável de ambiente APPDATA
                if let Ok(appdata) = std::env::var("APPDATA") {
                    PathBuf::from(appdata).join("Microsoft\\Windows\\Start Menu\\Programs\\Startup")
                } else {
                    eprintln!("{} Não foi possível localizar a pasta de Inicialização do Windows.", ERROR);
                    return;
                }
            }
        };

        let vbs_file = startup_dir.join("DuckerBackground.vbs");

        if enable {
            let exe_path = match std::env::current_exe() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{} Erro ao obter caminho do executável: {}", ERROR, e);
                    return;
                }
            };

            let log_dir = dirs::home_dir().map(|h| h.join(".ducker")).unwrap_or_else(|| PathBuf::from("."));
            let _ = std::fs::create_dir_all(&log_dir);
            let log_path = log_dir.join("ducker.log");

            let vbs_content = format!(
                "Set WshShell = CreateObject(\"WScript.Shell\")\n\
                 cmd = \"cmd.exe /c \" & Chr(34) & Chr(34) & \"{}\" & Chr(34) & \" serve --yes\" & Chr(34) & \" >> \" & Chr(34) & \"{}\" & Chr(34) & \" 2>&1\"\n\
                 WshShell.Run cmd, 0, False\n",
                exe_path.display(),
                log_path.display()
            );

            if let Err(e) = std::fs::write(&vbs_file, vbs_content) {
                eprintln!("{} Falha ao habilitar inicialização automática: {}", ERROR, e);
            } else {
                println!("\n{} {}", SUCCESS, style("Inicialização automática ATIVADA!").green().bold());
                println!("  • O Ducker agora iniciará silenciosamente toda vez que você ligar o PC.");
                println!("  • Você não precisará abrir nenhum terminal nem digitar comandos.");
                println!("  • Para desativar a qualquer momento: {}", style("ducker autostart disable").cyan());
            }
        } else {
            if vbs_file.exists() {
                let _ = std::fs::remove_file(&vbs_file);
            }
            println!("\n{} {}", SUCCESS, style("Inicialização automática DESATIVADA.").yellow().bold());
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("Inicialização automática com o sistema configurada para Windows.");
    }
}
