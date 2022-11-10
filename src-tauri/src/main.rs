#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use battleship::{Cols, CursorPos, Rows};
use serialport_manager::SerialDriver;
use std::sync::Mutex;

pub mod battleship;
pub mod serialport_manager;
pub mod python_manager;
pub mod vector2;

fn main() {
    tauri::Builder::default()
        // .manage(Port{name: Mutex::new("".into()), baudrate: Mutex::new(0)})
        .manage(SerialDriver {
            port: Mutex::new("".into()),
            baudrate: Mutex::new(0),
            buffer_recv: Mutex::new(None),
            writer_send: Mutex::new(None),
            exit_send: Mutex::new(None),
        })
        .manage(CursorPos(Mutex::new(None)))
        .manage(Cols(Mutex::new(None)))
        .manage(Rows(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            serialport_manager::get_ports,
            serialport_manager::pick_port,
            serialport_manager::close_port,
            battleship::set_cursor_pos,
            battleship::set_cols,
            battleship::set_rows,
            battleship::run_game,
            battleship::move_cursor
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
