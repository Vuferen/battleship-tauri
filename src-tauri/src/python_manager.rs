use pyo3::prelude::*;

pub fn get_ships() -> Vec<bool> {
    let mut output = vec![false; 100];
    Python::with_gil(|py| {
        // This path is resolved relative to this file.
        let code = include_str!("../scanner/scanner.py");
        let my_module = PyModule::from_code(py, code, "scanner", "scanner").unwrap();
        // Python::with_gil(|py| -> PyResult<()> {
        //     my_module = PyModule::from_code(py, code, "scanner", "positions").unwrap();
        //     Ok(())
        // }).unwrap();
        let res: Vec<i32> = my_module.getattr("refresh").unwrap().call0().unwrap().extract().unwrap();
        
        for i in 0..100 {
            if res[i] == 1 {
                output[i] = true;
            }
        }
        // for i in 0..10 {
        //     for j in 0..10 {
        //         print!("{} ", res[i*10+j]);
        //     }
        //     println!("");
        // }
    });
    
    return output;
}