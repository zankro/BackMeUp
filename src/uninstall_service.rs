use std::io;
use std::process::Command;

#[cfg(windows)]
fn main() -> windows_service::Result<()> {
    use std::{
        thread::sleep,
        time::{Duration, Instant},
        ffi::OsStr
    };

    use windows_service::{
        service::{ServiceAccess, ServiceState},
        service_manager::{ServiceManager, ServiceManagerAccess},
    };
    use windows_sys::Win32::Foundation::ERROR_SERVICE_DOES_NOT_EXIST;
    use sysinfo::System;

    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service("BackMeUp", service_access)?;

    // The service will be marked for deletion as long as this function call succeeds.
    // However, it will not be deleted from the database until it is stopped and all open handles to it are closed.
    service.delete()?;
    // Our handle to it is not closed yet. So we can still query it.
    if service.query_status()?.current_state != ServiceState::Stopped {
        // If the service cannot be stopped, it will be deleted when the system restarts.
        service.stop()?;
    }
    // Explicitly close our open handle to the service. This is automatically called when `service` goes out of scope.
    drop(service);

    // Win32 API does not give us a way to wait for service deletion.
    // To check if the service is deleted from the database, we have to poll it ourselves.
    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    while start.elapsed() < timeout {
        if let Err(windows_service::Error::Winapi(e)) =
            service_manager.open_service("BackMeUp", ServiceAccess::QUERY_STATUS)
        {
            if e.raw_os_error() == Some(ERROR_SERVICE_DOES_NOT_EXIST as i32) {
                println!("BackMeUp is deleted.");
                break;
            }
        }
        sleep(Duration::from_secs(1));
    }

    // Delete the scheduled task
    // First, end the task if it is running. This will also kill the backup_program process.
    let _ = Command::new("schtasks")
        .args(["/END", "/TN", "BackupProgramLauncher"])
        .output();

    // Then delete the task
    let task_result = Command::new("schtasks")
        .args(["/Delete", "/TN", "BackupProgramLauncher", "/F"])
        .output();

    match task_result {
        Ok(output) => {
            if output.status.success() {
                println!("Task deleted successfully.");
            } else {
                eprintln!("Error during task deletion: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            eprintln!("Error executing schtask command: {}", e);
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn main() -> io::Result<()> {
    println!("Disabling service on MacOs...");

    let _ = Command::new("launchctl")
        .args(["remove", "backmeup"])
        .output();

    let plist_path = format!("{}/Library/LaunchAgents/backmeup.plist", std::env::var("HOME").unwrap());
    if let Err(e) = std::fs::remove_file(&plist_path) {
        eprintln!("Error during plist file elimination: {}", e);
    } else {
        println!("Plist file deleted successfully.");
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn main() -> io::Result<()> {
    println!("Disabling service on Linux...");

    let _ = Command::new("systemctl")
        .args(["--user", "stop", "backmeup.service"])
        .output();

    let _ = Command::new("systemctl")
        .args(["--user", "disable", "backmeup.service"])
        .output();

    let service_path = format!("{}/.config/systemd/user/backmeup.service", std::env::var("HOME").unwrap());
    if let Err(e) = std::fs::remove_file(&service_path) {
        eprintln!("Error during service file elimination: {}", e);
    } else {
        println!("Service file deleted successfully.");
    }

    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output();

    println!("Service disabled and deleted successfully.");

    Ok(())
}
