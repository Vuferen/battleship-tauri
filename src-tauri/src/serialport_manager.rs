use std::io::{BufReader, BufRead};
use std::sync::mpsc::{self, Sender, Receiver};
use std::time::Instant;
use std::{sync::Mutex, time::Duration};
use std::{str, thread};
use serde::{Serialize, Deserialize};
use serialport::SerialPort;

use crate::battleship::JoystickDirections;

pub struct SerialDriver{
    pub port: Mutex<String>,
    pub baudrate: Mutex<u32>,
    pub buffer_recv: Mutex<Option<Receiver<String>>>,
    pub writer_send: Mutex<Option<Sender<String>>>,
}

impl SerialDriver{
    pub fn arduino_get_board(&self) -> Result<Option<Vec<bool>>, String> {
        let input;
        match self.write("1\n") {
            Ok(text) => input = text,
            Err(err) => return Err(format!("Could not write: {}", err)),
        };

        if input.trim() == "0" {
            return Ok(None); // Fire
        }

        let parsed: Board = match serde_json::from_str(&input) {
            Ok(res) => res,
            Err(err) => return Err(format!("Could not parse json: {}", err)),
        };

        return Ok(Some(parsed.board));
    }

    pub fn arduino_get_joystick_direction(& self) -> Result<Option<JoystickDirections>, String> {
        let input;
        match self.write("2\n") {
            Ok(text) => input = text,
            Err(err) => return Err(format!("Could not write: {}", err)),
        };

        match input.as_str().trim() {
            "0" => Ok(None), // Fire
            "1" => Ok(Some(JoystickDirections::Right)), // Right
            "2" => Ok(Some(JoystickDirections::Left)), // Left
            "3" => Ok(Some(JoystickDirections::Up)), // Up
            "4" => Ok(Some(JoystickDirections::Down)), // Down
            "5" => Ok(Some(JoystickDirections::Stay)), // Stay
            err => Err(format!("Could not match direction: {}", err)),
        }
    }

    pub fn arduino_vibrate(& self) -> Result<String, String> {
        match self.write("3\n") {
            Ok(text) => return Ok(text),
            Err(err) => return Err(format!("Could not write: {}", err)),
        };
    }

    fn write(&self, text: &str) -> Result<String, String> {
        (self.writer_send.lock().unwrap().as_ref().unwrap()).send(text.to_string()).unwrap();
        let res = (self.buffer_recv.lock().unwrap().as_ref().unwrap()).recv();

        match res {
            Ok(text) => return Ok(text),
            Err(err) => return Err(format!("Reciever error: {}", err)),
        }
    }

    pub fn run_port(&self) {
        let (sender, writer) = mpsc::channel();
        *self.writer_send.lock().unwrap() = Some(sender);

        let (buffer, buffer_recv) = mpsc::channel();
        *self.buffer_recv.lock().unwrap() = Some(buffer_recv);
        // let buffer = self.buffer_send.clone();
        let port_name  = (&*self.port.lock().expect("No port")).clone();
        let baudrate = *self.baudrate.lock().expect("No baudrate");
        
        thread::spawn(move || {
            loop {
                let mut port: Option<Box<dyn SerialPort>> = None;
                port = if port.is_some() { 
                        port
                    } else {
                        Some(open_port(port_name.as_str(), baudrate).unwrap())
                    };

                if port.is_some() {
                    let mut the_port = port.unwrap();
                    let res = writer.try_recv();
                    
                    if res.as_ref().is_ok() {
                        let output = res.as_ref().unwrap().as_bytes();
                        the_port.write(output).unwrap();
                        let mut reader = BufReader::new(the_port);
                        let mut input = String::new();
                        let now = Instant::now();
                        loop {
                            let res = reader.read_line(&mut input);
                            if res.is_ok() || now.elapsed() > Duration::from_millis(50) {
                                break;
                            } 
                            // Add timeout here
                        }
                        buffer.send(input).unwrap();
                    }   
                }             
            }
        });
    }
}

// pub struct Port{
//     pub name: Mutex<String>,
//     pub baudrate: Mutex<u32>
// }
// pub struct Port(String);
// pub struct RGB {
//     r: u32,
//     g: u32,
//     b: u32,
// }

#[derive(Serialize, Deserialize)]
struct Board {
    board: Vec<bool>,
}

// impl Port{
//     // pub fn set_arduino_leds(leds: Vec<RGB>) {
//     //     // Send LED info to arduino
//     // }

//     // pub fn arduino_vibrate(duration: Duration) {
//     //     // Send feedback to arduino
//     // }

//     pub fn arduino_get_joystick_direction(& self) -> Result<Option<JoystickDirections>, String> {
//         let port_name  = &*self.name.lock().expect("No port").clone();
//         let port_baudrate = *self.baudrate.lock().expect("No baudrate");
//         let mut port = open_port(port_name, port_baudrate).unwrap();
//         // let output = &[b'2'];
//         let output = "2\n".as_bytes();
//         port.write(output).unwrap();
//         let mut reader = BufReader::new(port);
//         let mut input = String::new();
//         loop {
//             let res = reader.read_line(&mut input);
//             if res.is_ok() {
//                 break;
//             } 
//         }

//         match input.as_str().trim() {
//             "0" => Ok(None), // Fire
//             "1" => Ok(Some(JoystickDirections::Right)), // Right
//             "2" => Ok(Some(JoystickDirections::Left)), // Left
//             "3" => Ok(Some(JoystickDirections::Up)), // Up
//             "4" => Ok(Some(JoystickDirections::Down)), // Down
//             err => Err(format!("Could not match direction: {}", err)),
//         }
//     }

//     pub fn arduino_get_board(& self) -> Result<Option<Vec<bool>>, String> {
//         let port_name  = &*self.name.lock().expect("No port").clone();
//         let port_baudrate = *self.baudrate.lock().expect("No baudrate");
//         let mut port = open_port(port_name, port_baudrate).unwrap();
//         // let output = &[b'1'];
//         let output = "1\n".as_bytes();
//         port.write(output).unwrap();
//         let mut reader = BufReader::new(port);
//         let mut input = String::new();
//         loop {
//             let res = reader.read_line(&mut input);
//             // match res {
//             //     Ok(_) => break,
//             //     Err(err) => println!("{}", err),
//             // }
//             if res.is_ok() {
//                 break;
//             } 
//         }

//         if input.trim() == "0" {
//             return Ok(None); // Fire
//         }

//         let parsed: Board = match serde_json::from_str(&input) {
//             Ok(res) => res,
//             Err(err) => return Err(format!("Could not parse json: {}", err)),
//         };

//         return Ok(Some(parsed.board));
//     }
// }

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
    port: tauri::State<SerialDriver>,
    port_name: String,
    baudrate: u32,
) -> Result<String, String> {
    match serialport::new(&port_name, baudrate)
        .timeout(Duration::from_millis(10))
        .open()
    {
        Ok(_) => {
            // (*port).0 = Some(Mutex::new(res));
            *port.port.lock().unwrap() = port_name;
            *port.baudrate.lock().unwrap() = baudrate;
            return Ok("Port saved".into());
        }
        Err(err) => return Err(format!("Failed to open port {}: {}", port_name, err).into()),
    };
}

// let mut serial_buf: Vec<u8> = vec![0; 32];
// port.read(serial_buf.as_mut_slice()).expect("Found no data!");
