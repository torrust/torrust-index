//! Configuration for the application.
use std::path::Path;
use std::{env, fs};

use config::{Config, ConfigError, File, FileFormat};
use log::warn;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Information displayed to the user in the website.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Website {
    /// The name of the website.
    pub name: String,
}

impl Default for Website {
    fn default() -> Self {
        Self {
            name: "Torrust".to_string(),
        }
    }
}

/// See `TrackerMode` in [`torrust-tracker-primitives`](https://docs.rs/torrust-tracker-primitives)
/// crate for more information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrackerMode {
    // todo: use https://crates.io/crates/torrust-tracker-primitives
    /// Will track every new info hash and serve every peer.
    Public,
    /// Will only serve authenticated peers.
    Private,
    /// Will only track whitelisted info hashes.
    Whitelisted,
    /// Will only track whitelisted info hashes and serve authenticated peers.
    PrivateWhitelisted,
}

impl Default for TrackerMode {
    fn default() -> Self {
        Self::Public
    }
}

/// Configuration for the associated tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tracker {
    /// Connection string for the tracker. For example: `udp://TRACKER_IP:6969`.
    pub url: String,
    /// The mode of the tracker. For example: `Public`.
    /// See `TrackerMode` in [`torrust-tracker-primitives`](https://docs.rs/torrust-tracker-primitives)
    /// crate for more information.
    pub mode: TrackerMode,
    /// The url of the tracker API. For example: `http://localhost:1212`.
    pub api_url: String,
    /// The token used to authenticate with the tracker API.
    pub token: String,
    /// The amount of seconds the token is valid.
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

/// Port number representing that the OS will choose one randomly from the available ports.
///
/// It's the port number `0`
pub const FREE_PORT: u16 = 0;

/// The the base URL for the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    /// The port to listen on. Default to `3000`.
    pub port: u16,
    /// The base URL for the API. For example: `http://localhost`.
    /// If not set, the base URL will be inferred from the request.
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

/// Whether the email is required on signup or not.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailOnSignup {
    /// The email is required on signup.
    Required,
    /// The email is optional on signup.
    Optional,
    /// The email is not allowed on signup. It will only be ignored if provided.
    None, // code-review: rename to `Ignored`?
}

impl Default for EmailOnSignup {
    fn default() -> Self {
        Self::Optional
    }
}

/// Authentication options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auth {
    /// Whether or not to require an email on signup.
    pub email_on_signup: EmailOnSignup,
    /// The minimum password length.
    pub min_password_length: usize,
    /// The maximum password length.
    pub max_password_length: usize,
    /// The secret key used to sign JWT tokens.
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

/// Database configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    /// The connection string for the database. For example: `sqlite://data.db?mode=rwc`.
    pub connect_url: String,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            connect_url: "sqlite://data.db?mode=rwc".to_string(),
        }
    }
}

/// SMTP configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mail {
    /// Whether or not to enable email verification on signup.
    pub email_verification_enabled: bool,
    /// The email address to send emails from.
    pub from: String,
    /// The email address to reply to.
    pub reply_to: String,
    /// The username to use for SMTP authentication.
    pub username: String,
    /// The password to use for SMTP authentication.
    pub password: String,
    /// The SMTP server to use.
    pub server: String,
    /// The SMTP port to use.
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

/// Configuration for the image proxy cache.
///
/// Users have a cache quota per period. For example: 100MB per day.
/// When users are navigating the site, they will be downloading images that are
/// embedded in the torrent description. These images will be cached in the
/// proxy. The proxy will not download new images if the user has reached the
/// quota.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageCache {
    /// Maximum time in seconds to wait for downloading the image form the original source.
    pub max_request_timeout_ms: u64,
    /// Cache size in bytes.
    pub capacity: usize,
    /// Maximum size in bytes for a single image.
    pub entry_size_limit: usize,
    /// Users have a cache quota per period. For example: 100MB per day.
    /// This is the period in seconds (1 day in seconds).
    pub user_quota_period_seconds: u64,
    /// Users have a cache quota per period. For example: 100MB per day.
    /// This is the maximum size in bytes (100MB in bytes).    
    pub user_quota_bytes: usize,
}

/// Core configuration for the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Api {
    /// The default page size for torrent lists.
    pub default_torrent_page_size: u8,
    /// The maximum page size for torrent lists.
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

/// Configuration for the tracker statistics importer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerStatisticsImporter {
    /// The interval in seconds to get statistics from the tracker.
    pub torrent_info_update_interval: u64,
}

impl Default for TrackerStatisticsImporter {
    fn default() -> Self {
        Self {
            torrent_info_update_interval: 3600,
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

/// The whole configuration for the backend.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TorrustBackend {
    /// Logging level. Possible values are: `Off`, `Error`, `Warn`, `Info`,
    /// `Debug` and `Trace`. Default is `Info`.
    pub log_level: Option<String>,
    /// The website customizable values.
    pub website: Website,
    /// The tracker configuration.
    pub tracker: Tracker,
    /// The network configuration.
    pub net: Network,
    /// The authentication configuration.
    pub auth: Auth,
    /// The database configuration.
    pub database: Database,
    /// The SMTP configuration.
    pub mail: Mail,
    /// The image proxy cache configuration.
    pub image_cache: ImageCache,
    /// The API configuration.
    pub api: Api,
    /// The tracker statistics importer job configuration.
    pub tracker_statistics_importer: TrackerStatisticsImporter,
}

/// The configuration service.
#[derive(Debug)]
pub struct Configuration {
    /// The state of the configuration.
    pub settings: RwLock<TorrustBackend>,
    /// The path to the configuration file. This is `None` if the configuration
    /// was loaded from the environment.
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
            warn!("No config file found. Creating default config file ...");

            let config = Configuration::default();
            let _ = config.save_to_file(config_path).await;

            return Err(ConfigError::Message(format!(
                "No config file found. Created default config file in {config_path}. Edit the file and start the application."
            )));
        }

        let torrust_config: TorrustBackend = match config.try_deserialize() {
            Ok(data) => Ok(data),
            Err(e) => Err(ConfigError::Message(format!("Errors while processing config: {e}."))),
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

    pub async fn get_all(&self) -> TorrustBackend {
        let settings_lock = self.settings.read().await;

        settings_lock.clone()
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

    pub async fn get_site_name(&self) -> String {
        let settings_lock = self.settings.read().await;

        settings_lock.website.name.clone()
    }
}

/// The public backend configuration.
/// There is an endpoint to get this configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationPublic {
    website_name: String,
    tracker_url: String,
    tracker_mode: TrackerMode,
    email_on_signup: EmailOnSignup,
}
