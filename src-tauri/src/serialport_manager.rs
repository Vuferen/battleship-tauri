use std::{sync::Mutex, time::Duration};

use serialport::SerialPort;

pub struct Port(pub Mutex<Option<Box<dyn SerialPort>>>);
pub struct RGB {
    r: u32,
    g: u32,
    b: u32,
}

pub fn set_arduino_leds(leds: Vec<RGB>) {
    // Send LED info to arduino
}

pub fn arduino_vibrate(duration: Duration) {
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
    match serialport::new(&port_name, baudrate)
        .timeout(Duration::from_millis(10))
        .open()
    {
        Ok(res) => {
            *port.0.lock().unwrap() = Some(res);
            return Ok("Port saved".into());
        }
        Err(err) => return Err(format!("Failed to open port {}: {}", port_name, err).into()),
    };
}

// let mut serial_buf: Vec<u8> = vec![0; 32];
// port.read(serial_buf.as_mut_slice()).expect("Found no data!");
