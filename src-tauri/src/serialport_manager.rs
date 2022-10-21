use std::io::{BufReader, BufRead};
use std::{sync::Mutex, time::Duration};
use std::{str, thread};
use serde::{Serialize, Deserialize};
use serialport::SerialPort;
use serde_json::{Value};

pub struct Port{
    pub name: Mutex<String>,
    pub baudrate: Mutex<u32>
}
// pub struct Port(String);
pub struct RGB {
    r: u32,
    g: u32,
    b: u32,
}

#[derive(Serialize, Deserialize)]
struct Board {
    board: Vec<bool>,
}

impl Port{
    pub fn set_arduino_leds(leds: Vec<RGB>) {
        // Send LED info to arduino
    }

    pub fn arduino_vibrate(duration: Duration) {
        // Send feedback to arduino
    }

    pub fn arduino_get_board(& self) -> Result<Vec<bool>, String> {
        let port_name  = &*self.name.lock().expect("No port").clone();
        let port_baudrate = *self.baudrate.lock().expect("No baudrate");
        let mut port = open_port(port_name, port_baudrate).unwrap();
        // match serialport::new(port_name, port_baudrate)
        // .timeout(Duration::from_millis(10))
        // .open()
        // {
        //     Ok(res) => { port = res }
        //     Err(err) => return Err(format!("Failed to open port {}: {}", port_name, err)),
        // };

        let output = r#"{"msg":"board"}"#.as_bytes();
        port.write(output).unwrap();
        // thread::sleep(Duration::from_millis(200));
        // port.flush().unwrap();
        let mut reader = BufReader::new(port);
        let mut input = String::new();
        loop {
            let res = reader.read_line(&mut input);
            if res.is_ok() {
                break;
            }
        }
        
        
        // let mut serial_buf: Vec<u8> = vec![0; 512];
        // port.read_to_end(&mut serial_buf).unwrap();
        // port.read(serial_buf.as_mut_slice()).expect("Found no data!");
        // let s = match str::from_utf8(&serial_buf) {
        //     Ok(v) => v.trim(),
        //     Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        // };
        // println!("@{}@", input);  
        // let mut board: Vec<bool> = Vec::new();
        let parsed: Board = match serde_json::from_str(&input) {
            Ok(res) => res,
            Err(err) => return Err(format!("Could not parse json: {}", err)),
        };
        // for (i, _col) in (&parsed.response_board).iter().enumerate() {
        //     for cell in &parsed.response_board[i] {
        //         board.push(*cell);
        //     }
        // }
        return Ok(parsed.board);
    }
}

fn open_port(name: &str, baudrate: u32) -> Result<Box<dyn SerialPort>, String> {
    match serialport::new(name, baudrate)
        .timeout(Duration::from_millis(10))
        .open()
        {
            Ok(res) => return Ok(res),
            Err(err) => return Err(format!("Failed to open port {}: {}", name, err)),
        };
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
            // (*port).0 = Some(Mutex::new(res));
            *port.name.lock().unwrap() = port_name;
            *port.baudrate.lock().unwrap() = baudrate;
            return Ok("Port saved".into());
        }
        Err(err) => return Err(format!("Failed to open port {}: {}", port_name, err).into()),
    };
}

// let mut serial_buf: Vec<u8> = vec![0; 32];
// port.read(serial_buf.as_mut_slice()).expect("Found no data!");
