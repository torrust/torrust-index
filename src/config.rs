use config::{ConfigError, Config, File};
use std::path::Path;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Auth {
    pub min_password_length: usize,
    pub max_password_length: usize,
    pub secret_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
    pub connect_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Storage {
    pub upload_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TorrustConfig {
    pub auth: Auth,
    pub database: Database,
    pub storage: Storage,
}

impl TorrustConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let mut config = Config::new();

        const CONFIG_PATH: &str = "./config.toml";

        if Path::new(CONFIG_PATH).exists() {
            config.merge(File::with_name(CONFIG_PATH))?;
        }

        match config.try_into() {
            Ok(data) => Ok(data),
            Err(e) => Err(ConfigError::Message(format!("Errors while processing config: {}.", e))),
        }
    }
}
