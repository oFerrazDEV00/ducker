use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};

pub static DUCK: Emoji<'_, '_> = Emoji("🦆 ", "");
pub static SUCCESS: Emoji<'_, '_> = Emoji("✓ ", "");
pub static ERROR: Emoji<'_, '_> = Emoji("✗ ", "");
pub static SEARCH: Emoji<'_, '_> = Emoji("🔍 ", "");

pub fn print_banner() {
    println!("{}", style("========================================").dim());
    println!("{} {}", DUCK, style("DUCKER — Be simple, be duck.").bold().yellow());
    println!("{}", style("========================================").dim());
}

pub fn create_transfer_progress_bar(total_bytes: u64, file_name: &str) -> ProgressBar {
    let pb = ProgressBar::new(total_bytes);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n[{bar:30.cyan/blue}] {percent}% ({bytes}/{total_bytes})")
            .unwrap()
            .progress_chars("██░"),
    );
    pb.set_message(format!("Enviando {}", file_name));
    pb
}
