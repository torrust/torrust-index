use std::path::Path;
use std::{env, fs};

use config::{Config, ConfigError, File, FileFormat};
use log::warn;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Website {
    pub name: String,
}

impl Default for Website {
    fn default() -> Self {
        Self {
            name: "Torrust".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrackerMode {
    // todo: use https://crates.io/crates/torrust-tracker-primitives
    Public,
    Private,
    Whitelisted,
    PrivateWhitelisted,
}

impl Default for TrackerMode {
    fn default() -> Self {
        Self::Public
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tracker {
    pub url: String,
    pub mode: TrackerMode,
    pub api_url: String,
    pub token: String,
    pub token_valid_seconds: u64,
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            url: "udp://localhost:6969".to_string(),
            mode: TrackerMode::default(),
            api_url: "http://localhost:1212".to_string(),
            token: "MyAccessToken".to_string(),
            token_valid_seconds: 7_257_600,
        }
    }
}

/// Port 0 means that the OS will choose a random free port.
pub const FREE_PORT: u16 = 0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub port: u16,
    pub base_url: Option<String>,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            port: 3000,
            base_url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailOnSignup {
    Required,
    Optional,
    None,
}

impl Default for EmailOnSignup {
    fn default() -> Self {
        Self::Optional
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auth {
    pub email_on_signup: EmailOnSignup,
    pub min_password_length: usize,
    pub max_password_length: usize,
    pub secret_key: String,
}

impl Default for Auth {
    fn default() -> Self {
        Self {
            email_on_signup: EmailOnSignup::default(),
            min_password_length: 6,
            max_password_length: 64,
            secret_key: "MaxVerstappenWC2021".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub connect_url: String,
    pub torrent_info_update_interval: u64,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            connect_url: "sqlite://data.db?mode=rwc".to_string(),
            torrent_info_update_interval: 3600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mail {
    pub email_verification_enabled: bool,
    pub from: String,
    pub reply_to: String,
    pub username: String,
    pub password: String,
    pub server: String,
    pub port: u16,
}

impl Default for Mail {
    fn default() -> Self {
        Self {
            email_verification_enabled: false,
            from: "example@email.com".to_string(),
            reply_to: "noreply@email.com".to_string(),
            username: String::default(),
            password: String::default(),
            server: String::default(),
            port: 25,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageCache {
    pub max_request_timeout_ms: u64,
    pub capacity: usize,
    pub entry_size_limit: usize,
    pub user_quota_period_seconds: u64,
    pub user_quota_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Api {
    pub default_torrent_page_size: u8,
    pub max_torrent_page_size: u8,
}

impl Default for Api {
    fn default() -> Self {
        Self {
            default_torrent_page_size: 10,
            max_torrent_page_size: 30,
        }
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self {
            max_request_timeout_ms: 1000,
            capacity: 128_000_000,
            entry_size_limit: 4_000_000,
            user_quota_period_seconds: 3600,
            user_quota_bytes: 64_000_000,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TorrustBackend {
    pub website: Website,
    pub tracker: Tracker,
    pub net: Network,
    pub auth: Auth,
    pub database: Database,
    pub mail: Mail,
    pub image_cache: ImageCache,
    pub api: Api,
}

#[derive(Debug)]
pub struct Configuration {
    pub settings: RwLock<TorrustBackend>,
    pub config_path: Option<String>,
}

impl Default for Configuration {
    fn default() -> Configuration {
        Configuration {
            settings: RwLock::new(TorrustBackend::default()),
            config_path: None,
        }
    }
}

impl Configuration {
    /// Loads the configuration from the configuration file.
    ///
    /// # Errors
    ///
    /// This function will return an error no configuration in the `CONFIG_PATH` exists, and a new file is is created.
    /// This function will return an error if the `config` is not a valid `TorrustConfig` document.
    pub async fn load_from_file(config_path: &str) -> Result<Configuration, ConfigError> {
        let config_builder = Config::builder();

        #[allow(unused_assignments)]
        let mut config = Config::default();

        if Path::new(config_path).exists() {
            config = config_builder.add_source(File::with_name(config_path)).build()?;
        } else {
            warn!("No config file found.");
            warn!("Creating config file..");
            let config = Configuration::default();
            let _ = config.save_to_file(config_path).await;
            return Err(ConfigError::Message(
                "Please edit the config.TOML in the root folder and restart the tracker.".to_string(),
            ));
        }

        let torrust_config: TorrustBackend = match config.try_deserialize() {
            Ok(data) => Ok(data),
            Err(e) => Err(ConfigError::Message(format!("Errors while processing config: {}.", e))),
        }?;

        Ok(Configuration {
            settings: RwLock::new(torrust_config),
            config_path: Some(config_path.to_string()),
        })
    }

    /// Loads the configuration from the environment variable. The whole
    /// configuration must be in the environment variable. It contains the same
    /// configuration as the configuration file with the same format.
    ///
    /// # Errors
    ///
    /// Will return `Err` if the environment variable does not exist or has a bad configuration.
    pub fn load_from_env_var(config_env_var_name: &str) -> Result<Configuration, ConfigError> {
        match env::var(config_env_var_name) {
            Ok(config_toml) => {
                let config_builder = Config::builder()
                    .add_source(File::from_str(&config_toml, FileFormat::Toml))
                    .build()?;
                let torrust_config: TorrustBackend = config_builder.try_deserialize()?;
                Ok(Configuration {
                    settings: RwLock::new(torrust_config),
                    config_path: None,
                })
            }
            Err(_) => Err(ConfigError::Message(
                "Unable to load configuration from the configuration environment variable.".to_string(),
            )),
        }
    }

    /// Returns the save to file of this [`Configuration`].
    pub async fn save_to_file(&self, config_path: &str) {
        let settings = self.settings.read().await;

        let toml_string = toml::to_string(&*settings).expect("Could not encode TOML value");

        drop(settings);

        fs::write(config_path, toml_string).expect("Could not write to file!");
    }

    /// Update the settings file based upon a supplied `new_settings`.
    ///
    /// # Errors
    ///
    /// Todo: Make an error if the save fails.
    ///
    /// # Panics
    ///
    /// Will panic if the configuration file path is not defined. That happens
    /// when the configuration was loaded from the environment variable.
    pub async fn update_settings(&self, new_settings: TorrustBackend) -> Result<(), ()> {
        match &self.config_path {
            Some(config_path) => {
                let mut settings = self.settings.write().await;
                *settings = new_settings;

                drop(settings);

                let _ = self.save_to_file(config_path).await;

                Ok(())
            }
            None => panic!(
                "Cannot update settings when the config file path is not defined. For example: when it's loaded from env var."
            ),
        }
    }

    pub async fn get_public(&self) -> ConfigurationPublic {
        let settings_lock = self.settings.read().await;

        ConfigurationPublic {
            website_name: settings_lock.website.name.clone(),
            tracker_url: settings_lock.tracker.url.clone(),
            tracker_mode: settings_lock.tracker.mode.clone(),
            email_on_signup: settings_lock.auth.email_on_signup.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationPublic {
    website_name: String,
    tracker_url: String,
    tracker_mode: TrackerMode,
    email_on_signup: EmailOnSignup,
}
