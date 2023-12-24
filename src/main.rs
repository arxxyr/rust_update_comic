use clap::{Arg, Command};
use filetime::FileTime;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::{Command as SystemCommand, Stdio};
use std::time::SystemTime;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    source: String,
    target: String,
    shutdown: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let matches = Command::new("Folder Compression Tool")
        .version("1.0")
        .author("ffqi")
        .about("Compresses folders")
        .arg(
            Arg::new("config")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file"), // .value(true)
        )
        .get_matches();

    let config_path = matches
        .get_one::<String>("config")
        .map(|s| s.as_str())
        .unwrap_or("config.yaml");

    let config = read_config(config_path)?;

    let source_path = Path::new(&config.source);
    let target_path = Path::new(&config.target);
    let shutdown = config.shutdown;

    let total_folders = count_folders(source_path)?;
    let pb = create_progress_bar(total_folders);

    for entry in WalkDir::new(source_path).min_depth(1).max_depth(1) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            let folder_path = entry.path();
            let folder_name = entry.file_name().to_str().unwrap();
            let target_file = format!("{}/{}.cbz", target_path.display(), folder_name);

            if should_compress(folder_path, &target_file)? {
                let _ = compress_folder(folder_path, &target_file)?;
            }
        }
        pb.inc(1);
    }

    pb.finish_with_message("All folders compressed");

    if shutdown {
        shutdown_system();
    }

    Ok(())
}

fn read_config(filename: &str) -> Result<Config, Box<dyn Error>> {
    let contents =
        fs::read_to_string(filename).map_err(|e| format!("Error reading {}: {}", filename, e))?;
    serde_yaml::from_str(&contents).map_err(|e| format!("Error parsing {}: {}", filename, e).into())
}

fn count_folders(path: &Path) -> Result<u64, Box<dyn Error>> {
    Ok(WalkDir::new(path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .count() as u64)
}

fn create_progress_bar(length: u64) -> ProgressBar {
    let pb = ProgressBar::new(length);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{wide_bar} {pos}/{len}")
            .expect("Invalid progress bar template") // 使用 expect 来处理 Result
            .progress_chars("=> "),
    );
    pb
}

fn should_compress(source_folder: &Path, target_file: &str) -> Result<bool, Box<dyn Error>> {
    if !Path::new(target_file).exists() {
        return Ok(true);
    }

    let source_modified = latest_modification_time(source_folder)?;
    let target_modified = fs::metadata(target_file).and_then(|metadata| metadata.modified())?;

    Ok(source_modified != target_modified)
}

fn latest_modification_time(path: &Path) -> Result<SystemTime, Box<dyn Error>> {
    let mut latest: Option<SystemTime> = None;
    for entry in WalkDir::new(path)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                if latest.is_none() || Some(modified) > latest {
                    latest = Some(modified);
                }
            }
        }
    }
    latest.ok_or_else(|| "No files found in the directory".into())
}

fn compress_folder(
    source_folder: &Path,
    target_file: &str,
    // pb: &ProgressBar,
) -> Result<(), Box<dyn Error>> {
    let msg = format!("Compressing: {}", source_folder.to_string_lossy());
    println!("{}", msg); // Print the message above the progress bar
    let original_folder = source_folder.join("original");
    let status = SystemCommand::new("7z")
        .current_dir(original_folder)
        .args(&["a", target_file, "."])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if status.success() {
        if let Ok(latest_modification) = latest_modification_time(source_folder) {
            let file_time = FileTime::from_system_time(latest_modification);
            filetime::set_file_mtime(target_file, file_time)
                .map_err(|e| format!("Failed to update modification date: {}", e))?;
            info!("Updated modification date for {}", target_file);
        }
    } else {
        error!(
            "Error compressing folder: {}",
            source_folder.to_string_lossy()
        );
        return Err("Compression failed".into());
    }

    Ok(())
}

fn shutdown_system() {
    info!("Shutting down the system...");
    #[cfg(target_os = "windows")]
    SystemCommand::new("shutdown")
        .args(&["/s", "/f", "/t", "300"])
        .output()
        .expect("Failed to execute shutdown command");
    #[cfg(not(target_os = "windows"))]
    SystemCommand::new("shutdown")
        .args(&["-h", "now"])
        .output()
        .expect("Failed to execute shutdown command");
}
