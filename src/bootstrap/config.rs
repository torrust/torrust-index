//! Initialize configuration from file or env var.
//!
//! All environment variables are prefixed with `TORRUST_IDX_BACK_`.
use std::env;

// Environment variables

/// The whole `config.toml` file content. It has priority over the config file.
pub const ENV_VAR_CONFIG: &str = "TORRUST_IDX_BACK_CONFIG";

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
        println!("Loading configuration from env var `{}`", ENV_VAR_CONFIG);

        Configuration::load_from_env_var(ENV_VAR_CONFIG).unwrap()
    } else {
        println!("Loading configuration from config file `{}`", ENV_VAR_DEFAULT_CONFIG_PATH);

        match Configuration::load_from_file(ENV_VAR_DEFAULT_CONFIG_PATH).await {
            Ok(config) => config,
            Err(error) => {
                panic!("{}", error)
            }
        }
    }
}
