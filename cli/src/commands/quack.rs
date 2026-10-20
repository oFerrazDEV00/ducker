use console::style;
use dialoguer::Confirm;
use ducker_core::quack::QuackFile;
use ducker_core::{Identity, Node, NodeConfig, SendOptions};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::ui::{print_banner, DUCK, ERROR, SUCCESS};

pub async fn validate(file_path: &Path) {
    println!("\n{} Validando arquivo de automação .quack...", DUCK);
    println!("   Arquivo: {}", style(file_path.display()).cyan());

    let quack = match QuackFile::load_from_file(file_path) {
        Ok(q) => q,
        Err(e) => {
            eprintln!(
                "\n{} Erro de validação sintática: {}",
                ERROR,
                style(e).red()
            );
            std::process::exit(1);
        }
    };

    let base_dir = file_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    let plan = match quack.resolve_plan(&base_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "\n{} Erro ao resolver arquivos da automação: {}",
                ERROR,
                style(e).red()
            );
            std::process::exit(1);
        }
    };

    print_plan(&plan);
    println!(
        "\n{} {}",
        SUCCESS,
        style("Arquivo .quack válido e pronto para execução!")
            .green()
            .bold()
    );
}

pub async fn run(
    identity: &Identity,
    file_path: &Path,
    override_dry_run: bool,
    skip_confirmation: bool,
) {
    print_banner();
    println!(
        " Executando automação .quack: {}\n",
        style(file_path.display()).yellow().bold()
    );

    let quack = match QuackFile::load_from_file(file_path) {
        Ok(q) => q,
        Err(e) => {
            eprintln!("{} Erro ao carregar .quack: {}", ERROR, style(e).red());
            std::process::exit(1);
        }
    };

    let base_dir = file_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    let mut plan = match quack.resolve_plan(&base_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{} Erro ao resolver plano: {}", ERROR, style(e).red());
            std::process::exit(1);
        }
    };

    if override_dry_run {
        plan.dry_run = true;
    }

    print_plan(&plan);

    if plan.files.is_empty() {
        println!(
            "\n{} Nenhum arquivo elegível para envio encontrado nas fontes especificadas.",
            style("!").yellow()
        );
        return;
    }

    if plan.dry_run {
        println!(
            "\n{} {}",
            style("[DRY-RUN]").yellow().bold(),
            style("Simulação concluída com sucesso! Nenhum pacote foi transmitido na rede.").dim()
        );
        return;
    }

    // Confirmação interativa se exigida e não suprimida por --yes
    if quack.options.require_confirmation && !skip_confirmation {
        let confirmed = Confirm::new()
            .with_prompt("Deseja confirmar o início do envio desta automação?")
            .default(true)
            .interact()
            .unwrap_or(false);

        if !confirmed {
            println!("\n{} Execução cancelada pelo usuário.", style("!").yellow());
            return;
        }
    }

    // Inicializa nó para envio
    let config = NodeConfig::new(identity.clone(), std::env::temp_dir());
    let (node, _events) = match Node::start(config).await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{} Falha ao inicializar nó Ducker: {}", ERROR, e);
            std::process::exit(1);
        }
    };

    println!("\n{} Procurando destinatário na rede local...", DUCK);
    tokio::time::sleep(std::time::Duration::from_millis(600)).await;

    let peers = node.peers();
    let target_peer = if let Some(found) = plan.target.resolve_peer(&peers) {
        found.clone()
    } else if let Some(synthetic) = plan.target.to_synthetic_peer() {
        synthetic
    } else {
        eprintln!(
            "\n{} Destinatário não encontrado na rede local. Verifique se o aparelho de destino está online com o Ducker/LocalSend aberto.",
            ERROR
        );
        std::process::exit(1);
    };

    println!(
        "{} Destinatário localizado: {} ({}:{})",
        SUCCESS,
        style(&target_peer.info.alias).green().bold(),
        target_peer.ip,
        target_peer.port
    );

    let file_paths: Vec<PathBuf> = plan.files.iter().map(|f| f.source_path.clone()).collect();
    let start_time = Instant::now();

    println!("{} Iniciando transferência...", style("➤").cyan());
    match node
        .send_files(&target_peer, file_paths, SendOptions::default(), |_| {})
        .await
    {
        Ok(()) => {
            let duration = start_time.elapsed().as_secs_f64().max(0.1);
            let mb = (plan.total_bytes as f64) / 1_048_576.0;
            let speed = mb / duration;

            println!(
                "\n{} Automação concluída com sucesso! 🦆",
                style("✓").green().bold()
            );
            println!(
                "   Enviados: {} arquivo(s) ({:.2} MB) em {:.1}s ({:.2} MB/s)",
                plan.file_count, mb, duration, speed
            );
        }
        Err(e) => {
            eprintln!("\n{} Falha na transferência: {}", ERROR, style(e).red());
            std::process::exit(1);
        }
    }
}

fn print_plan(plan: &ducker_core::QuackPlan) {
    println!(
        "\n{}",
        style("── Plano de Execução da Automação ──")
            .yellow()
            .bold()
    );
    println!("  Nome da Tarefa:  {}", style(&plan.name).bold());

    let target_desc = if let Some(ref a) = plan.target.alias {
        format!("Apelido '{}'", a)
    } else if let Some(ref q) = plan.target.quac_id {
        format!("ID Quac #{}", q)
    } else if let Some(ref ip) = plan.target.ip {
        format!("IP {}:{}", ip, plan.target.port)
    } else {
        "Não especificado".to_string()
    };
    println!("  Destinatário:    {}", style(target_desc).cyan());

    let mb = (plan.total_bytes as f64) / 1_048_576.0;
    println!(
        "  Total a enviar:  {} arquivo(s) ({:.2} MB)",
        plan.file_count, mb
    );
    println!("  Política conflito: {}", plan.conflict);
    if plan.dry_run {
        println!(
            "  Modo:            {}",
            style("SIMULAÇÃO (DRY-RUN)").yellow().bold()
        );
    }

    println!("\n  Arquivos selecionados:");
    let max_display = 15;
    for (i, f) in plan.files.iter().take(max_display).enumerate() {
        let kb = (f.size_bytes as f64) / 1024.0;
        println!(
            "   [{}] {} ({:.1} KB)",
            i + 1,
            style(&f.relative_dest_name).white(),
            kb
        );
    }
    if plan.files.len() > max_display {
        println!(
            "   ... e mais {} arquivo(s).",
            plan.files.len() - max_display
        );
    }
}
