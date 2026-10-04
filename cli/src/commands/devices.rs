use std::time::Duration;
use console::style;
use ducker_core::{DeviceIdentity, DiscoveryManager};
use crate::ui::SEARCH;

pub async fn execute(identity: &DeviceIdentity) {
    println!("\n{} {}", SEARCH, style("Procurando dispositivos Ducker na rede local...").bold());

    let discovery = DiscoveryManager::new(identity.clone());
    if let Err(e) = discovery.start().await {
        eprintln!("Erro ao iniciar serviço de descoberta: {}", e);
        return;
    }

    // Aguardar alguns instantes para receber os pacotes de beacon na LAN
    tokio::time::sleep(Duration::from_millis(2200)).await;

    let devices = discovery.list_devices().await;

    if devices.is_empty() {
        println!("\nNenhum outro dispositivo Ducker encontrado no momento.");
        println!("{}", style("Dica: Certifique-se de que os outros dispositivos (como o Celular) estão na mesma rede Wi-Fi e com o Ducker aberto.").dim());
    } else {
        println!("\n{}", style("Dispositivos encontrados:").bold());
        for (idx, dev) in devices.iter().enumerate() {
            println!(
                "\n{}. {} [{}]",
                style(idx + 1).bold(),
                style(&dev.device_name).cyan().bold(),
                style(dev.quac_id).green().bold()
            );
            println!("   Endereço: {}:{}", dev.address, dev.port);
        }
    }
    println!();
}
