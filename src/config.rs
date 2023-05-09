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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrackerMode {
    // todo: use https://crates.io/crates/torrust-tracker-primitives
    Public,
    Private,
    Whitelisted,
    PrivateWhitelisted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tracker {
    pub url: String,
    pub mode: TrackerMode,
    pub api_url: String,
    pub token: String,
    pub token_valid_seconds: u64,
}

/// Port 0 means that the OS will choose a random free port.
pub const FREE_PORT: u16 = 0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub port: u16,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailOnSignup {
    Required,
    Optional,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auth {
    pub email_on_signup: EmailOnSignup,
    pub min_password_length: usize,
    pub max_password_length: usize,
    pub secret_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub connect_url: String,
    pub torrent_info_update_interval: u64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfiguration {
    pub website: Website,
    pub tracker: Tracker,
    pub net: Network,
    pub auth: Auth,
    pub database: Database,
    pub mail: Mail,
    pub image_cache: ImageCache,
    pub api: Api,
}

impl Default for AppConfiguration {
    fn default() -> Self {
        Self {
            website: Website {
                name: "Torrust".to_string(),
            },
            tracker: Tracker {
                url: "udp://localhost:6969".to_string(),
                mode: TrackerMode::Public,
                api_url: "http://localhost:1212".to_string(),
                token: "MyAccessToken".to_string(),
                token_valid_seconds: 7_257_600,
            },
            net: Network {
                port: 3000,
                base_url: None,
            },
            auth: Auth {
                email_on_signup: EmailOnSignup::Optional,
                min_password_length: 6,
                max_password_length: 64,
                secret_key: "MaxVerstappenWC2021".to_string(),
            },
            database: Database {
                connect_url: "sqlite://data.db?mode=rwc".to_string(),
                torrent_info_update_interval: 3600,
            },
            mail: Mail {
                email_verification_enabled: false,
                from: "example@email.com".to_string(),
                reply_to: "noreply@email.com".to_string(),
                username: String::new(),
                password: String::new(),
                server: String::new(),
                port: 25,
            },
            image_cache: ImageCache {
                max_request_timeout_ms: 1000,
                capacity: 128_000_000,
                entry_size_limit: 4_000_000,
                user_quota_period_seconds: 3600,
                user_quota_bytes: 64_000_000,
            },
            api: Api {
                default_torrent_page_size: 10,
                max_torrent_page_size: 30,
            },
        }
    }
}

#[derive(Debug)]
pub struct Configuration {
    pub settings: RwLock<AppConfiguration>,
    pub config_path: Option<String>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            settings: RwLock::new(AppConfiguration::default()),
            config_path: None,
        }
    }
}

impl Configuration {
    /// Loads the configuration from the configuration file.
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

        let torrust_config: AppConfiguration = match config.try_deserialize() {
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
                let torrust_config: AppConfiguration = config_builder.try_deserialize()?;
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

    pub async fn save_to_file(&self, config_path: &str) {
        let settings = self.settings.read().await;

        let toml_string = toml::to_string(&*settings).expect("Could not encode TOML value");

        drop(settings);

        fs::write(config_path, toml_string).expect("Could not write to file!");
    }

    /// Updates the settings and saves them to the configuration file.
    ///
    /// # Panics
    ///
    /// Will panic if the configuration file path is not defined. That happens
    /// when the configuration was loaded from the environment variable.
    pub async fn update_settings(&self, new_settings: AppConfiguration) {
        match &self.config_path {
            Some(config_path) => {
                let mut settings = self.settings.write().await;
                *settings = new_settings;

                drop(settings);

                let _ = self.save_to_file(config_path).await;
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
