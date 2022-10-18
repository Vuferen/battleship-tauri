use rand::Rng;
use serialport_manager::Port;
use std::{
    sync::{mpsc, Mutex},
    thread,
    time::Duration,
};
use tauri::Manager;

use crate::serialport_manager;

struct Board {
    ships: Vec<bool>,
    hits: Vec<bool>,
    ships_left: u8,
}

#[tauri::command]
pub fn run_game(
    handle: tauri::AppHandle,
    port: tauri::State<'_, Port>,
    rows: u8,
    cols: u8,
    ship_sizes: Vec<u8>,
) {
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

        // Setup stage
        let (tx, rx) = mpsc::channel();
        let event_handler = handle.listen_global("confirm-ships", move |_| {
            match tx.send(true) {
                Ok(_) => {}
                Err(err) => println!("Error stopping thread: {}", err),
            };
        });

        let setup = thread::spawn(move || {
            // Place own ships
            loop {
                // Get ship positions from arduino
                my_board.ships[0] = true;
                my_board.ships[1] = true;

                // Send positions to frontend
                handle
                    .emit_all("board-state", my_board.ships.clone())
                    .unwrap();

                // Check if game should start (change to listen for arduino fire)
                if rx.try_recv().is_ok() {
                    handle.unlisten(event_handler);
                    break;
                }
                thread::sleep(Duration::from_nanos(1));
            }

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
                                    if pos as u8 > cols * (ship - 1) {
                                        if try_place_ship(
                                            &(*ship as usize),
                                            &mut their_board,
                                            pos,
                                            cols as usize,
                                            &|pos: usize, i: usize, cols: usize| pos - i * cols,
                                        ) {
                                            ship_placed = true;
                                        }
                                    }
                                }
                                1 => {
                                    //place ship going right from pos
                                    if pos as u8 % cols <= cols - ship {
                                        if try_place_ship(
                                            &(*ship as usize),
                                            &mut their_board,
                                            pos,
                                            cols as usize,
                                            &|pos: usize, i: usize, _cols: usize| pos + i,
                                        ) {
                                            ship_placed = true;
                                        }
                                    }
                                }
                                2 => {
                                    //place ship going down from pos
                                    if pos as u8 + cols * (ship - 1) <= cols * rows - 1 {
                                        if try_place_ship(
                                            &(*ship as usize),
                                            &mut their_board,
                                            pos,
                                            cols as usize,
                                            &|pos: usize, i: usize, cols: usize| pos + i * cols,
                                        ) {
                                            ship_placed = true;
                                        }
                                    }
                                }
                                3 => {
                                    //place ship going left from pos
                                    if pos as u8 % cols > (ship - 1) {
                                        if try_place_ship(
                                            &(*ship as usize),
                                            &mut their_board,
                                            pos,
                                            cols as usize,
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
            handle.emit_all("board-state", my_board.ships).unwrap();
        });

        // Game loop
        thread::spawn(move || {
            setup.join().unwrap();
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
                // if my_board.ships_left == 0 {
                //     // Defeat
                // }
                // if their_board.ships_left == 0 {
                //     // Victory
                // }
                break;
            }
        });
    }
}

enum JoystickDirections {
    Up,
    Right,
    Down,
    Left,
}

// pub fn board_state(handle: tauri::AppHandle, board: Vec<bool>) {
//     // let board = [true, false, false, true, false, false, false, false, false];
//     handle.emit_all("board-state", board).unwrap();
// }

pub fn joystick_direction(handle: tauri::AppHandle) {
    let direction = JoystickDirections::Right as u32;
    handle.emit_all("joystick_direction", direction).unwrap();
}

pub fn joystick_fire(handle: tauri::AppHandle, fire: Option<bool>) {
    if fire.unwrap_or(false) {
        handle.emit_all("joystick_fire", {}).unwrap();
    }
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
