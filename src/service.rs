#[cfg(not(target_os = "windows"))]
fn main() {
    panic!("This program is only intended to run on Windows.");
}

#[cfg(target_os = "windows")]
fn main() -> windows_service::Result<()> {
    service::run()
}

#[cfg(target_os = "windows")]
mod service {
    use std::{env, ffi::OsString, sync::mpsc, time::Duration};
    use std::{path::PathBuf, fs::File, fs::OpenOptions, io::Write, path::Path};
    use windows_sys::Win32::System::RemoteDesktop::{
        WTSEnumerateSessionsW, WTSQuerySessionInformationW, WTS_CURRENT_SERVER_HANDLE,
        WTS_INFO_CLASS, WTS_CONNECTSTATE_CLASS, WTSActive, WTSUserName, WTSFreeMemory, WTS_SESSION_INFOW
    };
    use windows_sys::core::PWSTR;
    use std::os::windows::ffi::OsStringExt;
    use windows_service::{
        define_windows_service,
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher, Result,
    };
    use std::{process, thread, ptr};
    use std::panic::catch_unwind;
    use std::process::Command;
    use egui::accesskit::DefaultActionVerb::Open;
    use sysinfo::Pid;
    use chrono::Local;
    use quick_xml::events::{BytesEnd, BytesStart, Event};
    use quick_xml::Reader;
    use quick_xml::Writer;
    use std::io::Cursor;

    const SERVICE_NAME: &str = "BackMeUp";
    const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

    pub fn run() -> Result<()> {
        // Register generated `ffi_service_main` with the system and start the service, blocking
        // this thread until the service is stopped.
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)
    }

    // Generate the windows service boilerplate.
    // The boilerplate contains the low-level service entry function (ffi_service_main) that parses
    // incoming service arguments into Vec<OsString> and passes them to user defined service
    // entry (my_service_main).
    define_windows_service!(ffi_service_main, my_service_main);

    // Service entry function which is called on background thread by the system with service
    // parameters. There is no stdout or stderr at this point so make sure to configure the log
    // output to file if needed.
    pub fn my_service_main(_arguments: Vec<OsString>) {
        if let Err(_e) = run_service() {
            // Handle the error.
            eprintln!("Error while running the service: {:?}", _e);
            return;
        }
    }

    pub fn run_service() -> Result<()> {
        // Create a channel to be able to poll a stop event from the service worker loop.
        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        // Define system service event handler that will be receiving service events.
        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                // Notifies a service to report its current status information to the service
                // control manager. Always return NoError even if not implemented.
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

                // Handle stop
                ServiceControl::Stop => {
                    shutdown_tx.send(()).unwrap();
                    ServiceControlHandlerResult::NoError
                }

                // treat the UserEvent as a stop request
                ServiceControl::UserEvent(code) => {
                    if code.to_raw() == 130 {
                        shutdown_tx.send(()).unwrap();
                    }
                    ServiceControlHandlerResult::NoError
                }

                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };

        // Register system service event handler.
        // The returned status handle should be used to report service status changes to the system.
        let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

        // Tell the system that service is running
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        log_message("Servizio avviato 1 \n");

        loop {
            // Poll shutdown event.
            match shutdown_rx.recv_timeout(Duration::from_secs(1)) {
                // Break the loop either upon stop or channel disconnect
                Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,

                // Continue work if no events were received within the timeout
                Err(mpsc::RecvTimeoutError::Timeout) => {log_message("Servizio avviato 2 \n"); ()},
            };

            log_message("Servizio avviato 3 \n");
            keep_backup_program_alive();
        }

        // Tell the system that service has stopped.
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        Ok(())
    }

    fn keep_backup_program_alive() {
        // Check if the process is running already
        if is_backup_program_running() {
            return;
        }

        // Start the program in the current user active session
        if let Err(e) = launch_backup_program() {
            eprintln!("Error while starting backup_program: {:?}", e);
        }
    }

    fn is_backup_program_running() -> bool {
        // Check if process is active already
        let output = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq backup_program.exe"])
            .output();

        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).contains("backup_program.exe"),
            Err(_) => false,
        }
    }

    fn launch_backup_program() -> std::io::Result<()> {

        log_message("Starting launch_backup_program");

        let exe_dir = match env::current_exe() {
            Ok(path) => {
                log_message(&format!("Current exe path: {:?}", path));
                path.parent()
                    .ok_or_else(|| {
                        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "Parent directory not found");
                        log_message(&format!("Error while trying to find directory parent: {:?}", err));
                        err
                    })?
                    .to_path_buf()
            }
            Err(e) => {
                log_message(&format!("Error while trying to find executable path: {:?}", e));
                return Err(e);
            }
        };

        let backup_program_path = exe_dir.join("backup_program.exe");
        log_message(&format!("backup_program path: {:?}", backup_program_path));

        if !backup_program_path.exists() {
            let err = std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("backup_program.exe not found in {:?}", backup_program_path)
            );
            log_message(&format!("Error: {}", err));
            return Err(err);
        }

        let username = get_current_user().unwrap();

        log_message(&format!("Username found: {}", username));

        let start_time = Local::now() + chrono::Duration::minutes(1);   // ToDo: see if this can be changed to instantly
        let start_time_str = start_time.format("%H:%M").to_string();

        log_message(&format!("Creating task at time: {}", start_time_str));

        let task_name = "BackupProgramLauncher";
        let xml_name = exe_dir.join("BackMeUp_task.xml");

        // Create the task specifying the user and the IT flag for interactivity
        let create_result = Command::new("schtasks")
            .args([
                "/Create",
                "/TN", task_name,
                "/TR", &backup_program_path.to_string_lossy(),
                "/SC", "ONCE",
                "/ST", &start_time_str,
                "/RU", &username,    // Specify the user
                "/IT",             // Allow interaction with desktop
                "/F",             // Force overwrite
                "/RL", "HIGHEST"  // Execute with privileges
            ])
            .output()?;

        // Modify task's XML to allow for execution on battery power

        // Execute the schtasks command and capture the XML output
        let output = Command::new("schtasks")
            .args(["/Query", "/TN", task_name, "/XML"])
            .output()?;

        if !output.status.success() {
            log_message("Error while executing schtasks\n");
        }

        let xml_content = String::from_utf8(output.stdout).unwrap();
        log_message(&xml_content);

        // Modify the XML in memory
        let mut modified_xml = xml_content.replace("<DisallowStartIfOnBatteries>true</DisallowStartIfOnBatteries>", "<DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>");
        modified_xml = modified_xml.replace("<StopIfGoingOnBatteries>true</StopIfGoingOnBatteries>", "<StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>");

        log_message("XML modified:\n");

        // Save the modified XML
        std::fs::write(xml_name.clone(), &modified_xml)?;

        // Use it to create the task
        let output = Command::new("schtasks")
            .args([
                "/Create",
                "/TN",
                task_name,
                "/XML",
                xml_name.display().to_string().as_str(),
                "/F",                   // Force overwrite if task already exists
            ])
            .output()?;


        log_message(&format!(
            "Task creation result:\nStatus: {}\nStdout: {}\nStderr: {}",
            create_result.status,
            String::from_utf8_lossy(&create_result.stdout),
            String::from_utf8_lossy(&create_result.stderr)
        ));

        if !create_result.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create scheduled task: {}",
                        String::from_utf8_lossy(&create_result.stderr))
            ));
        }

        // Start the task
        log_message("Starting task...");
        let run_result = Command::new("schtasks")
            .args([
                "/Run",
                "/TN", "BackupProgramLauncher"
            ])
            .output()?;

        log_message(&format!(
            "Task start result:\nStatus: {}\nStdout: {}\nStderr: {}",
            run_result.status,
            String::from_utf8_lossy(&run_result.stdout),
            String::from_utf8_lossy(&run_result.stderr)
        ));

        if !run_result.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to run scheduled task: {}",
                        String::from_utf8_lossy(&run_result.stderr))
            ));
        }

        // Remove task after start
        // log_message("Removing task...");
        // let delete_result = Command::new("schtasks")
        //     .args([
        //         "/Delete",
        //         "/TN", "BackupProgramLauncher",
        //         "/F"
        //     ])
        //     .output()?;
        //
        // log_message(&format!(
        //     "Remove task result:\nStatus: {}\nStdout: {}\nStderr: {}",
        //     delete_result.status,
        //     String::from_utf8_lossy(&delete_result.stdout),
        //     String::from_utf8_lossy(&delete_result.stderr)
        // ));

        log_message("launch_backup_program completed");
        Ok(())
    }

    fn log_message(message: &str) {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("C:\\Temp\\backup_service_detailed.log")
        {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            if let Err(e) = writeln!(file, "[{}] {}", timestamp, message) {
                eprintln!("Failed to write to log file: {}", e);
            }
        }
    }

    fn get_current_user() -> Option<String> {
        unsafe {
            let mut session_info_ptr: *mut WTS_SESSION_INFOW = ptr::null_mut();
            let mut session_count: u32 = 0;

            // Enumerate all sessions
            if WTSEnumerateSessionsW(
                WTS_CURRENT_SERVER_HANDLE,
                0,
                1,
                &mut session_info_ptr,
                &mut session_count,
            ) == 0
            {
                return None; // Failed to enumerate sessions
            }

            // Interpret the session info as a slice of WTS_SESSION_INFO structures
            let session_info_size = std::mem::size_of::<WTS_SESSION_INFO>();
            let session_info_slice = std::slice::from_raw_parts(
                session_info_ptr as *const WTS_SESSION_INFO,
                session_count as usize,
            );

            for session_info in session_info_slice {
                if session_info.State == WTSActive {
                    let mut user_ptr: PWSTR = ptr::null_mut();
                    let mut bytes_returned: u32 = 0;

                    // Query the username for the active session
                    if WTSQuerySessionInformationW(
                        WTS_CURRENT_SERVER_HANDLE,
                        session_info.SessionId,
                        WTSUserName,
                        &mut user_ptr,
                        &mut bytes_returned,
                    ) != 0
                    {
                        let username = to_string(user_ptr);
                        WTSFreeMemory(user_ptr as *mut _); // Free memory allocated by WTS
                        WTSFreeMemory(session_info_ptr as *mut _); // Free memory allocated by WTS
                        return Some(username);
                    }
                }
            }

            WTSFreeMemory(session_info_ptr as *mut _); // Free memory allocated by WTS
            None
        }
    }

    #[repr(C)]
    struct WTS_SESSION_INFO {
        SessionId: u32,
        pWinStationName: PWSTR,
        State: WTS_CONNECTSTATE_CLASS,
    }

    unsafe fn to_string(pwstr: PWSTR) -> String {
        if pwstr.is_null() {
            return String::new();
        }
        let len = (0..).take_while(|&i| *pwstr.offset(i) != 0).count();
        let slice = std::slice::from_raw_parts(pwstr, len);
        OsString::from_wide(slice).to_string_lossy().into_owned()
    }
}

