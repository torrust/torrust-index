//! Initialize configuration from file or env var.
//!
//! All environment variables are prefixed with `TORRUST_IDX_BACK_`.
use std::env;

// Environment variables

/// The whole `config.toml` file content. It has priority over the config file.
/// Even if the file is not on the default path.
pub const ENV_VAR_CONFIG: &str = "TORRUST_INDEX_CONFIG";

/// The `config.toml` file location.
pub const ENV_VAR_CONFIG_PATH: &str = "TORRUST_INDEX_CONFIG_PATH";

/// If present, CORS will be permissive.
pub const ENV_VAR_CORS_PERMISSIVE: &str = "TORRUST_INDEX_CORS_PERMISSIVE";

// Default values

pub const ENV_VAR_DEFAULT_CONFIG_PATH: &str = "./config.toml";

use crate::config::Configuration;

/// Initialize configuration from file or env var.
///
/// # Panics
///
/// Will panic if configuration is not found or cannot be parsed
pub async fn init_configuration() -> Configuration {
    if env::var(ENV_VAR_CONFIG).is_ok() {
        println!("Loading configuration from env var `{ENV_VAR_CONFIG}`");

        Configuration::load_from_env_var(ENV_VAR_CONFIG).unwrap()
    } else {
        let config_path = env::var(ENV_VAR_CONFIG_PATH).unwrap_or_else(|_| ENV_VAR_DEFAULT_CONFIG_PATH.to_string());

        println!("Loading configuration from config file `{config_path}`");

        match Configuration::load_from_file(&config_path).await {
            Ok(config) => config,
            Err(error) => {
                panic!("{}", error)
            }
        }
    }
}
