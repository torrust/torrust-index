//! Configuration for the application.
pub mod v2;
pub mod validator;

use std::env;
use std::sync::Arc;

use camino::Utf8PathBuf;
use derive_more::Display;
use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use serde::{Deserialize, Serialize};
use serde_with::{NoneAsEmptyString, serde_as};
use thiserror::Error;
use tokio::sync::RwLock;

use crate::web::api::server::DynError;

pub type Settings = v2::Settings;

pub type Api = v2::api::Api;

pub type Registration = v2::registration::Registration;
pub type Email = v2::registration::Email;

pub type Auth = v2::auth::Auth;
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
pub type Demo = v2::website::Demo;
pub type Terms = v2::website::Terms;
pub type TermsPage = v2::website::TermsPage;
pub type TermsUpload = v2::website::TermsUpload;
pub type Markdown = v2::website::Markdown;

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
#[display("Metadata(app: {app}, purpose: {purpose}, schema_version: {schema_version})")]
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
    const fn default_app() -> App {
        App::TorrustIndex
    }

    const fn default_purpose() -> Purpose {
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
    pub(crate) config_toml: Option<String>,
    pub(crate) config_toml_path: String,
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

        let config_toml = env::var(env_var_config_toml).ok().map(|config_toml| {
            println!("Loading extra configuration from environment variable {config_toml} ...");
            config_toml
        });

        let config_toml_path = env::var(env_var_config_toml_path).map_or_else(
            |_| {
                println!("Loading extra configuration from default configuration file: `{default_config_toml_path}` ...");
                default_config_toml_path
            },
            |config_toml_path| {
                println!("Loading extra configuration from file: `{config_toml_path}` ...");
                config_toml_path
            },
        );

        Ok(Self {
            config_toml,
            config_toml_path,
        })
    }

    #[must_use]
    pub fn from_toml(config_toml: &str) -> Self {
        Self {
            config_toml: Some(config_toml.to_owned()),
            config_toml_path: String::new(),
        }
    }
}

/// Errors that can occur when loading the configuration.
#[derive(Error, Debug)]
pub enum Error {
    /// Unable to load the configuration from the environment variable.
    /// This error only occurs if there is no configuration file and the
    /// `TORRUST_INDEX_CONFIG_TOML` environment variable is not set.
    #[error("Unable to load from Environmental Variable: {source}")]
    UnableToLoadFromEnvironmentVariable { source: DynError },

    #[error("Unable to load from Config File: {source}")]
    UnableToLoadFromConfigFile { source: DynError },

    /// Unable to load the configuration from the configuration file.
    #[error("Failed processing the configuration: {source}")]
    ConfigError { source: DynError },

    #[error("The error for errors that can never happen.")]
    Infallible,

    #[error("Unsupported configuration version: {version}")]
    UnsupportedVersion { version: Version },

    #[error("Missing mandatory configuration option. Option path: {path}")]
    MissingMandatoryOption { path: String },
}

impl From<figment::Error> for Error {
    fn from(err: figment::Error) -> Self {
        tracing::error!(%err, "Failed processing the configuration");
        Self::ConfigError { source: Arc::new(err) }
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
    fn default() -> Self {
        Self {
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
    pub fn load(info: &Info) -> Result<Self, Error> {
        let settings = Self::load_settings(info)?;

        Ok(Self {
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
        let figment = info.config_toml.as_ref().map_or_else(
            || {
                Figment::from(Toml::file(&info.config_toml_path))
                    .merge(Env::prefixed(CONFIG_OVERRIDE_PREFIX).split(CONFIG_OVERRIDE_SEPARATOR))
            },
            |config_toml| {
                // Config in env var has priority over config file path
                Figment::from(Toml::string(config_toml))
                    .merge(Env::prefixed(CONFIG_OVERRIDE_PREFIX).split(CONFIG_OVERRIDE_SEPARATOR))
            },
        );

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
        let mandatory_options = ["logging.threshold", "metadata.schema_version", "tracker.token"];

        for mandatory_option in mandatory_options {
            let found = figment.find_value(mandatory_option).is_ok();

            if !found {
                return Err(Error::MissingMandatoryOption {
                    path: mandatory_option.to_owned(),
                });
            }
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
