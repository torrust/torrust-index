//! Configuration for the application.
pub mod v1;
pub mod validator;

use std::env;
use std::str::FromStr;
use std::sync::Arc;

use camino::Utf8PathBuf;
use figment::providers::{Env, Format, Serialized, Toml};
use figment::Figment;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};
use thiserror::Error;
use tokio::sync::RwLock;
use torrust_index_located_error::LocatedError;
use url::Url;

use crate::web::api::server::DynError;

pub type Settings = v1::Settings;
pub type Api = v1::api::Api;
pub type Auth = v1::auth::Auth;
pub type Database = v1::database::Database;
pub type ImageCache = v1::image_cache::ImageCache;
pub type Mail = v1::mail::Mail;
pub type Network = v1::net::Network;
pub type TrackerStatisticsImporter = v1::tracker_statistics_importer::TrackerStatisticsImporter;
pub type Tracker = v1::tracker::Tracker;
pub type Website = v1::website::Website;
pub type EmailOnSignup = v1::auth::EmailOnSignup;

/// Prefix for env vars that overwrite configuration options.
const CONFIG_OVERRIDE_PREFIX: &str = "TORRUST_INDEX_CONFIG_OVERRIDE_";

/// Path separator in env var names for nested values in configuration.
const CONFIG_OVERRIDE_SEPARATOR: &str = "__";

/// The whole `index.toml` file content. It has priority over the config file.
/// Even if the file is not on the default path.
pub const ENV_VAR_CONFIG_TOML: &str = "TORRUST_INDEX_CONFIG_TOML";

/// The `index.toml` file location.
pub const ENV_VAR_CONFIG_TOML_PATH: &str = "TORRUST_INDEX_CONFIG_TOML_PATH";

/// Information required for loading config
#[derive(Debug, Default, Clone)]
pub struct Info {
    config_toml: Option<String>,
    config_toml_path: String,
}

impl Info {
    /// Build configuration Info.
    ///
    /// # Errors
    ///
    /// Will return `Err` if unable to obtain a configuration.
    ///
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(default_config_toml_path: String) -> Result<Self, Error> {
        let env_var_config_toml = ENV_VAR_CONFIG_TOML.to_string();
        let env_var_config_toml_path = ENV_VAR_CONFIG_TOML_PATH.to_string();

        let config_toml = if let Ok(config_toml) = env::var(env_var_config_toml) {
            println!("Loading configuration from environment variable {config_toml} ...");
            Some(config_toml)
        } else {
            None
        };

        let config_toml_path = if let Ok(config_toml_path) = env::var(env_var_config_toml_path) {
            println!("Loading configuration from file: `{config_toml_path}` ...");
            config_toml_path
        } else {
            println!("Loading configuration from default configuration file: `{default_config_toml_path}` ...");
            default_config_toml_path
        };

        Ok(Self {
            config_toml,
            config_toml_path,
        })
    }
}

/// Errors that can occur when loading the configuration.
#[derive(Error, Debug)]
pub enum Error {
    /// Unable to load the configuration from the environment variable.
    /// This error only occurs if there is no configuration file and the
    /// `TORRUST_INDEX_CONFIG_TOML` environment variable is not set.
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
    ConfigError {
        source: LocatedError<'static, dyn std::error::Error + Send + Sync>,
    },

    #[error("The error for errors that can never happen.")]
    Infallible,
}

impl From<figment::Error> for Error {
    #[track_caller]
    fn from(err: figment::Error) -> Self {
        Self::ConfigError {
            source: (Arc::new(err) as DynError).into(),
        }
    }
}

/* todo:

Use https://crates.io/crates/torrust-tracker-primitives for TrackerMode.

Enum variants:

  In Index                In Tracker
- `Public`             -> `Public`
- `Private`            -> `Private`
- `Whitelisted`        -> `Listed`
- `PrivateWhitelisted` -> `PrivateListed`

Enum serialized values:

  In Index                In Tracker
- `Public`             -> `public`
- `Private`            -> `private`
- `Whitelisted`        -> `listed`
- `PrivateWhitelisted` -> `private_listed`

It's a breaking change for the toml config file en the API.

*/

/// See `TrackerMode` in [`torrust-tracker-primitives`](https://docs.rs/torrust-tracker-primitives)
/// crate for more information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrackerMode {
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

impl FromStr for TrackerMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Public" => Ok(TrackerMode::Public),
            "Private" => Ok(TrackerMode::Private),
            "Whitelisted" => Ok(TrackerMode::Whitelisted),
            "PrivateWhitelisted" => Ok(TrackerMode::PrivateWhitelisted),
            _ => Err(format!(
                "{s} is not a valid tracker mode. Valid values: 'Public', 'Private', 'Whitelisted', 'PrivateWhitelisted' "
            )),
        }
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

/// Port number representing that the OS will choose one randomly from the available ports.
///
/// It's the port number `0`
pub const FREE_PORT: u16 = 0;

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

/// The configuration service.
#[derive(Debug)]
pub struct Configuration {
    /// The state of the configuration.
    pub settings: RwLock<Settings>,
}

impl Default for Configuration {
    fn default() -> Configuration {
        Configuration {
            settings: RwLock::new(Settings::default()),
        }
    }
}

impl Configuration {
    /// Loads the configuration from the `Info` struct.
    ///
    /// # Errors
    ///
    /// Will return `Err` if the environment variable does not exist or has a bad configuration.
    pub fn load(info: &Info) -> Result<Configuration, Error> {
        let settings = Self::load_settings(info)?;

        Ok(Configuration {
            settings: RwLock::new(settings),
        })
    }

    /// Loads the settings from the `Info` struct. The whole
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
    pub fn load_settings(info: &Info) -> Result<Settings, Error> {
        let figment = if let Some(config_toml) = &info.config_toml {
            // Config in env var has priority over config file path
            Figment::from(Serialized::defaults(Settings::default()))
                .merge(Toml::string(config_toml))
                .merge(Env::prefixed(CONFIG_OVERRIDE_PREFIX).split(CONFIG_OVERRIDE_SEPARATOR))
        } else {
            Figment::from(Serialized::defaults(Settings::default()))
                .merge(Toml::file(&info.config_toml_path))
                .merge(Env::prefixed(CONFIG_OVERRIDE_PREFIX).split(CONFIG_OVERRIDE_SEPARATOR))
        };

        let settings: Settings = figment.extract()?;

        Ok(settings)
    }

    pub async fn get_all(&self) -> Settings {
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
        settings_lock.net.base_url.as_ref().map(std::string::ToString::to_string)
    }
}

/// The public index configuration.
/// There is an endpoint to get this configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigurationPublic {
    website_name: String,
    tracker_url: Url,
    tracker_mode: TrackerMode,
    email_on_signup: EmailOnSignup,
}

#[cfg(test)]
mod tests {

    use url::Url;

    use crate::config::v1::auth::SecretKey;
    use crate::config::v1::tracker::ApiToken;
    use crate::config::{Configuration, ConfigurationPublic, Info, Settings};

    #[cfg(test)]
    fn default_config_toml() -> String {
        let config = r#"[website]
                                name = "Torrust"

                                [tracker]
                                url = "udp://localhost:6969"
                                mode = "Public"
                                api_url = "http://localhost:1212/"
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
        settings_lock.net.base_url = Some(Url::parse("http://localhost").unwrap());
        drop(settings_lock);

        assert_eq!(configuration.get_api_base_url().await, Some("http://localhost/".to_string()));
    }

    #[tokio::test]
    async fn configuration_could_be_loaded_from_a_toml_string() {
        figment::Jail::expect_with(|jail| {
            jail.create_dir("templates")?;
            jail.create_file("templates/verify.html", "EMAIL TEMPLATE")?;

            let info = Info {
                config_toml: Some(default_config_toml()),
                config_toml_path: String::new(),
            };

            let settings = Configuration::load_settings(&info).expect("Failed to load configuration from info");

            assert_eq!(settings, Settings::default());

            Ok(())
        });
    }

    #[tokio::test]
    async fn configuration_should_allow_to_override_the_tracker_api_token_provided_in_the_toml_file() {
        figment::Jail::expect_with(|jail| {
            jail.create_dir("templates")?;
            jail.create_file("templates/verify.html", "EMAIL TEMPLATE")?;

            jail.set_env("TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN", "OVERRIDDEN API TOKEN");

            let info = Info {
                config_toml: Some(default_config_toml()),
                config_toml_path: String::new(),
            };

            let settings = Configuration::load_settings(&info).expect("Could not load configuration from file");

            assert_eq!(settings.tracker.token, ApiToken::new("OVERRIDDEN API TOKEN"));

            Ok(())
        });
    }

    #[tokio::test]
    async fn configuration_should_allow_to_override_the_authentication_secret_key_provided_in_the_toml_file() {
        figment::Jail::expect_with(|jail| {
            jail.create_dir("templates")?;
            jail.create_file("templates/verify.html", "EMAIL TEMPLATE")?;

            jail.set_env("TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__SECRET_KEY", "OVERRIDDEN AUTH SECRET KEY");

            let info = Info {
                config_toml: Some(default_config_toml()),
                config_toml_path: String::new(),
            };

            let settings = Configuration::load_settings(&info).expect("Could not load configuration from file");

            assert_eq!(settings.auth.secret_key, SecretKey::new("OVERRIDDEN AUTH SECRET KEY"));

            Ok(())
        });
    }

    mod semantic_validation {
        use url::Url;

        use crate::config::validator::Validator;
        use crate::config::{Configuration, TrackerMode};

        #[tokio::test]
        async fn udp_trackers_in_close_mode_are_not_supported() {
            let configuration = Configuration::default();

            let mut settings_lock = configuration.settings.write().await;
            settings_lock.tracker.mode = TrackerMode::Private;
            settings_lock.tracker.url = Url::parse("udp://localhost:6969").unwrap();

            assert!(settings_lock.validate().is_err());
        }
    }
}
