//! Initialize configuration for the shared E2E tests environment from a
//! config file `config.toml` or env var.
//!
//! All environment variables are prefixed with `TORRUST_INDEX_E2E_`.

// Environment variables

use torrust_index::bootstrap::config::{ENV_VAR_API_ADMIN_TOKEN, ENV_VAR_AUTH_SECRET_KEY, ENV_VAR_CONFIG, ENV_VAR_PATH_CONFIG};
use torrust_index::config::{Configuration, Info};

// Default values
pub const DEFAULT_PATH_CONFIG: &str = "./share/default/config/index.development.sqlite3.toml";

/// If present, E2E tests will run against a shared instance of the server
pub const ENV_VAR_INDEX_SHARED: &str = "TORRUST_INDEX_E2E_SHARED";

/// `SQLx` connection string to connect to the E2E database
pub const ENV_VAR_DB_CONNECT_URL: &str = "TORRUST_INDEX_E2E_DB_CONNECT_URL";

/// It loads the application configuration from the environment.
///
/// There are two methods to inject the configuration:
///
/// 1. By using a config file: `index.toml`.
/// 2. Environment variable: `TORRUST_INDEX_CONFIG`. The variable contains the same contents as the `index.toml` file.
///
/// Environment variable has priority over the config file.
///
/// Refer to the [configuration documentation](https://docs.rs/torrust-index-configuration) for the configuration options.
///
/// # Panics
///
/// Will panic if it can't load the configuration from either
/// `./index.toml` file or the env var `TORRUST_INDEX_CONFIG`.
#[must_use]
pub fn initialize_configuration() -> Configuration {
    let info = Info::new(
        ENV_VAR_CONFIG.to_string(),
        ENV_VAR_PATH_CONFIG.to_string(),
        DEFAULT_PATH_CONFIG.to_string(),
        ENV_VAR_API_ADMIN_TOKEN.to_string(),
        ENV_VAR_AUTH_SECRET_KEY.to_string(),
    )
    .unwrap();

    Configuration::load(&info).unwrap()
}

#[cfg(test)]
mod tests {
    use torrust_index::bootstrap::config::initialize_configuration;

    #[test]
    fn it_should_load_with_default_config() {
        drop(initialize_configuration());
    }
}
