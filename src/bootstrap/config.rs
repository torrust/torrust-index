//! Initialize configuration from file or env var.
//!
//! All environment variables are prefixed with `TORRUST_INDEX_`.

// Environment variables

// Default values
/// Re-exported from `torrust-index-config` so the application,
/// helper binaries, and integration tests share one source of
/// truth for the default config-TOML location.
pub use torrust_index_config::DEFAULT_CONFIG_TOML_PATH as DEFAULT_PATH_CONFIG;

use crate::config::{Configuration, Info};

/// If present, CORS will be permissive.
pub const ENV_VAR_CORS_PERMISSIVE: &str = "TORRUST_INDEX_API_CORS_PERMISSIVE";

/// It loads the application configuration from the environment.
///
/// There are two methods to inject the configuration:
///
/// 1. By using a config file: `index.toml`.
/// 2. Environment variable: `TORRUST_INDEX_CONFIG_TOML`. The variable contains the same contents as the `index.toml` file.
///
/// Environment variable has priority over the config file.
///
/// Refer to the [configuration documentation](https://docs.rs/torrust-index-configuration) for the configuration options.
///
/// # Panics
///
/// Will panic if it can't load the configuration from either
/// `./index.toml` file or the env var `TORRUST_INDEX_CONFIG_TOML`.
#[must_use]
pub fn initialize_configuration() -> Configuration {
    let info = Info::new(DEFAULT_PATH_CONFIG.to_string()).unwrap();

    Configuration::load(&info).unwrap()
}
