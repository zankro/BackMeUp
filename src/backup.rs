use std::{fs, io};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use fs_extra::dir::CopyOptions;
use fs_extra::error::Error as FsExtraError;
use walkdir::{WalkDir, DirEntry};

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub source_path: String,
    pub destination_path: String,
    pub backup_type: String,
    pub extensions_to_backup: Vec<String>,
}

pub fn read_config(config_path: &str) -> Config {
    let mut file = fs::File::open(config_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    toml::from_str(&contents).unwrap()
}

#[derive(Debug)]
pub(crate) enum BackupError {
    SourceNotFound,
    InvalidBackupType,
    IoError(io::Error),
    FsExtraError(FsExtraError),
}

impl From<io::Error> for BackupError {
    fn from(error: io::Error) -> Self {
        BackupError::IoError(error)
    }
}

impl From<FsExtraError> for BackupError {
    fn from(error: FsExtraError) -> Self {
        BackupError::FsExtraError(error)
    }
}

pub(crate) fn backup_files(config: &Config) -> Result<(), BackupError> {
    let source_path = Path::new(&config.source_path);
    let mut destination_path= PathBuf::from(&config.destination_path);

    // Check if source_path has a file_name
    if let Some(source_folder_name) = source_path.file_name() {
        destination_path.push(source_folder_name);
    }

    println!("Backup started from: {:?}", source_path);
    println!("Backup towards folder: {:?}", destination_path);

    // Total size of files copied
    let mut total_size: u64 = 0;

    let start_time = Instant::now();

    // Check if source path exists
    if !source_path.exists() {
        return Err(BackupError::SourceNotFound);
    }

    // Create destination directory if it doesn't exist
    if !destination_path.exists() {
        fs::create_dir_all(destination_path.as_path())?;
        println!("Created destination directory: {:?}", destination_path.as_path());
    }

    let mut dir_options = CopyOptions::new();
    dir_options.overwrite = true;

    // Perform backup based on the backup type
    match config.backup_type.as_str() {
        "full-disk" | "directory" => {
            // Perform backup and calculate total size
            total_size = backup_with_walkdir(source_path, destination_path.as_path(), None)?;
        },
        "selective" => {
            // Copy files with specific extensions and calculate total size
            total_size = backup_with_walkdir(source_path, destination_path.as_path(), Some(&config.extensions_to_backup))?;
        },
        _ => return Err(BackupError::InvalidBackupType),
    }

    let backup_time = start_time.elapsed();
    backup_monitor(destination_path.as_path(), total_size, backup_time);
    Ok(())
}

fn backup_monitor(destination_path: &Path, total_size: u64, backup_time: Duration) {
    let log_path = destination_path.join("backup_log.txt");
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .unwrap();

    writeln!(file, "Total size of saved files: {} bytes", total_size).unwrap();
    writeln!(file, "Backup completed in: {:.2} seconds", backup_time.as_secs_f64()).unwrap();
}

fn backup_with_walkdir<P: AsRef<Path>, Q: AsRef<Path>>(
    source: P,
    destination: Q,
    extensions_to_backup: Option<&Vec<String>>,
) -> io::Result<u64> {
    let source = source.as_ref();
    let destination = destination.as_ref();

    if !source.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Percorso non trovato: {}", source.display()),
        ));
    }

    if !source.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Il percorso non è una directory: {}", source.display()),
        ));
    }

    // Creates the destination directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(destination) {
        eprintln!(
            "Errore durante la creazione della directory {}: {}. Ignorata.",
            destination.display(),
            e
        );
    }

    let mut total_size = 0;

    for entry in WalkDir::new(source)
        .into_iter()
        .filter_entry(|e| !is_hidden_or_problematic(e))            // Ignore hidden files and problematic directories
    {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Errore durante l'accesso a un elemento: {}. Ignorato.", e);
                continue;
            }
        };

        let entry_path = entry.path();
        println!("Elaborando: {:?}", entry_path);
        let relative_path = match entry_path.strip_prefix(source) {
            Ok(rel) => rel,
            Err(_) => {
                eprintln!(
                    "Errore nel calcolo del percorso relativo per: {}. Ignorato.",
                    entry_path.display()
                );
                continue;
            }
        };

        let dest_path = destination.join(relative_path);

        if entry.file_type().is_dir() {
            // Crea la directory di destinazione
            if let Err(e) = fs::create_dir_all(&dest_path) {
                eprintln!(
                    "Errore durante la creazione della directory {}: {}. Ignorata.",
                    dest_path.display(),
                    e
                );
            }
        } else if entry.file_type().is_file() {
            // Se ci sono estensioni specificate, copia solo i file con le estensioni corrispondenti
            if let Some(extensions) = extensions_to_backup {
                if let Some(extension) = entry_path.extension() {
                    if !extensions.contains(&extension.to_string_lossy().to_string()) {
                        continue; // Skip files that don't match with the extensions selected
                    }
                } else {
                    continue; // Skip files without an extension
                }
            }

            // Copia il file
            match fs::copy(&entry_path, &dest_path) {
                Ok(size) => total_size += size,
                Err(e) => eprintln!(
                    "Errore durante la copia del file {}: {}. Ignorato.",
                    entry_path.display(),
                    e
                ),
            }

            println!("Copiato: {:?}", dest_path);
        }
    }

    Ok(total_size)
}

fn is_hidden_or_problematic(entry: &DirEntry) -> bool {
    let file_name = entry.file_name().to_string_lossy();
    let path = entry.path();

    // Check if the file or directory is hidden
    if file_name.starts_with('.') {
        return true;
    }

    // Exclude problematic directories on Linux
    #[cfg(target_os = "linux")]
    {
        let problematic_dirs = ["/sys", "/proc", "/dev", "/run", "/tmp"];
        if path.is_dir() {
            if problematic_dirs.iter().any(|&d| path.starts_with(d)) {
                return true;
            }
        }
    }

    false
}