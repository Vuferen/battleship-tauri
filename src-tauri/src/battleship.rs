use rand::Rng;
use serde::Deserialize;
use serialport_manager::Port;
use std::{
    sync::{mpsc, Mutex},
    thread,
    time::Duration,
};
use tauri::{Event, Manager};

use crate::serialport_manager;
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
pub fn run_game(
    handle: tauri::AppHandle,
    port: tauri::State<'_, Port>,
    rows_state: tauri::State<'_, Rows>,
    cols_state: tauri::State<'_, Cols>,
    ship_sizes: Vec<u8>,
) {
    let rows = rows_state.0.lock().unwrap().unwrap().clone();
    let cols = cols_state.0.lock().unwrap().unwrap().clone();
    if port.0.lock().unwrap().is_some() || true {
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
        handle.listen_global("fire", move |_| {
            match tx.send(true) {
                Ok(_) => {}
                Err(err) => println!("Error stopping thread: {}", err),
            };
        });

        thread::spawn(move || {
            // Place own ships
            loop {
                // Get ship positions from arduino
                (&mut my_board).ships[0] = true;
                (&mut my_board).ships[1] = true;

                // Send positions to frontend
                handle
                    .emit_all("board-state", (&my_board).ships.clone())
                    .unwrap();

                // Check if game should start (change to listen for arduino fire)
                if rx.try_recv().is_ok() {
                    // setup_handle.unlisten(event_handler);
                    break;
                }
                thread::sleep(Duration::from_nanos(1));
            }
            handle
                .emit_all("board-state", (&my_board).ships.clone())
                .unwrap();

            let cursor_pos = 0;
            // let (tx, rx) = mpsc::channel();
            // let event_handler = handle.listen_global("fire", move |_| match tx.send(true) {
            //     Ok(_) => {}
            //     Err(err) => println!("Error stopping thread: {}", err),
            // });
            loop {
                // Game has started, wait for fire command
                if rx.try_recv().is_ok() && !(&mut their_board).hits[cursor_pos] {
                    // Handle fire
                    // Change hit state
                    (&mut their_board).hits[cursor_pos] = true;
                    if their_board.ships[cursor_pos] {
                        // Enemy ship was hit
                        their_board.ships_left -= 1;
                        handle.emit_all("enemy-board-hit", cursor_pos).unwrap();
                    } else {
                        // Miss
                        handle.emit_all("enemy-board-miss", cursor_pos).unwrap();
                    }

                    // Do enemy turn
                    let mut has_fired = false;
                    // Check if ship hit
                    for (i, hit) in (&my_board).hits.clone().iter().enumerate() {
                        if *hit {
                            // Check surrounding tiles
                            let mut target = None;
                            if i >= cols && (&my_board).hits[i - cols] {
                                // Up
                                target = Some(i - cols);
                            } else if i + 1 < (cols * rows) && (&my_board).hits[i + 1] {
                                // Right
                                target = Some(i + 1);
                            } else if (i + cols) < (cols * rows) && (&my_board).hits[i + cols] {
                                // Down
                                target = Some(i + cols);
                            } else if i >= 1 && (&my_board).hits[i - 1] {
                                // Left
                                target = Some(i - 1);
                            }
                            // Hit surrounding tile
                            if target.is_some() {
                                (&mut my_board).hits[target.unwrap()] = true;
                                (&mut my_board).ships_left -= 1;
                                has_fired = true;
                                handle.emit_all("my-board-hit", target.unwrap()).unwrap();
                            }
                        }
                    }
                    // Hit random cell
                    while !has_fired {
                        let mut rng = rand::thread_rng();
                        let pos: usize = rng.gen_range(0..=(cols * rows) - 1).into();
                        if !(&my_board).hits[pos] {
                            (&mut my_board).hits[pos] = true;
                            (&mut my_board).ships_left -= 1;
                            has_fired = true;
                            handle.emit_all("my-board-hit", pos).unwrap();
                        }
                    }

                    // Check game end condition, if ships left == 0
                    if (&my_board).ships_left == 0 {
                        // Defeat
                        handle.emit_all("game-end", false).unwrap();
                        break;
                    }
                    if (&their_board).ships_left == 0 {
                        // Victory
                        handle.emit_all("game-end", true).unwrap();
                        break;
                    }
                }
            }
        });
    }
}

#[derive(Deserialize)]
pub enum JoystickDirections {
    Up,
    Right,
    Down,
    Left,
}

#[tauri::command]
pub fn move_cursor(direction: JoystickDirections) {}

// pub fn board_state(handle: tauri::AppHandle, board: Vec<bool>) {
//     // let board = [true, false, false, true, false, false, false, false, false];
//     handle.emit_all("board-state", board).unwrap();
// }

// pub fn joystick_direction(handle: tauri::AppHandle) {
//     let direction = JoystickDirections::Right as u32;
//     handle.emit_all("joystick_direction", direction).unwrap();
// }

// pub fn joystick_fire(handle: tauri::AppHandle, fire: Option<bool>) {
//     if fire.unwrap_or(false) {
//         handle.emit_all("joystick_fire", {}).unwrap();
//     }
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
