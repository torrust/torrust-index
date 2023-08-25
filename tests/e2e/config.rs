//! Initialize configuration for the shared E2E tests environment from a
//! config file `config.toml` or env var.
//!
//! All environment variables are prefixed with `TORRUST_IDX_BACK_E2E`.
use std::env;

use torrust_index_backend::config::Configuration;

// Environment variables

/// If present, E2E tests will run against a shared instance of the server
pub const ENV_VAR_E2E_SHARED: &str = "TORRUST_IDX_BACK_E2E_SHARED";

/// The whole `config.toml` file content. It has priority over the config file.
pub const ENV_VAR_E2E_CONFIG: &str = "TORRUST_IDX_BACK_E2E_CONFIG";

/// The `config.toml` file location.
pub const ENV_VAR_E2E_CONFIG_PATH: &str = "TORRUST_IDX_BACK_E2E_CONFIG_PATH";

// Default values

pub const ENV_VAR_E2E_DEFAULT_CONFIG_PATH: &str = "./config-index.local.toml";

/// Initialize configuration from file or env var.
///
/// # Panics
///
/// Will panic if configuration is not found or cannot be parsed
pub async fn init_shared_env_configuration() -> Configuration {
    if env::var(ENV_VAR_E2E_CONFIG).is_ok() {
        println!("Loading configuration for E2E env from env var `{ENV_VAR_E2E_CONFIG}`");

        Configuration::load_from_env_var(ENV_VAR_E2E_CONFIG).unwrap()
    } else {
        let config_path = env::var(ENV_VAR_E2E_CONFIG_PATH).unwrap_or_else(|_| ENV_VAR_E2E_DEFAULT_CONFIG_PATH.to_string());

        println!("Loading configuration from config file `{config_path}`");

        match Configuration::load_from_file(&config_path).await {
            Ok(config) => config,
            Err(error) => {
                panic!("{}", error)
            }
        }
    }
}
