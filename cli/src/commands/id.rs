use console::style;
use ducker_core::Identity;
use crate::ui::DUCK;

pub fn execute(identity: &Identity) {
    println!("\n{} {}", DUCK, style("Identidade deste dispositivo").bold());
    println!("  Apelido (Alias):  {}", style(&identity.alias).cyan().bold());
    println!("  ID Quac:          {}", style(identity.quac_id).green().bold());
    println!("  Fingerprint SHA:  {}", style(&identity.fingerprint).yellow());
    if let Ok(dir) = Identity::default_config_dir() {
        println!("  Configuração:     {}", style(dir.display()).dim());
    }
    println!();
}
