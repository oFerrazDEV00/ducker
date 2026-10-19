use crate::ui::DUCK;
use console::style;
use ducker_core::{DuckiveProfile, Identity};
use std::path::Path;

pub fn execute(
    identity: &Identity,
    config_dir: &Path,
    equip_id: Option<String>,
    rename: Option<String>,
) {
    let mut profile = match DuckiveProfile::load_or_create(config_dir, &identity.alias) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Erro ao carregar Duckive: {}", e);
            return;
        }
    };

    if let Some(new_name) = rename {
        let trimmed = new_name.trim().to_string();
        if !trimmed.is_empty() {
            profile.name = trimmed;
            let _ = profile.save(config_dir);
            println!(
                "Nome do Duckive atualizado para: {}",
                style(&profile.name).yellow().bold()
            );
        }
    }

    if let Some(item_to_equip) = equip_id {
        match profile.equip_item(item_to_equip.trim()) {
            Ok(()) => {
                let _ = profile.save(config_dir);
                println!(
                    "{} Item '{}' equipado com sucesso!",
                    DUCK,
                    style(item_to_equip.trim()).green().bold()
                );
            }
            Err(e) => {
                eprintln!("Erro ao equipar item: {}", e);
            }
        }
        return;
    }

    let view = profile.to_view();
    let lvl = &view.level;

    println!(
        "\n{}",
        style("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").yellow()
    );
    println!(
        "  {} {} — O Mascote Pessoal do seu Ducker",
        DUCK,
        style("DUCKIVE").bold().yellow()
    );
    println!(
        "{}\n",
        style("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").yellow()
    );

    // Arte ASCII do pato estilizado
    println!("      {}", style("  __      ").yellow().bold());
    println!("      {}", style("<(o )___  ").yellow().bold());
    println!("      {}", style(" ( ._> /  ").yellow().bold());
    println!("      {}", style("  `---'   ").yellow().bold());

    println!("\n  Nome:           {}", style(&view.name).cyan().bold());
    println!(
        "  Nível:          {} ({})",
        style(lvl.level).green().bold(),
        style(&lvl.title).bold()
    );
    println!(
        "  Arquivos enviados: {}",
        style(view.files_sent_count).yellow().bold()
    );
    println!("  Transferências: {}", style(view.transfers_count).dim());

    // Barra de progresso para o próximo nível
    if let Some(next_req) = lvl.next_level_files {
        let pct = lvl.progress_pct as usize;
        let filled = (pct / 5).min(20);
        let empty = 20 - filled;
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

        println!(
            "\n  Próximo Nível:  [{}] {}% ({}/{} arquivos)",
            style(bar).cyan(),
            pct,
            view.files_sent_count,
            next_req
        );

        if let Some(ref reward) = lvl.next_reward_item {
            println!(
                "  Próximo item:   {} {} ({})",
                reward.icon_emoji,
                style(&reward.name).yellow().bold(),
                reward.rarity.label()
            );
        }
    } else {
        println!("\n  Nível Máximo Atingido! Você é o Mestre Cósmico supremo dos Quacs. 🌌");
    }

    println!(
        "\n{}",
        style("── Itens Equipados ─────────────────────────────").dim()
    );
    let head = view.equipped.head.as_deref().unwrap_or("Nenhum");
    let outfit = view.equipped.outfit.as_deref().unwrap_or("Nenhum");
    let acc = view.equipped.accessory.as_deref().unwrap_or("Nenhum");
    let eff = view.equipped.effect.as_deref().unwrap_or("Nenhum");

    println!("  👑 Cabeça:    {}", style(head).cyan());
    println!("  🧥 Roupa:     {}", style(outfit).cyan());
    println!("  👓 Acessório: {}", style(acc).cyan());
    println!("  ✨ Efeito:    {}", style(eff).cyan());

    println!(
        "\n{}",
        style("── Inventário de Cosméticos ─────────────────────").dim()
    );
    for item_status in &view.items {
        let it = &item_status.item;
        let status_str = if item_status.equipped {
            format!("{}", style("[EQUIPADO]").green().bold())
        } else if item_status.unlocked {
            format!("{}", style("[DESBLOQUEADO]").cyan())
        } else {
            format!(
                "{}/{} ({:.0}%)",
                item_status.progress_current,
                item_status.progress_required,
                item_status.progress_pct
            )
        };

        println!(
            "  {} {:<24} [{}] - {}",
            it.icon_emoji,
            style(&it.name).bold(),
            style(it.category.label()).dim(),
            status_str
        );
    }

    println!(
        "\n{}",
        style("── Conquistas ───────────────────────────────────").dim()
    );
    let unlocked_count = view.achievements.iter().filter(|a| a.unlocked).count();
    println!(
        "  Progresso: {}/{} desbloqueadas\n",
        unlocked_count,
        view.achievements.len()
    );

    for ach in &view.achievements {
        let tag = if ach.unlocked {
            style("✓ CONQUISTADO").green().bold()
        } else {
            style("🔒 BLOQUEADO").dim()
        };
        println!(
            "  {} {:<26} {}",
            ach.icon_emoji,
            style(&ach.title).bold(),
            tag
        );
        println!("     {}", style(&ach.description).dim());
    }

    println!(
        "\n{}",
        style("Para equipar um item via terminal: ducker duckive --equip <item_id>").dim()
    );
}
