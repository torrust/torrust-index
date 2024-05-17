//! Configuration for the application.
use std::sync::Arc;
use std::{env, fs};

use camino::Utf8PathBuf;
use config::{Config, ConfigError, File, FileFormat};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};
use thiserror::Error;
use tokio::sync::RwLock;
use torrust_index_located_error::{Located, LocatedError};
use url::{ParseError, Url};

/// Information required for loading config
#[derive(Debug, Default, Clone)]
pub struct Info {
    index_toml: String,
    tracker_api_token: Option<String>,
    auth_secret_key: Option<String>,
}

impl Info {
    /// Build Configuration Info
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use torrust_index::config::Info;
    /// # let (env_var_config, env_var_path_config, default_path_config, env_var_tracker_api_token, env_var_auth_secret_key) = ("".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string());
    /// let result = Info::new(env_var_config, env_var_path_config, default_path_config, env_var_tracker_api_token, env_var_auth_secret_key);
    /// ```
    ///
    /// # Errors
    ///
    /// Will return `Err` if unable to obtain a configuration.
    ///
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(
        env_var_config: String,
        env_var_path_config: String,
        default_path_config: String,
        env_var_tracker_api_token: String,
        env_var_auth_secret_key: String,
    ) -> Result<Self, Error> {
        let index_toml = if let Ok(index_toml) = env::var(&env_var_config) {
            println!("Loading configuration from env var {env_var_config} ...");

            index_toml
        } else {
            let config_path = if let Ok(config_path) = env::var(env_var_path_config) {
                println!("Loading configuration file: `{config_path}` ...");

                config_path
            } else {
                println!("Loading default configuration file: `{default_path_config}` ...");

                default_path_config
            };

            fs::read_to_string(config_path)
                .map_err(|e| Error::UnableToLoadFromConfigFile {
                    source: (Arc::new(e) as Arc<dyn std::error::Error + Send + Sync>).into(),
                })?
                .parse()
                .map_err(|_e: std::convert::Infallible| Error::Infallible)?
        };

        let tracker_api_token = env::var(env_var_tracker_api_token).ok();
        let auth_secret_key = env::var(env_var_auth_secret_key).ok();

        Ok(Self {
            index_toml,
            tracker_api_token,
            auth_secret_key,
        })
    }
}

/// Errors that can occur when loading the configuration.
#[derive(Error, Debug)]
pub enum Error {
    /// Unable to load the configuration from the environment variable.
    /// This error only occurs if there is no configuration file and the
    /// `TORRUST_TRACKER_CONFIG_TOML` environment variable is not set.
    #[error("Unable to load from Environmental Variable: {source}")]
    UnableToLoadFromEnvironmentVariable {
        source: LocatedError<'static, dyn std::error::Error + Send + Sync>,
    },

    #[error("Unable to load from Config File: {source}")]
    UnableToLoadFromConfigFile {
        source: LocatedError<'static, dyn std::error::Error + Send + Sync>,
    },

    /// Unable to load the configuration from the configuration file.
    #[error("Failed processing the configuration: {source}")]
    ConfigError { source: LocatedError<'static, ConfigError> },

    #[error("The error for errors that can never happen.")]
    Infallible,
}

/// Errors that can occur validating the configuration.
#[derive(Error, Debug)]
pub enum ValidationError {
    /// Unable to load the configuration from the configuration file.
    #[error("Invalid tracker URL: {source}")]
    InvalidTrackerUrl { source: LocatedError<'static, ParseError> },

    #[error("UDP private trackers are not supported. URL schemes for private tracker URLs must be HTTP ot HTTPS")]
    UdpTrackersInPrivateModeNotSupported,
}

impl From<ConfigError> for Error {
    #[track_caller]
    fn from(err: ConfigError) -> Self {
        Self::ConfigError {
            source: Located(err).into(),
        }
    }
}

/// Information displayed to the user in the website.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

impl TrackerMode {
    #[must_use]
    pub fn is_open(&self) -> bool {
        matches!(self, TrackerMode::Public | TrackerMode::Whitelisted)
    }

    #[must_use]
    pub fn is_close(&self) -> bool {
        !self.is_open()
    }
}

/// Configuration for the associated tracker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

impl Tracker {
    fn override_tracker_api_token(&mut self, tracker_api_token: &str) {
        self.token = tracker_api_token.to_string();
    }
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Network {
    /// The port to listen on. Default to `3001`.
    pub port: u16,
    /// The base URL for the API. For example: `http://localhost`.
    /// If not set, the base URL will be inferred from the request.
    pub base_url: Option<String>,
    /// TSL configuration.
    pub tsl: Option<Tsl>,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            port: 3001,
            base_url: None,
            tsl: None,
        }
    }
}

/// Whether the email is required on signup or not.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

impl Auth {
    fn override_secret_key(&mut self, secret_key: &str) {
        self.secret_key = secret_key.to_string();
    }
}

/// Database configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackerStatisticsImporter {
    /// The interval in seconds to get statistics from the tracker.
    pub torrent_info_update_interval: u64,
    /// The port the Importer API is listening on. Default to `3002`.
    pub port: u16,
}

impl Default for TrackerStatisticsImporter {
    fn default() -> Self {
        Self {
            torrent_info_update_interval: 3600,
            port: 3002,
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

#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct Tsl {
    /// Path to the SSL certificate file.
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default = "Tsl::default_ssl_cert_path")]
    pub ssl_cert_path: Option<Utf8PathBuf>,
    /// Path to the SSL key file.
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default = "Tsl::default_ssl_key_path")]
    pub ssl_key_path: Option<Utf8PathBuf>,
}

impl Tsl {
    #[allow(clippy::unnecessary_wraps)]
    fn default_ssl_cert_path() -> Option<Utf8PathBuf> {
        Some(Utf8PathBuf::new())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn default_ssl_key_path() -> Option<Utf8PathBuf> {
        Some(Utf8PathBuf::new())
    }
}

/// The whole configuration for the index.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct TorrustIndex {
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

impl TorrustIndex {
    fn override_tracker_api_token(&mut self, tracker_api_token: &str) {
        self.tracker.override_tracker_api_token(tracker_api_token);
    }

    fn override_auth_secret_key(&mut self, auth_secret_key: &str) {
        self.auth.override_secret_key(auth_secret_key);
    }

    pub fn remove_secrets(&mut self) {
        "***".clone_into(&mut self.tracker.token);
        "***".clone_into(&mut self.database.connect_url);
        "***".clone_into(&mut self.mail.password);
        "***".clone_into(&mut self.auth.secret_key);
    }
}

/// The configuration service.
#[derive(Debug)]
pub struct Configuration {
    /// The state of the configuration.
    pub settings: RwLock<TorrustIndex>,
    /// The path to the configuration file. This is `None` if the configuration
    /// was loaded from the environment.
    pub config_path: Option<String>,
}

impl Default for Configuration {
    fn default() -> Configuration {
        Configuration {
            settings: RwLock::new(TorrustIndex::default()),
            config_path: None,
        }
    }
}

impl Configuration {
    /// Loads the configuration from the `Info` struct. The whole
    /// configuration in toml format is included in the `info.index_toml` string.
    ///
    /// Optionally will override the:
    ///
    /// - Tracker api token.
    /// - The auth secret key.
    ///
    /// # Errors
    ///
    /// Will return `Err` if the environment variable does not exist or has a bad configuration.
    pub fn load(info: &Info) -> Result<Configuration, Error> {
        let config_builder = Config::builder()
            .add_source(File::from_str(&info.index_toml, FileFormat::Toml))
            .build()?;
        let mut index_config: TorrustIndex = config_builder.try_deserialize()?;

        if let Some(ref token) = info.tracker_api_token {
            index_config.override_tracker_api_token(token);
        };

        if let Some(ref secret_key) = info.auth_secret_key {
            index_config.override_auth_secret_key(secret_key);
        };

        Ok(Configuration {
            settings: RwLock::new(index_config),
            config_path: None,
        })
    }

    pub async fn get_all(&self) -> TorrustIndex {
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

    pub async fn get_api_base_url(&self) -> Option<String> {
        let settings_lock = self.settings.read().await;

        settings_lock.net.base_url.clone()
    }

    /// # Errors
    ///
    /// Will return an error if the configuration is invalid.
    pub async fn validate(&self) -> Result<(), ValidationError> {
        self.validate_tracker_config().await
    }

    /// # Errors
    ///
    /// Will return an error if the `tracker` configuration section is invalid.    
    pub async fn validate_tracker_config(&self) -> Result<(), ValidationError> {
        let settings_lock = self.settings.read().await;

        let tracker_mode = settings_lock.tracker.mode.clone();
        let tracker_url = settings_lock.tracker.url.clone();

        let tracker_url = match parse_url(&tracker_url) {
            Ok(url) => url,
            Err(err) => {
                return Err(ValidationError::InvalidTrackerUrl {
                    source: Located(err).into(),
                })
            }
        };

        if tracker_mode.is_close() && (tracker_url.scheme() != "http" && tracker_url.scheme() != "https") {
            return Err(ValidationError::UdpTrackersInPrivateModeNotSupported);
        }

        Ok(())
    }
}

fn parse_url(url_str: &str) -> Result<Url, url::ParseError> {
    Url::parse(url_str)
}

/// The public index configuration.
/// There is an endpoint to get this configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigurationPublic {
    website_name: String,
    tracker_url: String,
    tracker_mode: TrackerMode,
    email_on_signup: EmailOnSignup,
}

#[cfg(test)]
mod tests {

    use crate::config::{Configuration, ConfigurationPublic, Info};

    #[cfg(test)]
    fn default_config_toml() -> String {
        let config = r#"[website]
                                name = "Torrust"

                                [tracker]
                                url = "udp://localhost:6969"
                                mode = "Public"
                                api_url = "http://localhost:1212"
                                token = "MyAccessToken"
                                token_valid_seconds = 7257600

                                [net]
                                port = 3001

                                [auth]
                                email_on_signup = "Optional"
                                min_password_length = 6
                                max_password_length = 64
                                secret_key = "MaxVerstappenWC2021"

                                [database]
                                connect_url = "sqlite://data.db?mode=rwc"

                                [mail]
                                email_verification_enabled = false
                                from = "example@email.com"
                                reply_to = "noreply@email.com"
                                username = ""
                                password = ""
                                server = ""
                                port = 25

                                [image_cache]
                                max_request_timeout_ms = 1000
                                capacity = 128000000
                                entry_size_limit = 4000000
                                user_quota_period_seconds = 3600
                                user_quota_bytes = 64000000

                                [api]
                                default_torrent_page_size = 10
                                max_torrent_page_size = 30

                                [tracker_statistics_importer]
                                torrent_info_update_interval = 3600
                                port = 3002
        "#
        .lines()
        .map(str::trim_start)
        .collect::<Vec<&str>>()
        .join("\n");
        config
    }

    #[tokio::test]
    async fn configuration_should_build_settings_with_default_values() {
        let configuration = Configuration::default().get_all().await;

        let toml = toml::to_string(&configuration).expect("Could not encode TOML value for configuration");

        assert_eq!(toml, default_config_toml());
    }

    #[tokio::test]
    async fn configuration_should_return_all_settings() {
        let configuration = Configuration::default().get_all().await;

        let toml = toml::to_string(&configuration).expect("Could not encode TOML value for configuration");

        assert_eq!(toml, default_config_toml());
    }

    #[tokio::test]
    async fn configuration_should_return_only_public_settings() {
        let configuration = Configuration::default();
        let all_settings = configuration.get_all().await;

        assert_eq!(
            configuration.get_public().await,
            ConfigurationPublic {
                website_name: all_settings.website.name,
                tracker_url: all_settings.tracker.url,
                tracker_mode: all_settings.tracker.mode,
                email_on_signup: all_settings.auth.email_on_signup,
            }
        );
    }

    #[tokio::test]
    async fn configuration_should_return_the_site_name() {
        let configuration = Configuration::default();
        assert_eq!(configuration.get_site_name().await, "Torrust".to_string());
    }

    #[tokio::test]
    async fn configuration_should_return_the_api_base_url() {
        let configuration = Configuration::default();
        assert_eq!(configuration.get_api_base_url().await, None);

        let mut settings_lock = configuration.settings.write().await;
        settings_lock.net.base_url = Some("http://localhost".to_string());
        drop(settings_lock);

        assert_eq!(configuration.get_api_base_url().await, Some("http://localhost".to_string()));
    }

    #[tokio::test]
    async fn configuration_could_be_loaded_from_a_toml_string() {
        let info = Info {
            index_toml: default_config_toml(),
            tracker_api_token: None,
            auth_secret_key: None,
        };

        let configuration = Configuration::load(&info).expect("Failed to load configuration from info");

        assert_eq!(configuration.get_all().await, Configuration::default().get_all().await);
    }

    #[tokio::test]
    async fn configuration_should_allow_to_override_the_tracker_api_token_provided_in_the_toml_file() {
        let info = Info {
            index_toml: default_config_toml(),
            tracker_api_token: Some("OVERRIDDEN API TOKEN".to_string()),
            auth_secret_key: None,
        };

        let configuration = Configuration::load(&info).expect("Failed to load configuration from info");

        assert_eq!(
            configuration.get_all().await.tracker.token,
            "OVERRIDDEN API TOKEN".to_string()
        );
    }

    #[tokio::test]
    async fn configuration_should_allow_to_override_the_authentication_secret_key_provided_in_the_toml_file() {
        let info = Info {
            index_toml: default_config_toml(),
            tracker_api_token: None,
            auth_secret_key: Some("OVERRIDDEN AUTH SECRET KEY".to_string()),
        };

        let configuration = Configuration::load(&info).expect("Failed to load configuration from info");

        assert_eq!(
            configuration.get_all().await.auth.secret_key,
            "OVERRIDDEN AUTH SECRET KEY".to_string()
        );
    }

    mod syntax_checks {
        // todo: use rich types in configuration structs for basic syntax checks.

        use crate::config::Configuration;

        #[tokio::test]
        async fn tracker_url_should_be_a_valid_url() {
            let configuration = Configuration::default();

            let mut settings_lock = configuration.settings.write().await;
            settings_lock.tracker.url = "INVALID URL".to_string();
            drop(settings_lock);

            assert!(configuration.validate().await.is_err());
        }
    }

    mod semantic_validation {
        use crate::config::{Configuration, TrackerMode};

        #[tokio::test]
        async fn udp_trackers_in_close_mode_are_not_supported() {
            let configuration = Configuration::default();

            let mut settings_lock = configuration.settings.write().await;
            settings_lock.tracker.mode = TrackerMode::Private;
            settings_lock.tracker.url = "udp://localhost:6969".to_string();
            drop(settings_lock);

            assert!(configuration.validate().await.is_err());
        }
    }
}
