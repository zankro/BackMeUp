#![windows_subsystem = "windows"]

use std::{env, process, thread};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use sysinfo::Pid;
use eframe::{egui, Frame};

mod cpu_evaluation;
mod mouse_tracker;
mod audio;
mod backup;
mod display_window;

#[cfg(target_os = "windows")]
fn get_screen_resolution() -> (usize, usize){

    use windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics;
    use windows::Win32::UI::WindowsAndMessaging::{SM_CXSCREEN, SM_CYSCREEN};

    let mut width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    width = width + 25*width/100;
    let mut height = unsafe { GetSystemMetrics(SM_CYSCREEN)};
    height = height + 25*height/100;
    (width as usize, height as usize)
}

#[cfg(target_os = "macos")]
fn get_screen_resolution() -> (usize, usize){
    use core_graphics::display::CGDisplay;

    let display = CGDisplay::main();
    (display.pixels_wide() as usize, display.pixels_high() as usize)
}

#[cfg(target_os = "linux")]
fn get_screen_resolution() -> (u32, u32){
    use x11::xlib;
    use std::ptr;

    unsafe{
        let display = xlib::XOpenDisplay(ptr::null());
        let screen = xlib::XDefaultScreen(display);
        let width = xlib::XDisplayWidth(display, screen) as u32;
        let height = xlib::XDisplayHeight(display, screen) as u32;
        xlib::XCloseDisplay(display);
        (width, height)
    }
}

fn main() {
    let exe_path: PathBuf = PathBuf::from(env::current_exe().unwrap().parent().unwrap());
    let config_program_path = exe_path.join("config_program");
    let config_file_path = exe_path.parent().unwrap().join("Resources/");

    println!("Config file path: {:?}", config_file_path.join("config.toml"));

    // Check if config.toml exists.
    // If not, start the config program. This is done in case system is rebooted, backup_program service is started but the config.toml is deleted.
    if !config_file_path.join("config.toml").exists() {
        Command::new(config_program_path).arg("config").spawn().expect("Failed to start config program");
    }

    /* Start the actual backup program */

    // Get the monitor resolution
    let (width, height) = get_screen_resolution();
    println!("Screen resolution: {}, {}", width, height);

    mouse_tracker::track_mouse(width as f64, height as f64);

    let backup_pid = Pid::from_u32(process::id());
    cpu_evaluation::start_cpu_monitor(backup_pid, 120);

    // Loop to keep the program alive
    loop {
        thread::sleep(Duration::from_secs(1));
    }

}