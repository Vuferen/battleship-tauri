#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

pub mod serialport;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            serialport::joystick_fire_feedback,
            serialport::get_ports,
            serialport::pick_port
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
