use serde::{Deserialize, Serialize};
use serialport::{FlowControl, SerialPort};
use std::io::{BufRead, BufReader};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Instant;
use std::{str, thread};
use std::{sync::Mutex, time::Duration};

#[derive(Serialize, Deserialize)]
struct Board {
    board: Vec<bool>,
}
pub struct JoystickDirection {
    pub x: i32,
    pub y: i32,
}

#[derive(PartialEq)]
pub enum InputTag {
    Reset,
    Board,
    Fire,
    Joystick,
    End,
    Turn,
}

pub struct Input {
    pub tag: InputTag,
    pub ships: Vec<bool>,
    pub joystick_direction: JoystickDirection,
    pub turn: Option<bool>,
}

pub struct SerialDriver {
    pub port: Mutex<String>,
    pub baudrate: Mutex<u32>,
    pub buffer_recv: Mutex<Option<Receiver<String>>>,
    pub writer_send: Mutex<Option<Sender<String>>>,
    pub exit_send: Mutex<Option<Sender<bool>>>,
}

impl SerialDriver {
    pub fn arduino_reset(&self) -> Result<String, String> {
        match self.write("0\n") {
            Ok(text) => return Ok(text),
            Err(err) => return Err(format!("Could not write: {}", err)),
        };
    }

    // Depricated, should now send board instead
    // pub fn arduino_get_board(&self) -> Result<Option<Vec<bool>>, String> {
    //     let input;
    //     match self.write("1\n") {
    //         Ok(text) => input = text,
    //         Err(err) => return Err(format!("Could not write: {}", err)),
    //     };

    //     if input.trim() == "0" {
    //         return Ok(None); // Fire
    //     }

    //     let parsed: Board = match serde_json::from_str(&input) {
    //         Ok(res) => res,
    //         Err(err) => return Err(format!("Could not parse json: {}", err)),
    //     };

    //     return Ok(Some(parsed.board));
    // }
    pub fn arduino_try_get_board(&self) -> Result<Input, String> {
        // let board = Board{board: ships};
        // let json = json!(board);
        let input;
        match self.write("6\n") {
            Ok(text) => input = text,
            Err(err) => return Err(format!("Could not write: {}", err)),
        };
        return SerialDriver::handle_response(input);
    }

    pub fn arduino_send_board(&self, ships: Vec<bool>) -> Result<Input, String> {
        // encode:
        let mut num: u128 = 0;
        for (i,ship) in ships.iter().enumerate() {
            if *ship {
                num += (2 as u128).pow(i as u32) as u128;
            }
        }
        // println!("my: {num}");

        let input;
        // let bit_ships = bitarr![0; 100];
        // for (i,ship) in ships.iter().enumerate() {
        //     bit_ships[i] = *ship;
        // }
        // let temp = bit_ships.get(0..100).unwrap().to_string();

        match self.write(("1".to_owned() + num.to_string().as_str() + "\n").as_str()) {
            Ok(text) => input = text,
            Err(err) => return Err(format!("Could not write: {}", err)),
        };

        // Old json version
        // let board = Board { board: ships };
        // let json = json!(board).to_string();
        // let input;
        // match self.write(("1".to_owned() + json.as_str() + "\n").as_str()) {
        //     Ok(text) => input = text,
        //     Err(err) => return Err(format!("Could not write: {}", err)),
        // };
        return SerialDriver::handle_response(input);
    }

    pub fn arduino_get_joystick_direction(&self) -> Result<Input, String> {
        let input;
        match self.write("2\n") {
            Ok(text) => input = text,
            Err(err) => return Err(format!("Could not write: {}", err)),
        };
        return SerialDriver::handle_response(input);
        // println!("{}", input);
        // if input.len() == 0 {
        //     return Err("No response from arduino".to_string());
        // }
        // let first_char = input.as_bytes()[0];
        // match first_char {
        //     b'2' => Ok(None),
        //     b'3' => {
        //         let trimmed_input: &str = &input[1..input.len()].trim();
        //         let vec = trimmed_input.split(',').filter_map(|s| s.parse::<i32>().ok()).collect::<Vec<_>>(); // <- this does not work
        //         Ok(Some(JoystickDirection{x: vec[0], y: vec[1]}))},
        //     err => Err(format!("Could not match direction: {}", err)),
        // }
    }

    pub fn arduino_miss(&self) -> Result<String, String> {
        match self.write("4\n") {
            Ok(text) => return Ok(text),
            Err(err) => return Err(format!("Could not write: {}", err)),
        };
    }

    pub fn arduino_hit(&self, cell: usize) -> Result<String, String> {
        match self.write(format!("3{}\n", cell).as_str()) {
            Ok(text) => return Ok(text),
            Err(err) => return Err(format!("Could not write: {}", err)),
        };
    }

    pub fn arduino_try_get_fire(&self) -> Option<bool> {
        let input;
        match self.write("\n") {
            Ok(text) => input = text,
            Err(_) => return None,
        };
        let res = SerialDriver::handle_response(input);
        if res.is_ok() {
            if res.unwrap().tag == InputTag::Fire {
                return Some(true);
            }
        }
        return None;
    }

    pub fn arduino_try_get_turn(&self) -> Option<bool> {
        let input;
        match self.write("\n") {
            Ok(text) => input = text,
            Err(_) => return None,
        };
        let res = SerialDriver::handle_response(input);
        if res.is_ok() {
            if res.unwrap().tag == InputTag::Turn {
                return Some(true);
            }
        }
        return None;
    }

    fn handle_response(input: String) -> Result<Input, String> {
        if input.len() == 0 {
            return Err("No response from arduino".to_string());
        }
        let first_char = input.as_bytes()[0];
        match first_char {
            // Reset
            b'0' => Ok(Input {
                tag: InputTag::Reset,
                ships: vec![false; 0],
                joystick_direction: JoystickDirection { x: 0, y: 0 },
                turn: None,
            }),
            // Board
            b'1' => {
                // println!("{input}");
                let trimmed_input: &str = &input[1..input.len()].trim();
                // println!("{trimmed_input}");
                let num = trimmed_input.parse::<u128>().unwrap_or(0);
                // println!("{num}");

                let mut their_ships = vec![false;100];
                let bits = format!("{num:b}");
                for (i, ch) in bits.chars().rev().enumerate() {
                    their_ships[i] = ch == '1';
                }
                // for ship in their_ships.clone() {
                //     print!("{ship}, ");
                // }

                // let parsed: Board = match serde_json::from_str(&trimmed_input) {
                //     Ok(res) => res,
                //     Err(err) => return Err(format!("Could not parse json: {}", err)),
                // };

                Ok(Input {
                    tag: InputTag::Board,
                    ships: their_ships,
                    joystick_direction: JoystickDirection { x: 0, y: 0 },
                    turn: None,
                })
            }
            // Fire
            b'2' => Ok(Input {
                tag: InputTag::Fire,
                ships: vec![false; 0],
                joystick_direction: JoystickDirection { x: 0, y: 0 },
                turn: None,
            }),
            // JS Dir
            b'3' => {
                let trimmed_input: &str = &input[1..input.len()].trim();
                let vec = trimmed_input
                    .split(",")
                    .filter_map(|s| s.parse::<i32>().ok())
                    .collect::<Vec<_>>();
                
                if vec.len() > 2 {
                    Ok(Input {
                        tag: InputTag::Joystick,
                        ships: vec![false; 0],
                        joystick_direction: JoystickDirection {
                            x: vec[0],
                            y: vec[1],
                        },
                        turn: Some(vec[2] == 1),
                    })
                } else {
                    Ok(Input {
                        tag: InputTag::Joystick,
                        ships: vec![false; 0],
                        joystick_direction: JoystickDirection {
                            x: 0,
                            y: 0,
                        },
                        turn: Some(false),
                    })
                }
            }
            // Defeat
            b'4' => Ok(Input {
                tag: InputTag::End,
                ships: vec![false; 0],
                joystick_direction: JoystickDirection { x: 0, y: 0 },
                turn: None,
            }),
            // Your turn
            b'5' => Ok(Input {
                tag: InputTag::Turn,
                ships: vec![false; 0],
                joystick_direction: JoystickDirection { x: 0, y: 0 },
                turn: None,
            }),
            err => Err(format!("Could not handle response: {}", err)),
        }
    }

    fn write(&self, text: &str) -> Result<String, String> {
        (self.writer_send.lock().unwrap().as_ref().unwrap())
            .send(text.to_string())
            .unwrap();
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

        let (exit_sender, exit_recv) = mpsc::channel();
        *self.exit_send.lock().unwrap() = Some(exit_sender);

        let port_name = (&*self.port.lock().expect("No port")).clone();
        let baudrate = *self.baudrate.lock().expect("No baudrate");

        thread::spawn(move || {
            let mut exit = false;
            loop {
                let mut port: Option<Box<dyn SerialPort>> = None;

                if !port.is_some() {
                    match open_port(port_name.as_str(), baudrate) {
                        Ok(res) => {
                            port = Some(res);
                        }
                        Err(_) => exit = true,
                    }
                    // Some(open_port(port_name.as_str(), baudrate).unwrap())
                };

                // Check for exit message
                let exit_res = exit_recv.try_recv();
                if exit_res.as_ref().is_ok() {
                    exit = *exit_res.as_ref().unwrap();
                }

                // Terminate thread
                if exit {
                    break;
                }

                if port.is_some() {
                    let mut the_port = port.unwrap();
                    the_port.set_flow_control(FlowControl::Hardware).unwrap();
                    let res = writer.try_recv();

                    if res.as_ref().is_ok() {
                        let output = res.as_ref().unwrap().as_bytes();
                        the_port.write(output).unwrap();
                        let mut reader = BufReader::new(the_port);
                        let mut input = String::new();
                        let now = Instant::now();
                        loop {
                            let res = reader.read_line(&mut input);
                            if res.is_ok() || now.elapsed() > Duration::from_millis(250) {
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

    pub fn close_port(&self) -> Result<(),String> {
        let res_mutex = self.exit_send.lock();
        match res_mutex {
            Ok(_) => {
                // let res_option = ;
                match res_mutex.unwrap().as_ref(){
                    Some(sender) => return sender.send(true).map_err(|_|{return "Could not restart".to_string();}),
                    None => return Err("Restart sender not found".to_string()),
                };
            },
            Err(_) => return Err("Could not get mutex".to_string()),
        };
        // (self.exit_send.lock().unwrap().as_ref().unwrap())
        //     .send(true)
        //     .unwrap();
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
pub fn close_port(port: tauri::State<'_, SerialDriver>) {
    match port.close_port() {
        Ok(_) => (),
        Err(_) => (),
    }
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
        .timeout(Duration::from_millis(50))
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
