use std::path::PathBuf;

const APP_DIR_NAME: &str = "D-Medic";

fn base_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let base = dirs::data_dir().ok_or("data_dir bulunamadı (%APPDATA%)")?;
    Ok(base.join(APP_DIR_NAME))
}

pub fn log_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(base_dir()?.join("logs"))
}

pub fn snapshot_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(base_dir()?.join("snapshots"))
}

pub fn backups_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(base_dir()?.join("backups"))
}

pub fn settings_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(base_dir()?.join("settings.json"))
}

pub fn cache_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(base_dir()?.join("cache"))
}
