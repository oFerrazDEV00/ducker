use std::net::TcpListener;
use console::style;
use ducker_core::{
    DeviceIdentity, DiscoveryManager, FileReceiver, DEFAULT_DISCOVERY_PORT, DEFAULT_TRANSFER_PORT,
};
use crate::ui::{DUCK, ERROR, SUCCESS};

pub fn execute(identity: Option<&DeviceIdentity>) {
    println!("\n{} {}", DUCK, style("Ducker Doctor — Diagnóstico do Sistema").bold());
    println!("{}", style("----------------------------------------").dim());

    // 1. Identidade
    match identity {
        Some(id) => {
            println!(
                "{} Identidade configurada: {} [ID Quac: {}]",
                SUCCESS,
                style(&id.device_name).cyan(),
                style(id.quac_id).green()
            );
        }
        None => {
            println!("{} Identidade ainda não foi inicializada.", style("!").yellow());
        }
    }

    // 2. IP Local
    match DiscoveryManager::get_local_ip() {
        Some(ip) => {
            println!("{} IP de rede local identificado: {}", SUCCESS, style(ip).cyan());
        }
        None => {
            println!("{} Não foi possível identificar o IP local da rede.", ERROR);
        }
    }

    // 3. Porta TCP (Transferência)
    match TcpListener::bind(("0.0.0.0", DEFAULT_TRANSFER_PORT)) {
        Ok(_) => {
            println!("{} Porta de transferência {} (TCP) está livre para uso.", SUCCESS, DEFAULT_TRANSFER_PORT);
        }
        Err(e) => {
            println!("{} Porta de transferência {} (TCP) ocupada ou indisponível: {}", ERROR, DEFAULT_TRANSFER_PORT, e);
        }
    }

    // 4. Porta UDP (Descoberta)
    match std::net::UdpSocket::bind(("0.0.0.0", DEFAULT_DISCOVERY_PORT)) {
        Ok(_) => {
            println!("{} Porta de descoberta {} (UDP) está livre para uso.", SUCCESS, DEFAULT_DISCOVERY_PORT);
        }
        Err(e) => {
            println!("{} Porta de descoberta {} (UDP) ocupada ou indisponível: {}", ERROR, DEFAULT_DISCOVERY_PORT, e);
        }
    }

    // 5. Diretório de downloads
    let save_dir = FileReceiver::default_save_dir();
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
