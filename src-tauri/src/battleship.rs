use rand::{Rng};
use serde::Deserialize;
use std::{
    sync::{mpsc, Mutex},
    thread,
    time::{Duration, Instant},
};
use tauri::Manager;
// use soloud::*;
// use std::fs::File;
// use std::io::BufReader;
// use rodio::{Decoder, OutputStream, source::Source, Sink};

use crate::{serialport_manager::InputTag, vector2::*};
use crate::python_manager::get_ships;
use crate::serialport_manager::{JoystickDirection, SerialDriver};
pub struct CursorPos(pub Mutex<Option<usize>>);
#[tauri::command]
pub fn set_cursor_pos(cursor_pos: tauri::State<'_, CursorPos>, new_pos: usize) {
    // *cursor_pos.0.lock().unwrap() = Some(new_pos);
    if let Ok(mut state) = cursor_pos.0.lock() {
        *state = Some(new_pos);
    }
}
pub struct Cols(pub Mutex<Option<usize>>);
#[tauri::command]
pub fn set_cols(cols: tauri::State<'_, Cols>, new_cols: usize) {
    // *cols.0.lock().unwrap() = Some(new_cols);
    if let Ok(mut state) = cols.0.lock() {
        *state = Some(new_cols);
    }
}
pub struct Rows(pub Mutex<Option<usize>>);
#[tauri::command]
pub fn set_rows(rows: tauri::State<'_, Rows>, new_rows: usize) {
    if let Ok(mut state) = rows.0.lock() {
        *state = Some(new_rows);
    }
}
struct Board {
    ships: Vec<bool>,
    hits: Vec<bool>,
    ships_left: u8,
}

#[tauri::command]
pub async fn run_game(
    handle: tauri::AppHandle,
    port: tauri::State<'_, SerialDriver>,
    cursor_pos_state: tauri::State<'_, CursorPos>,
    rows_state: tauri::State<'_, Rows>,
    cols_state: tauri::State<'_, Cols>,
    ship_sizes: Vec<u8>,
    cam_id: usize,
) -> Result<bool, String> {
    let rows = rows_state.0.lock().unwrap().unwrap_or(10).clone();
    let cols = cols_state.0.lock().unwrap().unwrap_or(10).clone();
    // let guard = port.0.unwrap().lock().unwrap();
    if let Err(err) = port.run_port() {
        return Err(err);
    }


    let mut is_my_turn = false;

    let mut total_ships = 0;
    for ship in &ship_sizes {
        total_ships += ship;
    }

    let mut my_board = Board {
        ships: vec![false; (rows * cols).into()],
        hits: vec![false; (rows * cols).into()],
        ships_left: total_ships,
    };
    let mut their_board = Board {
        ships: vec![false; (rows * cols).into()],
        hits: vec![false; (rows * cols).into()],
        ships_left: total_ships,
    };

    //Place enemy ships
    {
        for ship in &ship_sizes {
            let mut ship_placed = false;
            while !ship_placed {
                let mut rng = rand::thread_rng();
                let pos: usize = rng.gen_range(0..=(cols * rows) - 1).into();
                let rot: usize = rng.gen_range(0..=3);

                if !their_board.ships[pos] {
                    match rot {
                        0 => {
                            //place ship going up from pos
                            if pos > cols * (*ship - 1) as usize {
                                if try_place_ship(
                                    &(*ship as usize),
                                    &mut their_board,
                                    pos,
                                    cols,
                                    &|pos: usize, i: usize, cols: usize| pos - i * cols,
                                ) {
                                    ship_placed = true;
                                }
                            }
                        }
                        1 => {
                            //place ship going right from pos
                            if pos % cols <= cols - (*ship as usize) {
                                if try_place_ship(
                                    &(*ship as usize),
                                    &mut their_board,
                                    pos,
                                    cols,
                                    &|pos: usize, i: usize, _cols: usize| pos + i,
                                ) {
                                    ship_placed = true;
                                }
                            }
                        }
                        2 => {
                            //place ship going down from pos
                            if pos + cols * (*ship - 1) as usize <= cols * rows - 1 {
                                if try_place_ship(
                                    &(*ship as usize),
                                    &mut their_board,
                                    pos,
                                    cols,
                                    &|pos: usize, i: usize, cols: usize| pos + i * cols,
                                ) {
                                    ship_placed = true;
                                }
                            }
                        }
                        3 => {
                            //place ship going left from pos
                            if pos % cols > (*ship - 1) as usize {
                                if try_place_ship(
                                    &(*ship as usize),
                                    &mut their_board,
                                    pos,
                                    cols,
                                    &|pos: usize, i: usize, _cols: usize| pos - i,
                                ) {
                                    ship_placed = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Listen to the fire event from the UI
    let (tx, rx) = mpsc::channel();
    let fire_event = handle.listen_global("fire", move |_| {
        match tx.send(true) {
            Ok(_) => {}
            Err(err) => println!("Error on fire: {}", err),
        };
    });

    // Listen to the reset event from the UI
    let mut restart = false;
    let (restart_send, restart_recv) = mpsc::channel();
    let restart_event = handle.listen_global("restart_game", move |_| {
        match restart_send.send(true) {
            Ok(_) => {},
            Err(err) => println!("Error on restart: {}", err),
        };
    });

    // thread::spawn(move || {

    // Place own ships and wait for opponents ships
    let mut is_own_ships_placed = false;
    let mut is_opponents_ships_placed = false;
    
    // let ships = vec![false; 100];
    let (py_tx, py_rx) = mpsc::channel();
    let (py_send_exit, py_recv_exit) = mpsc::channel();
    let (py_send_err, py_recv_err) = mpsc::channel();
    // println!("Get_ships");
    get_ships(py_tx, py_recv_exit, py_send_err, cam_id);

    loop {
        // Check if game should restart
        if restart_recv.try_recv().is_ok() {
            println!("Restarting!");
            restart = true;
            break;
        }

        if !is_own_ships_placed {
            _ = handle.emit_all("game-state", "Setup");
        }

        let mut fire = false;

        
        let py_res = py_rx.try_recv();
        if py_res.is_ok() && !is_own_ships_placed {
            my_board.ships = py_res.unwrap();
        }

        if let Ok(err) = py_recv_err.try_recv() {
            println!("Py error: {}", err);
            _ = handle.emit_all("error", err);
            // restart = true;
            // get_ships(py_tx, py_recv_exit, py_send_err);
        }

        // Get board from camera
        // println!("Start python");
        // println!("End python");
        // let mut ships = vec![false; 100];
        // ships[0] = true;
        // ships[1] = true;
        // ships[87] = true;
        // ships[88] = true;
        // ships[89] = true;
        // my_board.ships = ships.clone();

        // Check for fire input from arduino
        // println!("Start fire");
        // let fire_res = port.arduino_try_get_fire();
        if let Some(fire_res) = port.arduino_try_get_fire() {
            fire = fire_res;
            println!("Fire: {}", fire);
        }
        // println!("End fire");
        // let fire_res = port.arduino_get_joystick_direction();
        // if fire_res.is_some() {
        //     fire = fire_res.unwrap();
        //     // fire = input.tag == InputTag::Fire;
        //     println!("Fire: {}", fire);
        // }

        // Check to confirm ship positions (change to listen for arduino fire)
        if (fire || rx.try_recv().is_ok()) && !is_own_ships_placed {
            // Check that all ships have been placed
            let mut placed_ships = 0;
            for ship in my_board.ships.clone() {
                if ship {
                    placed_ships += 1;
                }
            }

            if placed_ships == total_ships {
                // let res = port.arduino_send_board(my_board.ships.clone());
                // if res.is_ok() {
                if let Ok(input) = port.arduino_send_board(my_board.ships.clone()) {
                    // let input = res.unwrap();
                    if input.tag == InputTag::Turn {
                        is_my_turn = true;
                    }
                }
                is_own_ships_placed = true;
                _ = handle.emit_all("game-state", "WaitSetup");
            }
        }

        // println!("Trying to get board");
        // let res = port.arduino_try_get_board();
        // if res.is_ok() {
        if let Ok(input) = port.arduino_try_get_board() {
            // let input = res.unwrap();
            if input.tag == InputTag::Board {
                println!("Got board");
                let mut num_ships = 0;
                for ship in &input.ships {
                    if *ship {
                        num_ships += 1;
                    }
                }
                if num_ships == total_ships {
                    println!("All ships placed");
                    their_board.ships = input.ships;
                    is_opponents_ships_placed = true;
                } else {
                    println!("Missing ships");
                }
            } else if input.tag == InputTag::Reset {
                println!("Got reset");
                restart = true;
                break;
            }
        }

        // Send positions to frontend
        _ = handle.emit_all("board-state", (&my_board).ships.clone());

        // Check if game should start
        if is_own_ships_placed && is_opponents_ships_placed {
            if is_my_turn {
                _ = handle.emit_all("game-state", "YourTurn");
            } else {
                _ = handle.emit_all("game-state", "OtherTurn");
            }
            break;
        }

        // thread::sleep(Duration::from_millis(10));
    }
    _ = py_send_exit.send(true);

    if !restart {
        _ = handle.emit_all("board-state", (&my_board).ships.clone());

        // Avoid missfire
        thread::sleep(Duration::from_millis(500));
    }

    // Game loop
    let mut arduino_end = false;
    let mut now = Instant::now();
    let mut cursor = Vector2 { x: 0.0, y: 0.0 };
    while !restart {
        // Check if game should restart
        if restart_recv.try_recv().is_ok() {
            println!("Restarting!");
            // restart = true;
            break;
        }

        let mut fire = false;

        // if !is_my_turn && port.arduino_try_get_turn().is_some() {
        //     is_my_turn = true;
        //     handle.emit_all("game-state", "YourTurn").unwrap();
        // }

        // let cursor_pos = cursor_pos_state.0.lock().unwrap().unwrap();
        // let cursor_pos;
        let cursor_pos_state_clone = cursor_pos_state.clone();
        let cursor_pos_state_clone_2 = cursor_pos_state.clone();
        // let cursor_pos = cursor_pos_state_clone.0.lock().unwrap().unwrap();
        // if let Ok(state_option) = cursor_pos_state_clone.0.lock() {
        //     if state_option.is_some() {
        //         cursor_pos = state_option.unwrap();
        //     }
        // }
        // let res = port.arduino_get_joystick_direction();

        // if res.is_ok() {
        _ = handle.emit_all("error", "Trying to get dir");
        if let Ok(input) = port.arduino_get_joystick_direction() {
            _ = handle.emit_all("error", "Got dir");
            // let input = res.unwrap();
            match input.tag {
                InputTag::Reset => restart = true,
                InputTag::Board => (),
                InputTag::Fire => fire = true,
                InputTag::Joystick => {
                    if now.elapsed() > Duration::from_millis(10) {
                        now = Instant::now();
                        move_cursor_by_dir(
                            handle.clone(),
                            cursor_pos_state_clone,
                            &mut cursor,
                            cols,
                            rows,
                            input.joystick_direction,
                        );
                    }
                }
                InputTag::End => {
                    arduino_end = true;
                    _ = handle.emit_all("game-state", "Defeat");
                    break;
                }
                InputTag::Turn => {
                    is_my_turn = true;
                    _ = handle.emit_all("game-state", "YourTurn");
                }
            }
            // if input.turn.is_some() {
            if let Some(turn) = input.turn {
                is_my_turn = turn;
                if turn {
                _ = handle.emit_all("game-state", "YourTurn");
                }
            }
        }

        if let Ok(state) = cursor_pos_state_clone_2.0.lock() {
            if state.is_some() {
                let cursor_pos = state.unwrap();
            // }
            // if let Some(cursor_pos) = state {
                // is_my_turn = true;
                // Do turn
                if is_my_turn {
                    // Game has started, wait for fire command
                    if (fire || rx.try_recv().is_ok()) && !((&mut their_board).hits[cursor_pos]) {
                        // Handle fire
                        // Change hit state
                        (&mut their_board).hits[cursor_pos] = true;
                        if their_board.ships[cursor_pos] {
                            // Enemy ship was hit
                            their_board.ships_left -= 1;
                            _ = handle.emit_all("enemy-board-hit", cursor_pos);
                            thread::sleep(Duration::from_millis(1750));
                            if let Err(err) = port.arduino_hit(cursor_pos) {
                                println!("{err}");
                                _ = handle.emit_all("error", err);
                            }
                        } else {
                            // Miss
                            _ = handle.emit_all("enemy-board-miss", cursor_pos);
                            thread::sleep(Duration::from_millis(1750));
                            if let Err(err) = port.arduino_miss(cursor_pos) {
                                println!("{err}");
                                _ = handle.emit_all("error", err);
                            }
                        }
                        is_my_turn = false;
                        _ = handle.emit_all("game-state", "OtherTurn");
                    }
                }
            }
        }

        // Check game end condition, if ships left == 0
        if (&their_board).ships_left == 0 {
            // Victory
            _ = handle.emit_all("game-state", "Win");
            break;
        }
        if (&my_board).ships_left == 0 {
            // Defeat
            _ = handle.emit_all("game-state", "Defeat");
            break;
        }
    }

    if restart {
        _ = handle.emit_all("game-state", "PreSetup");
        if let Err(err) = port.arduino_reset() {
            println!("{err}");
            _ = handle.emit_all("error", err);
        }
    } else {
        // Avoid send end back and forth
        if !arduino_end {
            if let Err(err) = port.arduino_end() {
                println!("{err}");
                _ = handle.emit_all("error", err);
            }
        }
        // Wait for fire button to be pressed before restarting
        loop {
            // let mut fire = false;
            // let fire_res = port.arduino_try_get_fire();
            // if fire_res.is_some() {
            if let Some(_) = port.arduino_try_get_fire() {
                // fire = fire_res.unwrap();
                break;
            }
            if rx.try_recv().is_ok() {
                break;
            }   
        }
        if let Err(err) = port.arduino_reset() {
            println!("{err}");
            _ = handle.emit_all("error", err);
        }
    }

    match port.close_port() {
        Ok(_) => (),
        Err(_) => (),
    }
    handle.unlisten(fire_event);
    handle.unlisten(restart_event);

    return Ok(true);
}

#[derive(Deserialize)]
pub enum JoystickDirections {
    Up,
    Right,
    Down,
    Left,
    Stay,
}

#[tauri::command]
pub fn move_cursor(
    handle: tauri::AppHandle,
    cursor_pos_state: tauri::State<'_, CursorPos>,
    cols_state: tauri::State<'_, Cols>,
    rows_state: tauri::State<'_, Rows>,
    direction: i32,
) {
    let cursor_pos = cursor_pos_state.0.lock().unwrap().unwrap();
    let cols = cols_state.0.lock().unwrap().unwrap().clone();
    let rows = rows_state.0.lock().unwrap().unwrap().clone();
    let change: i32;
    let joystick_direction = match direction {
        0 => JoystickDirections::Up,
        1 => JoystickDirections::Right,
        2 => JoystickDirections::Down,
        3 => JoystickDirections::Left,
        4 => JoystickDirections::Stay,
        _ => return,
    };
    match joystick_direction {
        JoystickDirections::Down => {
            if cursor_pos < cols {
                change = (cols * (rows - 1)) as i32;
            } else {
                change = -(cols as i32);
            }
        }
        JoystickDirections::Right => {
            if (cursor_pos + 1) % cols != 0 {
                change = 1;
            } else {
                change = -(cols as i32) + 1;
            }
        }
        JoystickDirections::Up => {
            if cursor_pos + cols > cols * rows - 1 {
                change = -(cols as i32) * ((rows as i32) - 1)
            } else {
                change = cols as i32;
            }
        }
        JoystickDirections::Left => {
            if cursor_pos % cols != 0 {
                change = -1;
            } else {
                change = cols as i32 - 1;
            }
        }
        JoystickDirections::Stay => {
            change = 0;
        }
    }
    if let Ok(mut state) = cursor_pos_state.0.lock(){
        *state = Some((cursor_pos as i32 + change) as usize);
    }
    _ = handle
        .emit_all("update-cursor-pos", cursor_pos as i32 + change);
}

fn move_cursor_by_dir(
    handle: tauri::AppHandle,
    cursor_pos_state: tauri::State<'_, CursorPos>,
    cursor: &mut Vector2,
    cols: usize,
    rows: usize,
    joystick_direction: JoystickDirection,
) {
    // let cursor_pos = cursor_pos_state.0.lock().unwrap().unwrap();
    let mut dir = Vector2 {
        x: joystick_direction.x as f32,
        y: joystick_direction.y as f32,
    };
    dir = dir.normalize();
    cursor.x = (cursor.x - dir.x * 0.01).clamp(-1.0, 1.0);
    cursor.y = (cursor.y - dir.y * 0.01).clamp(-1.0, 1.0);
    // let change: i32;
    // match joystick_direction {
    //     JoystickDirections::Down => {
    //         cursor.y = (cursor.y - 0.01).max(-0.99);
    //     }
    //     JoystickDirections::Right => {
    //         cursor.x = (cursor.x + 0.01).min(0.99);
    //     }
    //     JoystickDirections::Up => {
    //         cursor.y = (cursor.y + 0.01).min(0.99);
    //     }
    //     JoystickDirections::Left => {
    //         cursor.x = (cursor.x - 0.01).max(-0.99);
    //     }
    //     JoystickDirections::Stay => {
    //         // change = 0;
    //     }
    // }
    // *cursor_pos_state.0.lock().unwrap() = Some(cursor.selected(rows, cols));
    if let Ok(mut cursor_pos) = cursor_pos_state.0.lock() {
        *cursor_pos = Some(cursor.selected(rows, cols));
    }

    _ = handle.emit_all("update-2d-cursor-pos", *cursor);
    _ = handle.emit_all("update-cursor-pos", cursor.selected(rows, cols));
}

// fn move_cursor_by_dir(
//     handle: tauri::AppHandle,
//     cursor_pos_state: tauri::State<'_, CursorPos>,
//     cols: usize,
//     rows: usize,
//     joystick_direction: JoystickDirections) {
//     let cursor_pos = cursor_pos_state.0.lock().unwrap().unwrap();
//     let change: i32;
//     match joystick_direction {
//         JoystickDirections::Down => {
//             if cursor_pos < cols {
//                 change = (cols * (rows - 1)) as i32;
//             } else {
//                 change = -(cols as i32);
//             }
//         }
//         JoystickDirections::Right => {
//             if (cursor_pos + 1) % cols != 0 {
//                 change = 1;
//             } else {
//                 change = -(cols as i32) + 1;
//             }
//         }
//         JoystickDirections::Up => {
//             if cursor_pos + cols > cols * rows - 1 {
//                 change = -(cols as i32) * ((rows as i32) - 1)
//             } else {
//                 change = cols as i32;
//             }
//         }
//         JoystickDirections::Left => {
//             if cursor_pos % cols != 0 {
//                 change = -1;
//             } else {
//                 change = cols as i32 - 1;
//             }
//         }
//         JoystickDirections::Stay => {
//             change = 0;
//         }
//     }
//     *cursor_pos_state.0.lock().unwrap() = Some((cursor_pos as i32 + change) as usize);
//     handle
//         .emit_all("update-cursor-pos", cursor_pos as i32 + change)
//         .unwrap();
// }

fn try_place_ship(
    ship: &usize,
    board: &mut Board,
    pos: usize,
    cols: usize,
    f: &dyn Fn(usize, usize, usize) -> usize,
) -> bool {
    let mut can_place = true;
    for i in 0..*ship {
        if board.ships[f(pos, i, cols)] {
            can_place = false;
        }
    }
    if can_place {
        for i in 0..*ship {
            board.ships[f(pos, i, cols)] = true;
        }
        return true;
    }
    return false;
}

// // Do enemy turn
// let mut has_fired = false;
// // Check if ship hit
// let mut ship_hits = vec![false; (rows * cols).into()];

// for (i, hit) in (&my_board).hits.clone().iter().enumerate() {
//     if (&my_board).ships[i] && *hit {
//         ship_hits[i] = true;
//     } else {
//         ship_hits[i] = false;
//     }
// }

// for (i, hit) in ship_hits.iter().enumerate() {
//     if *hit && !has_fired {
//         // Check surrounding tiles
//         let mut target = None;
//         if i >= cols && !(&my_board).hits[i - cols] {
//             // Up
//             target = Some(i - cols);
//         } else if (i + 1) % cols != 0 && !(&my_board).hits[i + 1] {
//             // Right
//             target = Some(i + 1);
//         } else if (i + cols) < (cols * rows) && !(&my_board).hits[i + cols] {
//             // Down
//             target = Some(i + cols);
//         } else if i % cols != 0 && !(&my_board).hits[i - 1] {
//             // Left
//             target = Some(i - 1);
//         }
//         // Hit surrounding tile
//         if target.is_some() {
//             (&mut my_board).hits[target.unwrap()] = true;
//             has_fired = true;
//             handle.emit_all("my-board-hit", target.unwrap()).unwrap();
//             if my_board.ships[target.unwrap()] {
//                 // port.arduino_vibrate().unwrap();
//                 // port.arduino_hit(target.unwrap());
//                 (&mut my_board).ships_left -= 1;
//             }
//         }
//     }
// }
// Hit random cell
// while !has_fired {
//     let mut rng = rand::thread_rng();
//     let pos: usize = rng.gen_range(0..=(cols * rows) - 1).into();
//     if !(&my_board).hits[pos] {
//         (&mut my_board).hits[pos] = true;
//         has_fired = true;
//         handle.emit_all("my-board-hit", pos).unwrap();
//         if my_board.ships[pos] {
//             // port.arduino_vibrate().unwrap();
//             // port.arduino_set_led(pos);
//             (&mut my_board).ships_left -= 1;
//         }
//     }
// }
