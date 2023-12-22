use clap::{Arg, Command};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;
use std::process::{Command as SystemCommand, Stdio};
use std::time::SystemTime;
use walkdir::WalkDir;

fn main() {
    let matches = Command::new("Folder Compression Tool")
        .version("1.0")
        .author("Your Name")
        .about("Compresses folders")
        .arg(
            Arg::new("source")
                .long("source")
                .value_name("SOURCE")
                .help("Sets the source directory"), // .takes_value(true),
        )
        .arg(
            Arg::new("target")
                .long("target")
                .value_name("TARGET")
                .help("Sets the target directory"), // .takes_value(true),
        )
        .arg(
            Arg::new("shutdown")
                .long("shutdown")
                .value_name("SHUTDOWN")
                .help("Set to true to shutdown the computer after completing tasks"), // .takes_value(true),
        )
        .get_matches();

    let source_path = matches
        .get_one::<String>("source")
        .expect("A source directory is required")
        .as_str();
    let target_path = matches
        .get_one::<String>("target")
        .expect("A target directory is required")
        .as_str();
    let shutdown = matches
        .get_one::<String>("shutdown")
        .map_or(false, |s| s == "true");

    let total_folders = WalkDir::new(source_path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .count();

    let pb = ProgressBar::new(total_folders as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{wide_bar} {pos}/{len}")
            .unwrap()
            .progress_chars("=> "),
    );

    for entry in WalkDir::new(source_path).min_depth(1).max_depth(1) {
        let entry = entry.unwrap();
        if entry.file_type().is_dir() {
            let folder_path = entry.path();
            let folder_name = entry.file_name().to_str().unwrap();
            let target_file = format!("{}/{}.cbz", target_path, folder_name);

            if should_compress(folder_path, &target_file) {
                compress_folder(folder_path, &target_file, &pb);
                pb.inc(1);
            }
        }
    }

    pb.finish_with_message("All folders compressed");

    if shutdown {
        // 关机逻辑
        #[cfg(target_os = "windows")]
        SystemCommand::new("shutdown")
            .args(&["/s", "/f", "/t", "300"])
            .output()
            .expect("Failed to shutdown");
        #[cfg(not(target_os = "windows"))]
        SystemCommand::new("shutdown")
            .args(&["-h", "now"])
            .output()
            .expect("Failed to shutdown");
    }
}

fn should_compress(source_folder: &Path, target_file: &str) -> bool {
    if !Path::new(target_file).exists() {
        return true;
    }

    let source_modified = latest_modification_time(source_folder).unwrap_or(SystemTime::UNIX_EPOCH);
    let target_modified = fs::metadata(target_file)
        .and_then(|metadata| metadata.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    source_modified > target_modified
}

fn latest_modification_time(path: &Path) -> Option<SystemTime> {
    let mut latest: Option<SystemTime> = None;
    if path.is_dir() {
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
    }
    latest
}

fn compress_folder(source_folder: &Path, target_file: &str, pb: &ProgressBar) {
    // Create a temporary directory to store the compressed files
    let temp_dir = source_folder.join("temp");
    fs::create_dir_all(&temp_dir).expect("Failed to create temporary directory");

    let msg = format!("Compressing: {}", source_folder.to_string_lossy());
    println!("{}", msg); // Print the message above the progress bar
                         // Compress the folder contents into a temporary directory
    let status = SystemCommand::new("7z")
        .current_dir(source_folder)
        .args(&["a", target_file, "."])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("Failed to execute 7z command");
    if !status.success() {
        eprintln!(
            "Error compressing folder: {}",
            source_folder.to_string_lossy()
        );
    }

    // Update the progress bar
    pb.inc(1);
}

// Implement the should_compress and compress_folder functions as before
