use std::{env, fs, path::PathBuf};

use log::info;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("unable to read config: {0}")]
    UnableToReadConfig(PathBuf),
    #[error("invalid toml file: {0}")]
    TomlSyntaxError(String),
    #[error("$HOME is not defined")]
    HomeNotDefined,
}

#[derive(Deserialize)]
pub struct Config {
    pub startup: Vec<String>,
}

pub fn load_config() -> Result<Config, ConfigError> {
    let home_dir = match env::var("HOME") {
        Ok(home_dir) => home_dir,
        Err(_e) => return Err(ConfigError::HomeNotDefined),
    };

    let config_path: PathBuf = [home_dir.as_str(), ".config", "tdawm", "tdawm.toml"]
        .iter()
        .collect();
    let config_content = fs::read_to_string(config_path.clone())
        .map_err(|_| ConfigError::UnableToReadConfig(config_path))?;

    let config: Config =
        toml::from_str(&config_content).map_err(|e| ConfigError::TomlSyntaxError(e.to_string()))?;
    info!("config loaded !");
    Ok(config)
}
