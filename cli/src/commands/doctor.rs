use console::style;
use ducker_core::diagnostics::run_diagnostics;
use ducker_core::Identity;
use std::path::Path;

use crate::ui::{print_banner, ERROR, SUCCESS};

pub async fn execute(identity: Option<&Identity>, export_path: Option<&Path>) {
    print_banner();
    println!(
        " {}\n",
        style("Debugger do Pato — Laboratório & Diagnóstico de Rede")
            .yellow()
            .bold()
    );

    let report = run_diagnostics(None, identity).await;

    // 1. Identidade do nó local
    println!("{}", style("── Identidade & Credenciais ──").dim());
    match identity {
        Some(id) => {
            println!(
                "{} Dispositivo: {} [ID Quac: #{}]",
                SUCCESS,
                style(&id.alias).cyan().bold(),
                style(id.quac_id).green().bold()
            );
            println!("   Fingerprint: {}", style(&id.fingerprint).yellow());
        }
        None => {
            println!(
                "{} Dispositivo não configurado localmente.",
                style("!").yellow()
            );
        }
    }
    println!();

    // 2. Interfaces de rede
    println!("{}", style("── Interfaces de Rede Detectadas ──").dim());
    if report.interfaces.is_empty() {
        println!("{} Nenhuma interface de rede encontrada.", ERROR);
    } else {
        for iface in &report.interfaces {
            let ip_str = iface
                .ipv4
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| "Sem IPv4".to_string());
            let status_badge = if iface.is_loopback {
                style("[Loopback]").dim()
            } else if iface.is_up {
                style("[Ativa]").green().bold()
            } else {
                style("[Inativa]").dim()
            };

            let mask_str = iface
                .netmask_v4
                .map(|m| format!("(máscara {})", m))
                .unwrap_or_default();

            println!(
                " • {} {}: {} {}",
                style(&iface.name).white().bold(),
                status_badge,
                style(ip_str).cyan(),
                style(mask_str).dim()
            );
        }
    }
    println!();

    // 3. Portas do protocolo
    println!("{}", style("── Portas do Protocolo LocalSend v2 ──").dim());
    for p in &report.ports {
        let (icon, state_styled) = if p.state == "DISPONÍVEL" {
            (SUCCESS, style(p.state.clone()).green())
        } else {
            (ERROR, style(p.state.clone()).red().bold())
        };
        println!(
            "{} Porta {} ({}): {} — {}",
            icon, p.port, p.protocol, state_styled, p.message
        );
    }
    println!();

    // 4. Multicast
    println!("{}", style("── Descoberta Multicast UDP ──").dim());
    println!(
        " • Grupo Multicast: {}:{}",
        report.multicast.group, report.multicast.port
    );
    println!(" • Status: {}", report.multicast.message);
    println!();

    // 5. Diagnóstico de Anomalias e Soluções
    println!(
        "{}",
        style("── Diagnóstico Educativo & Recomendações ──").dim()
    );
    if report.anomalies.is_empty() {
        println!(
            "{} {}",
            SUCCESS,
            style("Tudo operando normalmente! Rede local pronta para transferências P2P.")
                .green()
                .bold()
        );
    } else {
        for (i, a) in report.anomalies.iter().enumerate() {
            println!(
                "\n{} {}. {}",
                style("⚠").yellow().bold(),
                i + 1,
                style(&a.category).yellow().bold()
            );
            println!("   Observado:    {}", style(&a.observed).white());
            println!("   Significado:  {}", style(&a.meaning).dim());
            println!("   Causa provável: {}", style(&a.probable_cause).dim());
            println!(
                "   Ação sugerida:  {}",
                style(&a.recommended_action).cyan().bold()
            );
        }
    }
    println!();

    // 6. Exportação do relatório
    if let Some(path) = export_path {
        let md = report.to_markdown();
        match std::fs::write(path, md) {
            Ok(_) => {
                println!(
                    "{} Relatório do Debugger do Pato exportado com sucesso para: {}",
                    SUCCESS,
                    style(path.display()).green().bold()
                );
            }
            Err(e) => {
                eprintln!(
                    "{} Erro ao salvar relatório em '{}': {}",
                    ERROR,
                    path.display(),
                    e
                );
            }
        }
    } else {
        println!(
            "{}",
            style("💡 Dica: execute 'ducker doctor --export relatorio.md' para exportar um relatório de suporte.")
                .dim()
        );
    }
}
