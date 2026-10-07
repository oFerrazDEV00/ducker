// Evita abrir a janela de console do Windows em builds release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ducker_app_lib::run();
}
