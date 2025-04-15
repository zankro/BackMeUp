use sysinfo::{System, Pid, ProcessesToUpdate};
use std::{env, fs::OpenOptions, thread};
use chrono::Local;
use std::io::Write;
use std::time::Duration;


/// Create or open a log file to store CPU usage data.
fn create_log_file() -> std::fs::File {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(env::current_exe().unwrap().parent().unwrap().parent().unwrap().join("Resources/cpu_log.txt"))
        .expect("Unable to create or open log file")
}

/// Log CPU usage for a given process using its PID.
fn log_cpu_usage(average_cpu_usage: f32, log_file: &mut std::fs::File) {
    let current_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_entry = format!(
        "{} - CPU Usage: {:.6}%",
        current_time,
        average_cpu_usage,
    );
    writeln!(log_file, "{}", log_entry).expect("Unable to write to log file");
}

/// Start monitoring CPU usage for a specific process.
pub fn start_cpu_monitor(pid: Pid, interval_secs: u64) {
    let mut log_file = create_log_file(); // Open log file
    let mut sys = System::new_all();  // Create sysinfo system object
    let mut average_cpu_usage = 0.0;
    let mut count = 0;

    // Spawn a thread for continuous monitoring
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));

            count += 1;

            sys.refresh_processes(ProcessesToUpdate::Some(&[pid]));

            if let Some(process) = sys.process(pid){
                average_cpu_usage += process.cpu_usage() / sys.cpus().len() as f32;
            }

            if count == interval_secs{
                log_cpu_usage(average_cpu_usage / count as f32, &mut log_file);
                count = 0;
            }
        }
    });
}