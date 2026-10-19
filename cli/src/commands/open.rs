use console::style;
use std::process::Command;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        style("🦆 Abrindo o Ducker Desktop...").yellow().bold()
    );

    // Se houver ducker.exe ou ducker-app.exe travado na porta, limpa
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("taskkill")
            .args(["/IM", "ducker-app.exe", "/F"])
            .output();
        let _ = Command::new("taskkill")
            .args(["/IM", "ducker.exe", "/F"])
            .output();
    }

    let exe_candidates = [
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("ducker-app.exe"))),
        Some(std::path::PathBuf::from("target/debug/ducker-app.exe")),
        Some(std::path::PathBuf::from("target/release/ducker-app.exe")),
    ];

    for path in exe_candidates.into_iter().flatten() {
        if path.exists() {
            println!(
                "{} {}",
                style("Iniciando:").dim(),
                style(path.display()).cyan()
            );
            Command::new(&path).spawn()?;
            println!(
                "{}",
                style("✨ Interface visual aberta com sucesso!")
                    .green()
                    .bold()
            );
            return Ok(());
        }
    }

    println!(
        "{}",
        style("Compilando e abrindo interface visual com cargo...").dim()
    );
    Command::new("cargo")
        .args(["run", "-p", "ducker-app"])
        .spawn()?;

    Ok(())
}
