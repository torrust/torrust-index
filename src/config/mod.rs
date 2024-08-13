//! Configuration for the application.
pub mod v2;
pub mod validator;

use std::env;
use std::sync::Arc;

use camino::Utf8PathBuf;
use derive_more::Display;
use figment::providers::{Env, Format, Serialized, Toml};
use figment::Figment;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};
use thiserror::Error;
use tokio::sync::RwLock;
use torrust_index_located_error::LocatedError;

use crate::web::api::server::DynError;

pub type Settings = v2::Settings;

pub type Api = v2::api::Api;

pub type Registration = v2::registration::Registration;
pub type Email = v2::registration::Email;

pub type Auth = v2::auth::Auth;
pub type SecretKey = v2::auth::ClaimTokenPepper;
pub type PasswordConstraints = v2::auth::PasswordConstraints;

pub type Database = v2::database::Database;

pub type ImageCache = v2::image_cache::ImageCache;

pub type Mail = v2::mail::Mail;
pub type Smtp = v2::mail::Smtp;
pub type Credentials = v2::mail::Credentials;

pub type Network = v2::net::Network;

pub type TrackerStatisticsImporter = v2::tracker_statistics_importer::TrackerStatisticsImporter;

pub type Tracker = v2::tracker::Tracker;
pub type ApiToken = v2::tracker::ApiToken;

pub type Logging = v2::logging::Logging;
pub type Threshold = v2::logging::Threshold;

pub type Website = v2::website::Website;

/// Configuration version
const VERSION_2: &str = "2.0.0";

/// Prefix for env vars that overwrite configuration options.
const CONFIG_OVERRIDE_PREFIX: &str = "TORRUST_INDEX_CONFIG_OVERRIDE_";

/// Path separator in env var names for nested values in configuration.
const CONFIG_OVERRIDE_SEPARATOR: &str = "__";

/// The whole `index.toml` file content. It has priority over the config file.
/// Even if the file is not on the default path.
pub const ENV_VAR_CONFIG_TOML: &str = "TORRUST_INDEX_CONFIG_TOML";

/// The `index.toml` file location.
pub const ENV_VAR_CONFIG_TOML_PATH: &str = "TORRUST_INDEX_CONFIG_TOML_PATH";

pub const LATEST_VERSION: &str = "2.0.0";

/// Info about the configuration specification.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Display, Clone)]
#[display(fmt = "Metadata(app: {app}, purpose: {purpose}, schema_version: {schema_version})")]
pub struct Metadata {
    /// The application this configuration is valid for.
    #[serde(default = "Metadata::default_app")]
    app: App,

    /// The purpose of this parsed file.
    #[serde(default = "Metadata::default_purpose")]
    purpose: Purpose,

    /// The schema version for the configuration.
    #[serde(default = "Metadata::default_schema_version")]
    #[serde(flatten)]
    schema_version: Version,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            app: Self::default_app(),
            purpose: Self::default_purpose(),
            schema_version: Self::default_schema_version(),
        }
    }
}

impl Metadata {
    fn default_app() -> App {
        App::TorrustIndex
    }

    fn default_purpose() -> Purpose {
        Purpose::Configuration
    }

    fn default_schema_version() -> Version {
        Version::latest()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Display, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum App {
    TorrustIndex,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Display, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Purpose {
    Configuration,
}

/// The configuration version.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Display, Clone)]
#[serde(rename_all = "lowercase")]
pub struct Version {
    #[serde(default = "Version::default_semver")]
    schema_version: String,
}

impl Default for Version {
    fn default() -> Self {
        Self {
            schema_version: Self::default_semver(),
        }
    }
}

impl Version {
    fn new(semver: &str) -> Self {
        Self {
            schema_version: semver.to_owned(),
        }
    }

    fn latest() -> Self {
        Self {
            schema_version: LATEST_VERSION.to_string(),
        }
    }

    fn default_semver() -> String {
        LATEST_VERSION.to_string()
    }
}

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
            println!("Loading extra configuration from environment variable {config_toml} ...");
            Some(config_toml)
        } else {
            None
        };

        let config_toml_path = if let Ok(config_toml_path) = env::var(env_var_config_toml_path) {
            println!("Loading extra configuration from file: `{config_toml_path}` ...");
            config_toml_path
        } else {
            println!("Loading extra configuration from default configuration file: `{default_config_toml_path}` ...");
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

    #[error("Unsupported configuration version: {version}")]
    UnsupportedVersion { version: Version },

    #[error("Missing mandatory configuration option. Option path: {path}")]
    MissingMandatoryOption { path: String },
}

impl From<figment::Error> for Error {
    #[track_caller]
    fn from(err: figment::Error) -> Self {
        Self::ConfigError {
            source: (Arc::new(err) as DynError).into(),
        }
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
    /// Configuration provided via env var has priority over config file path.
    ///
    /// # Errors
    ///
    /// Will return `Err` if the environment variable does not exist or has a bad configuration.
    pub fn load_settings(info: &Info) -> Result<Settings, Error> {
        // Load configuration provided by the user, prioritizing env vars
        let figment = if let Some(config_toml) = &info.config_toml {
            // Config in env var has priority over config file path
            Figment::from(Toml::string(config_toml)).merge(Env::prefixed(CONFIG_OVERRIDE_PREFIX).split(CONFIG_OVERRIDE_SEPARATOR))
        } else {
            Figment::from(Toml::file(&info.config_toml_path))
                .merge(Env::prefixed(CONFIG_OVERRIDE_PREFIX).split(CONFIG_OVERRIDE_SEPARATOR))
        };

        // Make sure user has provided the mandatory options.
        Self::check_mandatory_options(&figment)?;

        // Fill missing options with default values.
        let figment = figment.join(Serialized::defaults(Settings::default()));

        // Build final configuration.
        let settings: Settings = figment.extract()?;

        if settings.metadata.schema_version != Version::new(VERSION_2) {
            return Err(Error::UnsupportedVersion {
                version: settings.metadata.schema_version,
            });
        }

        Ok(settings)
    }

    /// Some configuration options are mandatory. The tracker will panic if
    /// the user doesn't provide an explicit value for them from one of the
    /// configuration sources: TOML or ENV VARS.
    ///
    /// # Errors
    ///
    /// Will return an error if a mandatory configuration option is only
    /// obtained by default value (code), meaning the user hasn't overridden it.
    fn check_mandatory_options(figment: &Figment) -> Result<(), Error> {
        let mandatory_options = [
            "auth.user_claim_token_pepper",
            "logging.threshold",
            "metadata.schema_version",
            "tracker.token",
        ];

        for mandatory_option in mandatory_options {
            figment
                .find_value(mandatory_option)
                .map_err(|_err| Error::MissingMandatoryOption {
                    path: mandatory_option.to_owned(),
                })?;
        }

        Ok(())
    }

    pub async fn get_all(&self) -> Settings {
        let settings_lock = self.settings.read().await;

        settings_lock.clone()
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

#[cfg(test)]
mod tests {

    use url::Url;

    use crate::config::{ApiToken, Configuration, Info, SecretKey, Settings};

    #[cfg(test)]
    fn default_config_toml() -> String {
        let config = r#"[metadata]
                                app = "torrust-index"
                                purpose = "configuration"
                                schema_version = "2.0.0"

                                [logging]
                                threshold = "info"

                                [website]
                                name = "Torrust"

                                [tracker]
                                api_url = "http://localhost:1212/"
                                listed = false
                                private = false
                                token = "MyAccessToken"
                                token_valid_seconds = 7257600
                                url = "udp://localhost:6969"

                                [net]
                                bind_address = "0.0.0.0:3001"

                                [auth]
                                user_claim_token_pepper = "MaxVerstappenWC2021"

                                [auth.password_constraints]
                                max_password_length = 64
                                min_password_length = 6

                                [database]
                                connect_url = "sqlite://data.db?mode=rwc"

                                [mail]
                                from = "example@email.com"
                                reply_to = "noreply@email.com"

                                [mail.smtp]
                                port = 25
                                server = ""

                                [mail.smtp.credentials]
                                password = ""
                                username = ""

                                [image_cache]
                                capacity = 128000000
                                entry_size_limit = 4000000
                                max_request_timeout_ms = 1000
                                user_quota_bytes = 64000000
                                user_quota_period_seconds = 3600

                                [api]
                                default_torrent_page_size = 10
                                max_torrent_page_size = 30

                                [tracker_statistics_importer]
                                port = 3002
                                torrent_info_update_interval = 3600
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

    #[test]
    fn configuration_should_use_the_default_values_when_only_the_mandatory_options_are_provided_by_the_user_via_toml_file() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "index.toml",
                r#"
                [metadata]
                schema_version = "2.0.0"

                [logging]
                threshold = "info"

                [tracker]
                token = "MyAccessToken"

                [auth]
                user_claim_token_pepper = "MaxVerstappenWC2021"
            "#,
            )?;

            let info = Info {
                config_toml: None,
                config_toml_path: "index.toml".to_string(),
            };

            let settings = Configuration::load_settings(&info).expect("Could not load configuration from file");

            assert_eq!(settings, Settings::default());

            Ok(())
        });
    }

    #[test]
    fn configuration_should_use_the_default_values_when_only_the_mandatory_options_are_provided_by_the_user_via_toml_content() {
        figment::Jail::expect_with(|_jail| {
            let config_toml = r#"
                [metadata]
                schema_version = "2.0.0"

                [logging]
                threshold = "info"

                [tracker]
                token = "MyAccessToken"

                [auth]
                user_claim_token_pepper = "MaxVerstappenWC2021"
            "#
            .to_string();

            let info = Info {
                config_toml: Some(config_toml),
                config_toml_path: String::new(),
            };

            let settings = Configuration::load_settings(&info).expect("Could not load configuration from file");

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
    async fn configuration_should_allow_to_override_the_authentication_user_claim_token_pepper_provided_in_the_toml_file() {
        figment::Jail::expect_with(|jail| {
            jail.create_dir("templates")?;
            jail.create_file("templates/verify.html", "EMAIL TEMPLATE")?;

            jail.set_env(
                "TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__USER_CLAIM_TOKEN_PEPPER",
                "OVERRIDDEN AUTH SECRET KEY",
            );

            let info = Info {
                config_toml: Some(default_config_toml()),
                config_toml_path: String::new(),
            };

            let settings = Configuration::load_settings(&info).expect("Could not load configuration from file");

            assert_eq!(
                settings.auth.user_claim_token_pepper,
                SecretKey::new("OVERRIDDEN AUTH SECRET KEY")
            );

            Ok(())
        });
    }

    mod semantic_validation {
        use url::Url;

        use crate::config::validator::Validator;
        use crate::config::Configuration;

        #[tokio::test]
        async fn udp_trackers_in_private_mode_are_not_supported() {
            let configuration = Configuration::default();

            let mut settings_lock = configuration.settings.write().await;
            settings_lock.tracker.private = true;
            settings_lock.tracker.url = Url::parse("udp://localhost:6969").unwrap();

            assert!(settings_lock.validate().is_err());
        }
    }
}
