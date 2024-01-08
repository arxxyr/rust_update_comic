use clap::{Arg, Command};
use filetime::FileTime;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info};
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::{Command as SystemCommand, Stdio};
use std::time::SystemTime;
use tokio::time::{sleep, Duration};
use walkdir::WalkDir;

mod shutdown;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    source: String,
    target: String,
    shutdown: bool,
    delay: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
    // let pb = create_progress_bar(total_folders);

    for (index, entry) in (0_u64..).zip(WalkDir::new(source_path).min_depth(1).max_depth(1)) {
        let entry = entry?;
        print!(
            "Start compressing {}/{} : {}",
            index,
            total_folders,
            entry.path().display()
        );
        if entry.file_type().is_dir() {
            let folder_path = entry.path();
            let folder_name = entry
                .file_name()
                .to_str()
                .ok_or("Failed to convert path to string")?;
            // let target_file = format!("{}/{}.cbz", target_path.display(), folder_name);
            // let target_file = target_path.join(format!("{}.cbz", folder_name));
            let target_file_pathbuf = target_path.join(folder_name).with_extension("cbz");

            // 将 PathBuf 转换为 &str
            let target_file_str = target_file_pathbuf
                .to_str()
                .ok_or("Failed to convert path to string")?;

            if should_compress(folder_path, target_file_str)? {
                compress_folder(folder_path, target_file_str)?;
                println!("    ...done");
            } else {
                println!("    ...skipped");
            }
            // 暂停200毫秒
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        // pb.inc(1);
    }

    // pb.finish_with_message("All folders compressed");

    if shutdown {
        shutdown::shutdown(config.delay).await?;
    } else {
        Notification::new()
            .summary("bika 压缩完毕")
            .body("已经压缩完毕")
            .icon("firefox")
            .show()?;

        // 在返回前暂停一段时间
        println!("Pausing before exiting...");
        sleep(Duration::from_secs(5)).await; // 异步暂停3秒
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

#[allow(dead_code)]
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

fn compress_folder(source_folder: &Path, target_file: &str) -> Result<(), Box<dyn Error>> {
    // 获取系统临时文件夹路径
    let temp_dir = std::env::temp_dir();
    let temp_path: std::path::PathBuf = tempfile::Builder::new()
        .suffix(".cbz")
        .tempfile_in(temp_dir)?
        .into_temp_path()
        .to_path_buf();

    // 将 PathBuf 转换为 &str
    let temp_file_str = temp_path
        .to_str()
        .ok_or("Failed to convert path to string")?;
    let original_folder = source_folder.join("original");

    let status = SystemCommand::new("7z")
        .current_dir(original_folder)
        .args(["a", "-tzip", &temp_file_str, "."])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if status.success() {
        // 尝试移动临时文件到目标位置，如果失败则使用复制和删除
        if fs::rename(&temp_path, target_file).is_err() {
            println!("\nMoving file failed, using copy and delete instead");
            let result = fs::copy(&temp_path, target_file);
            match result {
                Ok(_) => {
                    println!("Copy successful");
                    let remove_result = fs::remove_file(&temp_path);
                    match remove_result {
                        Ok(_) => println!("Remove original file successful"),
                        Err(e) => println!("Remove original file failed with error: {}", e),
                    }
                }
                Err(e) => println!("Copy failed with error: {}", e),
            }
        }

        // 更新压缩文件的修改时间
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
