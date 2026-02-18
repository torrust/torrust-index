//! Initialize configuration from file or env var.
//!
//! All environment variables are prefixed with `TORRUST_INDEX_`.

// Environment variables

use crate::config::{Configuration, Info};

// Default values
pub const DEFAULT_PATH_CONFIG: &str = "./share/default/config/index.development.sqlite3.toml";

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

#[cfg(test)]
mod tests {

    #[test]
    fn it_should_load_with_default_config() {
        // Use an absolute path derived from CARGO_MANIFEST_DIR so this test
        // is not affected by figment::Jail tests that change the process-wide
        // current working directory in parallel.
        let config_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/share/default/config/index.development.sqlite3.toml"
        );
        let config_content = std::fs::read_to_string(config_path)
            .unwrap_or_else(|e| panic!("Could not read default config at {config_path}: {e}"));
        let info = crate::config::Info::from_toml(&config_content);
        drop(crate::config::Configuration::load(&info).unwrap());
    }
}
