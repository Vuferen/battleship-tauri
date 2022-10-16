use std::sync::Mutex;

use serialport::SerialPort;
use tauri::Manager;

enum JoystickDirections {
    Up,
    Right,
    Down,
    Left,
}

pub struct Port(pub Mutex<Option<Box<dyn SerialPort>>>);

pub fn board_state(handle: tauri::AppHandle) {
    let board = [true, false, false, true, false, false, false, false, false];
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

#[tauri::command]
pub fn joystick_fire_feedback() {
    // Send feedback to arduino
}

#[tauri::command]
pub fn get_ports() -> Result<Vec<String>, String> {
    let mut ports: Vec<String> = Vec::new();
    match serialport::available_ports() {
        Ok(port_infos) => {
            for port_info in port_infos {
                ports.push(port_info.port_name);
            }
            return Ok(ports.into());
        }
        Err(err) => return Err(format!("Could not get ports: {}", err).into()),
    };
}

#[tauri::command]
pub fn pick_port(
    port: tauri::State<Port>,
    port_name: String,
    baudrate: u32,
) -> Result<String, String> {
    match serialport::new(&port_name, baudrate).open() {
        Ok(res) => {
            *port.0.lock().unwrap() = Some(res);
            return Ok("Port saved".into());
        }
        Err(err) => return Err(format!("Failed to open port {}: {}", port_name, err).into()),
    };
}
