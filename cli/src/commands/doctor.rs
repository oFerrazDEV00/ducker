use std::net::{TcpListener, UdpSocket};
use console::style;
use ducker_core::{default_save_dir, discovery::local_ipv4s, Identity, DEFAULT_PORT};
use crate::ui::{DUCK, ERROR, SUCCESS};

pub fn execute(identity: Option<&Identity>) {
    println!("\n{} {}", DUCK, style("Ducker Doctor — Diagnóstico do Sistema (LocalSend v2)").bold());
    println!("{}", style("---------------------------------------------------------").dim());

    // 1. Identidade
    match identity {
        Some(id) => {
            println!(
                "{} Identidade: {} [ID Quac: {}]",
                SUCCESS,
                style(&id.alias).cyan().bold(),
                style(id.quac_id).green().bold()
            );
            println!("   Fingerprint: {}", style(&id.fingerprint).yellow());
        }
        None => {
            println!("{} Identidade ainda não foi inicializada.", style("!").yellow());
        }
    }

    // 2. Interfaces IPv4
    let ips = local_ipv4s();
    if ips.is_empty() {
        println!("{} Nenhuma interface de rede IPv4 local ativa encontrada.", ERROR);
    } else {
        let ip_strs: Vec<String> = ips.iter().map(|ip| ip.to_string()).collect();
        println!("{} Interfaces IPv4 ativas: {}", SUCCESS, style(ip_strs.join(", ")).cyan());
    }

    // 3. Porta TCP 53317 (Servidor HTTP/HTTPS)
    match TcpListener::bind(("0.0.0.0", DEFAULT_PORT)) {
        Ok(_) => {
            println!("{} Porta padrão {} (TCP) está livre para uso.", SUCCESS, DEFAULT_PORT);
        }
        Err(e) => {
            println!(
                "{} Porta padrão {} (TCP) ocupada ou bloqueada: {} (outro Ducker/LocalSend ativo?)",
                ERROR, DEFAULT_PORT, e
            );
        }
    }

    // 4. Porta UDP 53317 (Descoberta Multicast)
    match UdpSocket::bind(("0.0.0.0", DEFAULT_PORT)) {
        Ok(_) => {
            println!("{} Porta padrão {} (UDP Multicast) está livre para uso.", SUCCESS, DEFAULT_PORT);
        }
        Err(e) => {
            println!("{} Porta padrão {} (UDP) ocupada ou bloqueada: {}", ERROR, DEFAULT_PORT, e);
        }
    }

    // 5. Diretório de downloads
    let save_dir = default_save_dir();
    match std::fs::create_dir_all(&save_dir) {
        Ok(_) => {
            println!("{} Diretório de recebimento acessível: {}", SUCCESS, style(save_dir.display()).dim());
        }
        Err(e) => {
            println!("{} Erro ao acessar diretório de recebimento {}: {}", ERROR, save_dir.display(), e);
        }
    }

    println!("\n{} Diagnóstico concluído.", SUCCESS);
    println!();
}
