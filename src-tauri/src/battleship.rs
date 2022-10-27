use rand::Rng;
use serde::Deserialize;
use std::{
    sync::{mpsc, Mutex},
    thread,
    time::Duration,
};
use tauri::Manager;

use crate::serialport_manager::{SerialDriver};
pub struct CursorPos(pub Mutex<Option<usize>>);
#[tauri::command]
pub fn set_cursor_pos(cursor_pos: tauri::State<'_, CursorPos>, new_pos: usize) {
    *cursor_pos.0.lock().unwrap() = Some(new_pos);
}
pub struct Cols(pub Mutex<Option<usize>>);
#[tauri::command]
pub fn set_cols(cols: tauri::State<'_, Cols>, new_cols: usize) {
    *cols.0.lock().unwrap() = Some(new_cols);
}
pub struct Rows(pub Mutex<Option<usize>>);
#[tauri::command]
pub fn set_rows(rows: tauri::State<'_, Rows>, new_rows: usize) {
    *rows.0.lock().unwrap() = Some(new_rows);
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
    is_first_game: bool,
) -> Result<bool, ()> {
    let rows = rows_state.0.lock().unwrap().unwrap().clone();
    let cols = cols_state.0.lock().unwrap().unwrap().clone();
    // let guard = port.0.unwrap().lock().unwrap();
    if is_first_game {
        port.run_port();
    }
    
    if true {
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

        let (tx, rx) = mpsc::channel();
        let fire_event = handle.listen_global("fire", move |_| {
            match tx.send(true) {
                Ok(_) => {}
                Err(err) => println!("Error stopping thread: {}", err),
            };
        });

        // thread::spawn(move || {
        // Place own ships
        loop {
            let mut fire = false;
            // Get ship positions from arduino
            let res = port.arduino_get_board();
            if res.is_ok() {
                match res.unwrap() {
                    Some(ships) => my_board.ships = ships,
                    None => fire = true,
                };
            }
            // Send positions to frontend
            handle
                .emit_all("board-state", (&my_board).ships.clone())
                .unwrap();

            // Check if game should start (change to listen for arduino fire)
            if fire || rx.try_recv().is_ok() {
                // setup_handle.unlisten(event_handler);
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        handle
            .emit_all("board-state", (&my_board).ships.clone())
            .unwrap();

        loop {
            let mut fire = false;
            // let cursor_pos = cursor_pos_state.0.lock().unwrap().unwrap();
            let cursor_pos_state_clone = cursor_pos_state.clone();
            let cursor_pos = cursor_pos_state_clone.0.lock().unwrap().unwrap();
            
            let res = port.arduino_get_joystick_direction();
            if res.is_ok() {
                match res.unwrap() {
                    Some(direction) => move_cursor_by_dir(handle.clone(), cursor_pos_state_clone, cols, rows, direction),
                    None => fire = true,
                };
            }
            
            // Game has started, wait for fire command
            if (fire || rx.try_recv().is_ok()) && !((&mut their_board).hits[cursor_pos]) {
                // Handle fire
                // Change hit state
                (&mut their_board).hits[cursor_pos] = true;
                if their_board.ships[cursor_pos] {
                    // Enemy ship was hit
                    their_board.ships_left -= 1;
                    port.arduino_vibrate().unwrap();
                    handle.emit_all("enemy-board-hit", cursor_pos).unwrap();
                } else {
                    // Miss
                    handle.emit_all("enemy-board-miss", cursor_pos).unwrap();
                }

                // Do enemy turn
                let mut has_fired = false;
                // Check if ship hit
                let mut ship_hits = vec![false; (rows * cols).into()];

                for (i, hit) in (&my_board).hits.clone().iter().enumerate() {
                    if (&my_board).ships[i] && *hit {
                        ship_hits[i] = true;
                    } else {
                        ship_hits[i] = false;
                    }
                }

                for (i, hit) in ship_hits.iter().enumerate() {
                    if *hit && !has_fired {
                        // Check surrounding tiles
                        let mut target = None;
                        if i >= cols && !(&my_board).hits[i - cols] {
                            // Up
                            target = Some(i - cols);
                        } else if (i + 1) % cols != 0 && !(&my_board).hits[i + 1] {
                            // Right
                            target = Some(i + 1);
                        } else if (i + cols) < (cols * rows) && !(&my_board).hits[i + cols] {
                            // Down
                            target = Some(i + cols);
                        } else if i % cols != 0 && !(&my_board).hits[i - 1] {
                            // Left
                            target = Some(i - 1);
                        }
                        // Hit surrounding tile
                        if target.is_some() {
                            (&mut my_board).hits[target.unwrap()] = true;
                            has_fired = true;
                            handle.emit_all("my-board-hit", target.unwrap()).unwrap();
                            if my_board.ships[target.unwrap()] {
                                port.arduino_vibrate().unwrap();
                                port.arduino_set_led(target.unwrap());
                                (&mut my_board).ships_left -= 1;
                            }
                        }
                    }
                }
                // Hit random cell
                while !has_fired {
                    let mut rng = rand::thread_rng();
                    let pos: usize = rng.gen_range(0..=(cols * rows) - 1).into();
                    if !(&my_board).hits[pos] {
                        (&mut my_board).hits[pos] = true;
                        has_fired = true;
                        handle.emit_all("my-board-hit", pos).unwrap();
                        if my_board.ships[pos] {
                            port.arduino_vibrate().unwrap();
                            port.arduino_set_led(pos);
                            (&mut my_board).ships_left -= 1;
                        }
                    }
                }

                // Check game end condition, if ships left == 0
                if (&their_board).ships_left == 0 {
                    // Victory
                    handle.emit_all("game-end", true).unwrap();
                    break;
                }
                if (&my_board).ships_left == 0 {
                    // Defeat
                    handle.emit_all("game-end", false).unwrap();
                    break;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        port.arduino_reset_leds().unwrap();
        handle.unlisten(fire_event);
        // });
    }
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
    *cursor_pos_state.0.lock().unwrap() = Some((cursor_pos as i32 + change) as usize);
    handle
        .emit_all("update-cursor-pos", cursor_pos as i32 + change)
        .unwrap();
}

fn move_cursor_by_dir(
    handle: tauri::AppHandle,
    cursor_pos_state: tauri::State<'_, CursorPos>,
    cols: usize,
    rows: usize,
    joystick_direction: JoystickDirections) {
    let cursor_pos = cursor_pos_state.0.lock().unwrap().unwrap();
    let change: i32;
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
    *cursor_pos_state.0.lock().unwrap() = Some((cursor_pos as i32 + change) as usize);
    handle
        .emit_all("update-cursor-pos", cursor_pos as i32 + change)
        .unwrap();
}

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
