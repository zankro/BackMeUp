use std::{io, process::Command};
use service_manager::*;
use std::ffi::OsString;
use std::path::{PathBuf};
use std::env;
use std::error::Error;

mod cpu_evaluation;
mod uninstall_service;
mod service;

#[cfg(target_os = "windows")]
fn main() -> windows_service::Result<()> {
    use std::ffi::OsString;
    use windows_service::{
        service::{ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType},
        service_manager::{ServiceManager, ServiceManagerAccess},
    };

    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    // Installs the service defined in `service.rs`.
    let service_binary_path = ::std::env::current_exe().unwrap().with_file_name("service.exe");

    let service_info = ServiceInfo {
        name: OsString::from("BackMeUp"),
        display_name: OsString::from("BackMeUp"),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: vec![],
        dependencies: vec![],
        account_name: None, // run as System
        account_password: None,
    };
    let service = service_manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
    service.set_description("Group16 BackMeUp Application Service")?;
    Ok(())
}


#[cfg(not(target_os = "windows"))]
fn is_service_installed(service_name: &str) -> Result<bool, Box<dyn Error>> {
    #[cfg(target_os = "macos")]
    {
        // Check if service is installed on MacOs platform
        let output = Command::new("launchctl")
            .arg("list")
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Check if the service name is present in the output
        Ok(stdout.contains(service_name))
    }

    #[cfg(target_os = "linux")]
    {
        // Check if the service is installed on Linux platform
        let output = Command::new("systemctl")
            .arg("list-units")
            .arg("--user")
            .arg("--type=service")
            .arg("--all")
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check if the service name is present in the output
        Ok(stdout.contains(service_name))
    }
}

#[cfg(not(target_os = "windows"))]
fn main() -> io::Result<()> {

    // Create a label for our service
    let label: ServiceLabel = "backmeup".parse().unwrap();
    let mut env_vars: Option<Vec<(String, String)>> = None;

    #[cfg(target_os = "linux")]
    {
        // Set the DISPLAY environment variable for the service to allow GUI applications
        env_vars = Some(vec![("DISPLAY".to_string(), ":0".to_string())]);
    }


    // Get generic service by detecting what is available on the platform
    let mut manager = <dyn ServiceManager>::native()
        .expect("Failed to detect management platform");

    // Update our manager to work with user-level services
    manager.set_level(ServiceLevel::User)
        .expect("Service manager does not support user-level services");


    match is_service_installed("backmeup") {
        Ok(installed) => {
            if installed {
                println!("Service already installed and running");
            } else {
                println!("Service not installed, starting installation..");
                // Install our service using the underlying service management platform
                let result = manager.install(ServiceInstallCtx {
                    label: label.clone(),
                    program: PathBuf::from(env::current_exe().unwrap().parent().unwrap().join("backup_program")),
                    args: vec![],
                    contents: None,
                    username: None,
                    working_directory: None,
                    environment: env_vars,
                    autostart: true,
                });

                match result {
                    Ok(_) => {
                        #[cfg(target_os = "linux")]
                        {
                            // Start our service using the underlying service management platform
                            manager.start(ServiceStartCtx {
                                label: label.clone()
                            }).expect("Failed to start");
                        }

                        println!("Service installed successfully");
                    }
                    Err(e) => {
                        eprintln!("Error during service installation: {}", e);
                    }
                }

            }

            println!("Service started successfully");
        }
        Err(e) => {
            eprintln!("Error during service check: {}", e);
        }
    }

    Ok(())
}