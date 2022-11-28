use std::{sync::mpsc::{Sender, Receiver}, thread::{self, JoinHandle}};
// use tauri::async_runtime::channel;

use tauri::api::process::{Command};

pub fn get_ships(py_tx: Sender<Vec<bool>>, exit: Receiver<bool>) -> JoinHandle<()> {

    let handle = thread::spawn(move || {
        loop {
            if exit.try_recv().is_ok() {
                break;
            }

            let output = Command::new_sidecar("scanner").unwrap().output().unwrap();
            let line = output.stdout;

            let start_index = line.find('[');
            let end_index = line.find(']');
            if start_index.is_some() && end_index.is_some() {
                let arr_string = line[start_index.unwrap()+1..end_index.unwrap()].to_string();
                // println!("{}", arr_string);
                let cells = arr_string.split(", ").filter_map(|s| s.parse::<i32>().ok())
                .collect::<Vec<_>>();
                
                let mut ships = vec![false; 100];
                for (i, cell) in cells.iter().enumerate() {
                    ships[i] = *cell == 1;
                }
                ships.reverse(); // 180deg rotation
                py_tx.send(ships).unwrap();
            }







            // let (mut rx, mut _child) = Command::new_sidecar("scanner")
            // .expect("failed to create `scanner` binary command")
            // .spawn()
            // .expect("Failed to spawn sidecar");

            // let (async_send, mut async_recv) = channel(1000);

            // tauri::async_runtime::spawn(async move {
            //     // read events such as stdout
            //     // let joined = future::join(rx.recv());
            //     while let Some(event) = rx.recv().await {
            //         if let CommandEvent::Stdout(line) = event {
            //             let start_index = line.find('[');
            //             let end_index = line.find(']');
            //             if start_index.is_some() && end_index.is_some() {
            //                 println!("Got ships");
            //                 let arr_string = line[start_index.unwrap()..end_index.unwrap()].to_string();
            //                 let cells = arr_string.split(", ").filter_map(|s| s.parse::<i32>().ok())
            //                 .collect::<Vec<_>>();
                            
            //                 let mut ships = vec![false; 100];
            //                 for (i, cell) in cells.iter().enumerate() {
            //                     ships[i] = *cell == 1;
            //                     print!("{}", *cell);
            //                 }
            //                 println!("\n");
            //                 async_send.send(ships).await.unwrap();
            //             }

            //             println!("{}",line);
            //         }
            //     }
            // });



            // let res = async_recv.try_recv();
            // if res.is_ok() {
            //     py_tx.send(res.unwrap()).unwrap();
            // }


            // println!("Here2");
            // // `new_sidecar()` expects just the filename, NOT the whole path like in JavaScript
            // let (mut rx, mut _child) = Command::new_sidecar("scanner")
            // .expect("failed to create `scanner` binary command")
            // .spawn()
            // .expect("Failed to spawn sidecar");
            // println!("Here3");
            // // read events such as stdout
            // println!("Here");
            // let res = rx.try_recv();
            // if res.is_ok() {
            //     let event = res.unwrap();
            //     println!("Async!");
            //     if let CommandEvent::Stdout(line) = event {
            //         let start_index = line.find('[');
            //         let end_index = line.find(']');
            //         if start_index.is_some() && end_index.is_some() {
            //             println!("Got ships");
            //             let arr_string = line[start_index.unwrap()..end_index.unwrap()].to_string();
            //             let cells = arr_string.split(", ").filter_map(|s| s.parse::<i32>().ok())
            //             .collect::<Vec<_>>();
                        
            //             let mut ships = vec![false; 100];
            //             for (i, cell) in cells.iter().enumerate() {
            //                 ships[i] = *cell == 1;
            //                 print!("{}", *cell);
            //             }
            //             println!("\n");
            //             py_tx.send(ships).unwrap();
            //         }

            //         println!("{}",line);
            //     // window
            //     //     .emit("message", Some(format!("'{}'", line)))
            //     //     .expect("failed to emit event");
            //     // }
            //     }
            // }
            // ships = vec![false; 100];
        }
    });
    return handle;
}
//"./libcrypto-1_1.dll", "libffi-7.dll", "libssl-1_1.dll", "python3.dll", "./python310.dll", "sqlite3.dll", "vcruntime140.dll", "vcruntime140_1.dll"
// use pyo3::prelude::*;

// pub fn get_ships() -> Vec<bool> {
//     let mut output = vec![false; 100];
//     Python::with_gil(|py| {
//         // This path is resolved relative to this file.
//         let code = include_str!("../scanner/scanner.py");
//         let my_module = PyModule::from_code(py, code, "scanner", "scanner").unwrap();
//         // Python::with_gil(|py| -> PyResult<()> {
//         //     my_module = PyModule::from_code(py, code, "scanner", "positions").unwrap();
//         //     Ok(())
//         // }).unwrap();
//         let res: Vec<i32> = my_module.getattr("GetShips").unwrap().call0().unwrap().extract().unwrap();
        
//         for i in 0..100 {
//             if res[i] == 1 {
//                 output[i] = true;
//             }
//         }
//         // for i in 0..10 {
//         //     for j in 0..10 {
//         //         print!("{} ", res[i*10+j]);
//         //     }
//         //     println!("");
//         // }
//     });
    
//     return output;
// }

// use rustpython_vm as vm;


// macro_rules! add_python_function {
//     ( $scope:ident, $vm:ident, $src:literal $(,)? ) => {{
//         // compile the code to bytecode
//         let code = vm::py_compile!(source = $src);
//         // convert the rustpython_compiler_core::CodeObject to a PyRef<PyCode>
//         let code = $vm.ctx.new_code(code);

//         // run the python code in the scope to store the function
//         $vm.run_code_obj(code, $scope.clone())
//     }};
// }

// pub fn get_ships() -> Vec<bool> {
//     vm::Interpreter::without_stdlib(Default::default()).enter(|vm| {
//         let scope = vm.new_scope_with_builtins();

//         let code_obj = vm
//             .compile(
//                 r#"print("Hello World!")"#,
//                 vm::compiler::Mode::Exec,
//                 "<embedded>".to_owned(),
//             )
//             .map_err(|err| vm.new_syntax_error(&err))?;

//         vm.run_code_obj(code_obj, scope)?;

//         Ok(())
//     })
// }


// use std::str;

// use wasmer_runtime::{
//     imports,
//     func,
//     instantiate,
//     error,
//     Ctx,
// };

// // Make sure that the compiled wasm-sample-app is accessible at this path.
// static WASM: &'static [u8] = include_bytes!("../scanner/scanner.wasm");

// pub fn get_ships() -> Vec<bool> {
//     // Let's define the import object used to import our function
//     // into our webassembly sample application.
//     //
//     // We've defined a macro that makes it super easy.
//     //
//     // The signature tells the runtime what the signature (the parameter
//     // and return types) of the function we're defining here is.
//     // The allowed types are `i32`, `u32`, `i64`, `u64`,
//     // `f32`, and `f64`.
//     //
//     // Make sure to check this carefully!
//     // let import_object = imports! {
//     //     // Define the "env" namespace that was implicitly used
//     //     // by our sample application.
//     //     "env" => {
//     //         // name        // the func! macro autodetects the signature
//     //         "print_str" => func!(print_str),
//     //     },
//     // };

//     let import_object = imports! {};
//     // Compile our webassembly into an `Instance`.
//     let instance = instantiate(WASM, &import_object)?;

//     // Call our exported function!
//     let res = instance.call("GetShips", &[]);
//     let ships: Option<Vec<i32>> = None;
//     match res {
//         Ok(items) => ships = Some(items),
//         Err(err) => println!("Could not get ships: {}", err),
//     }

//     let mut output = vec![false; 100];
//     if ships.is_some() {
//         let input = ships.unwrap();
//         for i in 0..100 {
//             if input[i] == 1 {
//                 output[i] = true;
//             }
//         }>
//     }


//     return output;
// }

// use cpython::{Python, PyDict, PyResult};

// pub fn get_ships() -> Vec<bool> {
//     let gil = Python::acquire_gil();
//     hello(gil.python()).unwrap();
//     return vec![false; 100];
// }

// fn hello(py: Python) -> PyResult<()> {
//     let scanner = py.import("scanner")?;
//     let res: Vec<i32> = scanner.get(py, "GetShips")?.extract(py)?;

//     // let locals = PyDict::new(py);
//     // locals.set_item(py, "os", py.import("os")?)?;
//     // let user: String = py.eval("os.getenv('USER') or os.getenv('USERNAME')", None, Some(&locals))?.extract(py)?;

//     // println!("Hello {}, I'm Python {}", user, version);
//     for i in 0..10 {
//         for j in 0..10 {
//             print!("{} ", res[i*10+j]);
//         }
//         println!("");
//     }
//     println!("-----------------");
//     Ok(())
// }