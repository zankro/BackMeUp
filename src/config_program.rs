#![windows_subsystem = "windows"]

mod display_window;

use std::env;

fn main(){

    // Retrieve arguments
    let args: Vec<String> = env::args().collect();
    let last_arg = args.last().unwrap();

    match last_arg.as_str() {
        "backup" => {
            if let Err(e) = display_window::show_backup_gui() {
                eprintln!("Errore nella generazione della GUI: {}", e);
            }
        },
        "config" => {
            if let Err(e) = display_window::show_gui_if_needed() {
                eprintln!("Errore nella generazione della GUI: {}", e);
            }
        },
        _ => {
            eprintln!("Last arg not present")
        }
    }
}