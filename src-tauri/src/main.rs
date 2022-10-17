#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use rand::Rng;
use serialport_manager::Port;
use std::sync::{mpsc, Mutex};
use tauri::Manager;

pub mod battleship;
pub mod serialport_manager;

fn main() {
    tauri::Builder::default()
        .manage(Port(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            serialport_manager::get_ports,
            serialport_manager::pick_port,
            battleship::run_game
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
