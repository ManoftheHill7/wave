use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct GameConfig {
    pub audio: AudioConfig,
}

#[derive(Serialize, Deserialize)]
pub struct AudioConfig {
    pub muted: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        AudioConfig { muted: false }
    }
}

fn get_config_path() -> PathBuf {
    let mut path = if let Some(data_dir) = dirs::config_dir() {
        data_dir
    } else {
        PathBuf::from(".")
    };
    path.push("waves");
    path.push("config.toml");
    path
}

pub fn load_config() -> GameConfig {
    let config_path = get_config_path();

    if let Ok(config_str) = fs::read_to_string(&config_path) {
        if let Ok(config) = toml::from_str(&config_str) {
            return config;
        }
    }

    // Return default config if file doesn't exist or parsing fails
    GameConfig::default()
}

pub fn save_config(config: &GameConfig) -> Result<(), String> {
    let config_path = get_config_path();

    // Create parent directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let config_str =
        toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, config_str)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}
