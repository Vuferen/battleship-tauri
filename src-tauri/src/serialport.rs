use tauri::Manager;

enum JoystickDirections {
    Up,
    Right,
    Down,
    Left,
}

pub fn board_state(handle: tauri::AppHandle) {
    let board = [true, false, false, true, false, false, false, false, false];
    handle.emit_all("board-state", board).unwrap();
}

pub fn joystick_direction(handle: tauri::AppHandle) {
    let direction = JoystickDirections::Right as u32;
    handle.emit_all("joystick_direction", direction).unwrap();
}

pub fn joystick_fire(handle: tauri::AppHandle) {
    // Tell frontend to fire at current cell
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
            // handle.emit_all("available_ports", ports).unwrap();
            return Ok(ports.into());
        }
        Err(err) => return Err(format!("Could not get ports: {}", err).into()),
    };
}

#[tauri::command]
pub fn pick_port(port: String, baudrate: u32) -> Result<String, String> {
    match serialport::new(&port, baudrate).open() {
        Ok(_) => todo!(),
        Err(err) => return Err(format!("Failed to open port {}: {}", port, err).into()),
    };
}
