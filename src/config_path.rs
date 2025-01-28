//! Configuration path management for PetCLI
//!
//! This module handles the management of configuration file paths and directories,
//! ensuring proper setup of the application's configuration structure. It provides
//! functionality for:
//! - Locating and creating the configuration directory
//! - Managing configuration file paths
//! - Ensuring configuration directory exists
//!
//! Consider moving this into a broader configuration management module if the
//! configuration system becomes more complex.

use std::path::PathBuf;

/// Returns the path to the PetCLI configuration directory
pub fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".config").join("petcli")
}

/// Ensures the configuration directory exists, creating it if necessary
pub fn ensure_config_dir() -> std::io::Result<PathBuf> {
    let config_dir = get_config_dir();
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?
    }
    Ok(config_dir)
}

/// Returns the path to a configuration file
///
/// If a name is provided, returns the path to that specific configuration file.
/// Otherwise, returns the path to the default config.toml file.
pub fn get_config_file_path(name: Option<&str>) -> PathBuf {
    let config_dir = get_config_dir();
    match name {
        Some(name) => config_dir.join(name).with_extension("toml"),
        None => config_dir.join("config.toml"),
    }
}