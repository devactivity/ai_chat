use log::LevelFilter;
use simplelog::{CombinedLogger, Config, WriteLogger};
use std::{fs::File, path::PathBuf};

pub fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = dirs::home_dir()
        .ok_or("home directory not found")?
        .join(".config")
        .join("chat_tui");

    std::fs::create_dir_all(&config_dir)?;

    let log_file_path = config_dir.join("logs").join("errors.log");
    std::fs::create_dir_all(log_file_path.parent().unwrap())?;

    let file = File::create(log_file_path)?;

    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        file,
    )])?;

    Ok(())
}

pub fn get_log_file_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".config")
        .join("chat_tui")
        .join("logs")
        .join("errors.log")
}
