use std::path::PathBuf;

pub fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".config").join("petcli")
}

pub fn ensure_config_dir() -> std::io::Result<PathBuf> {
    let config_dir = get_config_dir();
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?
    }
    Ok(config_dir)
}

pub fn get_config_file_path(name: Option<&str>) -> PathBuf {
    let config_dir = get_config_dir();
    match name {
        Some(name) => config_dir.join(name).with_extension("toml"),
        None => config_dir.join("config.toml"),
    }
}