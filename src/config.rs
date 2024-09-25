use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub api_endpoint: String,
    pub model: String,
    pub temperature: f32,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config_dir = dirs::home_dir()
            .ok_or(ConfigError::HomeDirNotFound)?
            .join(".config")
            .join("chat_tui");

        fs::create_dir_all(&config_dir).map_err(|e| ConfigError::IoError(e))?;

        let config_path = config_dir.join("config.toml");

        if !config_path.exists() {
            Self::create_default_config(&config_path)?;
        }

        let config_content =
            fs::read_to_string(&config_path).map_err(|e| ConfigError::IoError(e))?;
        let config: Config =
            toml::from_str(&config_content).map_err(|e| ConfigError::TomlParseError(e))?;

        Ok(config)
    }

    fn create_default_config(path: &PathBuf) -> Result<(), ConfigError> {
        let default_config = Config {
            api_endpoint: "".to_string(),
            model: "".to_string(),
            temperature: 0.7,
        };

        let toml_string =
            toml::to_string(&default_config).map_err(|e| ConfigError::TomlSerializeError(e))?;
        fs::write(path, toml_string).map_err(|e| ConfigError::IoError(e))?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("home directory not found")]
    HomeDirNotFound,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParseError(#[from] toml::de::Error),

    #[error("Toml serialize error: {0}")]
    TomlSerializeError(#[from] toml::ser::Error),
}
