#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Mutex;

use serialport::SerialPort;
use serialport_manager::Port;

pub mod serialport_manager;

fn main() {
    tauri::Builder::default()
        .manage(Port(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            serialport_manager::joystick_fire_feedback,
            serialport_manager::get_ports,
            serialport_manager::pick_port
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
