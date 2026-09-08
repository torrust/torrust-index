//! Configuration schema and loader for the Torrust Index.
//!
//! This crate owns the *parsing* surface of the index's
//! configuration. The runtime `Configuration` wrapper that holds
//! settings under a `tokio::sync::RwLock` lives in the root crate
//! alongside the application.
//!
//! Extracted from `src/config/` per ADR-T-009 phase 3.
pub mod permissions;
pub mod v2;
pub mod validator;

#[doc(hidden)]
pub mod test_helpers;

#[cfg(test)]
mod tests;

use std::env;
use std::sync::Arc;

use camino::Utf8PathBuf;
use derive_more::Display;
use figment::Figment;
use figment::providers::{Env, Format, Toml};
use serde::{Deserialize, Serialize};
use serde_with::{NoneAsEmptyString, serde_as};
use thiserror::Error;

/// Type-erased boxed error used by the configuration loader's
/// error variants.
///
/// Re-exported by the root crate as `crate::web::api::server::DynError`
/// for backwards compatibility.
pub type DynError = Arc<dyn std::error::Error + Send + Sync>;

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

/// Prefix for env vars that overwrite configuration options.
const CONFIG_OVERRIDE_PREFIX: &str = "TORRUST_INDEX_CONFIG_OVERRIDE_";

/// Path separator in env var names for nested values in configuration.
const CONFIG_OVERRIDE_SEPARATOR: &str = "__";

/// The whole `index.toml` file content. It has priority over the config file.
/// Even if the file is not on the default path.
pub const ENV_VAR_CONFIG_TOML: &str = "TORRUST_INDEX_CONFIG_TOML";

/// The `index.toml` file location.
pub const ENV_VAR_CONFIG_TOML_PATH: &str = "TORRUST_INDEX_CONFIG_TOML_PATH";

/// Default path for the configuration TOML when neither
/// [`ENV_VAR_CONFIG_TOML`] nor [`ENV_VAR_CONFIG_TOML_PATH`] is set.
///
/// Both the application bootstrap and helper binaries (e.g.
/// `torrust-index-config-probe`) refer to this constant so the
/// default cannot drift between call sites.
pub const DEFAULT_CONFIG_TOML_PATH: &str = "./share/default/config/index.development.sqlite3.toml";

/// The latest (and currently only) supported configuration schema version.
///
/// `load_settings` rejects any parsed `Settings` whose
/// `metadata.schema_version` is not equal to this constant.
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
    pub config_toml: Option<String>,
    pub config_toml_path: String,
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
        let info = Self::from_env(&default_config_toml_path);

        if info.config_toml.is_some() {
            // The TOML body may contain secrets (DB connect URLs, API
            // tokens, SMTP passwords, …) so log only the env-var name
            // — never its value — and route through `tracing` (stderr)
            // so we don't pollute the JSON-only stdout contract used
            // by helper binaries (ADR-T-010).
            tracing::info!(
                env_var = ENV_VAR_CONFIG_TOML,
                "loading extra configuration from environment variable"
            );
        }

        if env::var(ENV_VAR_CONFIG_TOML_PATH).is_ok_and(|s| !s.is_empty()) {
            tracing::info!(path = %info.config_toml_path, "loading extra configuration from file");
        } else {
            tracing::info!(
                path = %info.config_toml_path,
                "loading extra configuration from default configuration file"
            );
        }

        Ok(info)
    }

    /// Build [`Info`] from the same env vars [`Self::new`] reads,
    /// without the diagnostic `println!`s.
    ///
    /// Helper binaries that own a JSON-only stdout contract (ADR-T-010)
    /// must use this constructor instead of [`Self::new`] to avoid
    /// corrupting their output stream.
    #[must_use]
    pub fn from_env(default_config_toml_path: &str) -> Self {
        // Treat an empty value as unset so callers (e.g. `docker compose`/`podman-compose`)
        // can safely forward `KEY=${HOST_VAR}` without clobbering the file-based config
        // when `HOST_VAR` is not exported.
        let config_toml = env::var(ENV_VAR_CONFIG_TOML).ok().filter(|s| !s.is_empty());
        let config_toml_path = env::var(ENV_VAR_CONFIG_TOML_PATH)
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| default_config_toml_path.to_string());

        Self {
            config_toml,
            config_toml_path,
        }
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

/// TLS configuration for the HTTPS endpoint.
///
/// Both fields default to `None` (treated as "not configured") when
/// absent **or** when given as an empty string. An incomplete
/// `[net.tls]` table — for example one missing `ssl_key_path` — therefore
/// behaves identically to `[net.tls]` being absent altogether, rather
/// than being silently coerced into a `Some("")` that would later fail
/// at TLS load time with a misleading "no such file" error.
#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct Tls {
    /// Path to the SSL certificate file.
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub ssl_cert_path: Option<Utf8PathBuf>,
    /// Path to the SSL key file.
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub ssl_key_path: Option<Utf8PathBuf>,
}

/// Loads the settings from the [`Info`] struct.
///
/// The whole configuration in TOML format is included in
/// `info.config_toml` (and overrides the file path when set).
/// Configuration provided via env var takes priority over the
/// configuration file path.
///
/// # Errors
///
/// Will return `Err` if a mandatory option is missing, the schema
/// version is not supported, or the underlying figment extraction
/// fails.
pub fn load_settings(info: &Info) -> Result<Settings, Error> {
    // Load configuration provided by the user, prioritizing env vars
    let figment = info.config_toml.as_ref().map_or_else(
        || {
            Figment::from(Toml::file(&info.config_toml_path))
                .merge(Env::prefixed(CONFIG_OVERRIDE_PREFIX).split(CONFIG_OVERRIDE_SEPARATOR))
        },
        |config_toml| {
            // Config in env var has priority over config file path
            Figment::from(Toml::string(config_toml)).merge(Env::prefixed(CONFIG_OVERRIDE_PREFIX).split(CONFIG_OVERRIDE_SEPARATOR))
        },
    );

    // Make sure user has provided the mandatory options.
    check_mandatory_options(&figment)?;

    // Build final configuration. Per-field `#[serde(default = "...")]`
    // attributes fill in defaults for absent optional sections; the
    // mandatory `tracker.token` and `database.connect_url` (and the
    // enclosing `[tracker]` / `[database]` sections themselves) carry
    // no schema-level default, so an absent value fails here with a
    // precise serde `missing field` error (ADR-T-009 §D2).
    let settings: Settings = figment.extract()?;

    if settings.metadata.schema_version != Version::new(LATEST_VERSION) {
        return Err(Error::UnsupportedVersion {
            version: settings.metadata.schema_version,
        });
    }

    Ok(settings)
}

/// Some configuration options are mandatory. The application will
/// refuse to start if any is only obtained via the in-code default.
///
/// # Errors
///
/// Will return an error if a mandatory configuration option is only
/// obtained by default value, meaning the user hasn't overridden it.
fn check_mandatory_options(figment: &Figment) -> Result<(), Error> {
    let mandatory_options = ["logging.threshold", "metadata.schema_version"];

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
