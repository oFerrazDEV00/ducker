use console::style;
use ducker_core::DeviceIdentity;
use crate::ui::DUCK;

pub fn execute(identity: &DeviceIdentity) {
    println!("\n{} {}", DUCK, style("Identidade deste dispositivo").bold());
    println!("  Nome:    {}", style(&identity.device_name).cyan().bold());
    println!("  ID Quac: {}", style(identity.quac_id).green().bold());
    if let Ok(path) = DeviceIdentity::default_config_path() {
        println!("  Config:  {}", style(path.display()).dim());
    }
    println!();
}
