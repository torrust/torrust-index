use std::env;

pub const CONFIG_PATH: &str = "./config.toml";
pub const CONFIG_ENV_VAR_NAME: &str = "TORRUST_IDX_BACK_CONFIG";

use crate::config::Configuration;

/// Initialize configuration from file or env var.
///
/// # Panics
///
/// Will panic if configuration is not found or cannot be parsed
pub async fn init_configuration() -> Configuration {
    if env::var(CONFIG_ENV_VAR_NAME).is_ok() {
        println!("Loading configuration from env var `{}`", CONFIG_ENV_VAR_NAME);

        Configuration::load_from_env_var(CONFIG_ENV_VAR_NAME).unwrap()
    } else {
        println!("Loading configuration from config file `{}`", CONFIG_PATH);

        match Configuration::load_from_file(CONFIG_PATH).await {
            Ok(config) => config,
            Err(error) => {
                panic!("{}", error)
            }
        }
    }
}
