use eframe::egui::{self, CentralPanel, ComboBox};
use std::{env, fs};
use std::path::{PathBuf};
use std::io::Write;
use std::sync::{Arc, Mutex};
use eframe::Frame;
use egui::{Align, Color32, Context, Layout, RichText, ViewportCommand, Window};
use rfd::FileDialog;
#[cfg(target_os = "linux")]
use std::process::Command;

// Structure for the Config file
#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Config {
    source_path: String,
    destination_path: String,
    backup_type: String,
    extensions_to_backup: Vec<String>,
}

// Application GUI
#[derive(Default)]
struct ConfigWindow {
    source_path: String,
    destination_path: String,
    backup_type: String,
    extensions_to_backup: String,
}

impl ConfigWindow {
    fn new(config_file_path: PathBuf) -> Self {
        let mut window = Self::default();
        if config_file_path.join("config.toml").exists() {
            window.read_config(config_file_path);
        }
        window
    }

    // Method for saving the configuration file
    fn save_config(&self, config_file_path: PathBuf) {
        println!("{:?}", self.backup_type);
        let config = Config {
            source_path: self.source_path.clone(),
            destination_path: self.destination_path.clone(),
            backup_type: self.backup_type.clone(),
            extensions_to_backup: self.extensions_to_backup
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        };
        let toml_str = toml::to_string(&config).unwrap();
        let mut file = fs::File::create(config_file_path.join("config.toml")).unwrap();
        file.write_all(toml_str.as_bytes()).unwrap();
    }

    // Read config.toml if exists and fills fields with the values
    fn read_config(&mut self, config_file_path: PathBuf) {
        let config = fs::read_to_string(config_file_path.join("config.toml")).unwrap();
        let config: Config = toml::from_str(&config).unwrap();
        self.source_path = config.source_path;
        self.destination_path = config.destination_path;
        self.backup_type = config.backup_type;
        self.extensions_to_backup = config.extensions_to_backup.join(", ");
    }

    // Method for selecting a directory using a file dialog
    fn select_directory() -> Option<String> {
        #[cfg(not(target_os = "linux"))]
        {
            FileDialog::new()
                .pick_folder()  // Apre il dialogo per selezionare una cartella
                .map(|path| path.display().to_string())  // Converte il percorso selezionato in stringa
        }
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("zenity")
                .arg("--file-selection")
                .arg("--directory")
                .output()
                .ok()?;

            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        }
    }
}

impl eframe::App for ConfigWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        let exe_path: PathBuf = PathBuf::from(env::current_exe().unwrap().parent().unwrap());
        let config_file_path = exe_path.parent().unwrap().join("Resources/");

        // Variable to track errors
        let mut error_message = String::new();

        // Function to check the validity of the fields
        let is_valid = !self.source_path.trim().is_empty()
            && !self.destination_path.trim().is_empty()
            && !self.backup_type.trim().is_empty()
            && (self.backup_type != "selective" || !self.extensions_to_backup.trim().is_empty())
            && self.source_path != self.destination_path; // Controllo sui percorsi

        CentralPanel::default().show(ctx, |ui| {
            // Spacing and global style
            let spacing = ui.spacing_mut();
            spacing.item_spacing = egui::Vec2::new(5.0, 7.0); // Horizontal and vertical spacing
            spacing.text_edit_width = 300.0; // Textfield width

            ui.heading("Backup Configuration");
            ui.add_space(10.0);

            ui.label("Source Path:");
            ui.horizontal(|ui| {
                // Source path text field
                ui.text_edit_singleline(&mut self.source_path);

                // File dialog button
                if ui.button("...").clicked() {
                    if let Some(path) = ConfigWindow::select_directory() {
                        self.source_path = path;
                    }
                }
            });
            ui.add_space(5.0);

            ui.label("Destination Path:");
            ui.horizontal(|ui| {
                // Destination path text field
                ui.text_edit_singleline(&mut self.destination_path);

                // File dialog button
                if ui.button("...").clicked() {
                    if let Some(path) = ConfigWindow::select_directory() {
                        self.destination_path = path;
                    }
                }
            });
            ui.add_space(5.0);

            // Backup type selector
            ui.label("Backup Type:");
            ComboBox::from_label("")
                .selected_text(&self.backup_type)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.backup_type, "full-disk".to_string(), "Full Disk");
                    ui.selectable_value(&mut self.backup_type, "directory".to_string(), "Directory");
                    ui.selectable_value(&mut self.backup_type, "selective".to_string(), "Selective");

                });

            // Only shows the "File Extensions" field if the backup type is "selective"
            if self.backup_type == "selective" {
                ui.label("File Extensions (comma separated):");
                ui.text_edit_singleline(&mut self.extensions_to_backup);
            }

            // Check if the fields are valid and set the error message if needed
            if self.source_path.trim().is_empty() {
                error_message.push_str("Source path is required.\n");
            }
            if self.destination_path.trim().is_empty() {
                error_message.push_str("Destination path is required.\n");
            }
            if self.backup_type == "selective" && self.extensions_to_backup.trim().is_empty() {
                error_message.push_str("Extensions are required for selective backup.\n");
            }
            if self.source_path == self.destination_path {
                error_message.push_str("Source and destination paths cannot be the same.\n");
            }


            ui.add_space(10.0);

            // Shows the save button with an error message if needed
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                // Colore del pulsante
                let save_button_color = if is_valid {
                    Color32::from_rgb(51, 204, 51) // Verde se valido
                } else {
                    Color32::from_rgb(200, 100, 100) // Rosso se non valido
                };

                let button_text = RichText::new("Save and Exit")
                    .strong()
                    .size(15.0);

                let save_button = ui.add_sized(
                    [120.0, 30.0],
                    egui::Button::new(button_text).fill(save_button_color),
                );

                // Save and close only if the button is clicked and valid
                if save_button.clicked() {
                    self.save_config(config_file_path);
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }

                // Shows the error message if there are invalid fields
                if !is_valid {
                    ui.add_space(10.0);
                    ui.label(RichText::new(error_message).color(Color32::from_rgb(255, 0, 0)));
                }
            });
        });
    }
}


// Funzione per avviare la GUI solo se `config.toml` non esiste
pub fn show_gui_if_needed() -> Result<(), eframe::Error> {
    println!("Verifica se il file di configurazione esiste...");

    let exe_path: PathBuf = PathBuf::from(env::current_exe().unwrap().parent().unwrap());
    let mut config_file_path;

    config_file_path = exe_path.parent().unwrap().join("Resources/");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([350f32, 325f32]),
        ..Default::default()
    };
    eframe::run_native(
        "BackMeUp",
        options,
        Box::new(|_cc| Ok(Box::new(ConfigWindow::new(config_file_path)))),
    )
}

#[derive(Default)]
struct BackupWindow{
    should_close: Arc<Mutex<bool>>
}



impl eframe::App for BackupWindow {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        // Aggiungiamo un pop-up al centro dello schermo
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // Titolo del pop-up con testo più grande
                ui.heading(
                    RichText::new("Do you want to proceed with backup?")
                        .strong(),  // Grassetto per maggiore enfasi
                );
                ui.add_space(15.0); // Spaziatura più grande sotto il titolo

                // Legenda con testo più grande e distanziato
                ui.label(RichText::new("1. Slide from bottom-left corner to bottom-right corner to confirm.").size(13.0));
                ui.add_space(10.0); // Spazio tra le righe
                ui.label(RichText::new("2. Slide from bottom-left corner to top-left corner to abort.").size(13.0));
                ui.add_space(10.0); // Spazio tra le righe
                ui.label(RichText::new("3. Slide from bottom-left corner to top-right corner to change config.").size(13.0));
            });
        });
    }
}

// Funzione per mostrare la finestra di backup come pop-up
pub fn show_backup_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_always_on_top()
            .with_inner_size([450f32, 150f32])
            .with_decorations(true)
            .with_drag_and_drop(true),
        centered: true,
        ..Default::default()
    };

    // Avvia l'interfaccia grafica con la finestra di backup
    eframe::run_native(
        "BackMeUp",
        options,
        Box::new(|_cc| Ok(Box::new(BackupWindow {should_close: Arc::new(Mutex::new(false))}))),
    )
}

pub fn close_backup_window(should_close: Arc<Mutex<bool> >) {
    let mut should_close = should_close.lock().unwrap();
    *should_close = true;
}

pub fn is_window_open(should_close: Arc<Mutex<bool>>) -> bool {
    let should_close = should_close.lock().unwrap();
    !*should_close
}