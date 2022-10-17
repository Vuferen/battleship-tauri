#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use serialport_manager::Port;
use std::sync::Mutex;
use tauri::Manager;

pub mod serialport_manager;

fn main() {
    tauri::Builder::default()
        .manage(Port(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            serialport_manager::get_ports,
            serialport_manager::pick_port,
            run_game
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

struct Board {
    ships: Vec<bool>,
    hits: Vec<bool>,
    ships_left: u8,
}

#[tauri::command]
fn run_game(
    handle: tauri::AppHandle,
    port: tauri::State<'_, Port>,
    rows: u8,
    cols: u8,
    ship_sizes: Vec<u8>,
) {
    if port.0.lock().unwrap().is_some() {
        let my_board: Board;
        let their_board: Board;
        // Setup stage
        loop {
            // Get ship positions from arduino
            // Sends positions to frontend
            // Check if game should start
            //      Place enemy ships
            break;
        }
        // Game loop
        loop {
            // Game has started, wait for fire command
            // Handle fire
            //      Change hit state
            //      Send hit state to frontend
            // Do enemy turn
            //      Check if ship hit
            //          Check surrounding tiles
            //              Hit surrounding tiles
            //          Hit random cell

            // Check game end condition, if ships left == 0
            if my_board.ships_left == 0 {
                // Defeat
            }
            if their_board.ships_left == 0 {
                // Victory
            }
        }
    }
}

enum JoystickDirections {
    Up,
    Right,
    Down,
    Left,
}

pub fn board_state(handle: tauri::AppHandle, board: Vec<bool>) {
    // let board = [true, false, false, true, false, false, false, false, false];
    handle.emit_all("board-state", board).unwrap();
}

pub fn joystick_direction(handle: tauri::AppHandle) {
    let direction = JoystickDirections::Right as u32;
    handle.emit_all("joystick_direction", direction).unwrap();
}

pub fn joystick_fire(handle: tauri::AppHandle, fire: Option<bool>) {
    if fire.unwrap_or(false) {
        handle.emit_all("joystick_fire", {}).unwrap();
    }
}
