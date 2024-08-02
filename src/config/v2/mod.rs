pub mod api;
pub mod auth;
pub mod database;
pub mod image_cache;
pub mod logging;
pub mod mail;
pub mod net;
pub mod registration;
pub mod tracker;
pub mod tracker_statistics_importer;
pub mod website;

use logging::Logging;
use registration::Registration;
use serde::{Deserialize, Serialize};

use self::api::Api;
use self::auth::{Auth, ClaimTokenPepper};
use self::database::Database;
use self::image_cache::ImageCache;
use self::mail::Mail;
use self::net::Network;
use self::tracker::{ApiToken, Tracker};
use self::tracker_statistics_importer::TrackerStatisticsImporter;
use self::website::Website;
use super::validator::{ValidationError, Validator};
use super::Metadata;

/// The whole configuration for the index.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    /// Configuration metadata.
    #[serde(default = "Settings::default_metadata")]
    pub metadata: Metadata,

    /// The logging configuration.
    #[serde(default = "Settings::default_logging")]
    pub logging: Logging,

    /// The website customizable values.
    #[serde(default = "Settings::default_website")]
    pub website: Website,

    /// The tracker configuration.
    #[serde(default = "Settings::default_tracker")]
    pub tracker: Tracker,

    /// The network configuration.
    #[serde(default = "Settings::default_network")]
    pub net: Network,

    /// The authentication configuration.
    #[serde(default = "Settings::default_auth")]
    pub auth: Auth,

    /// The database configuration.
    #[serde(default = "Settings::default_database")]
    pub database: Database,

    /// The SMTP configuration.
    #[serde(default = "Settings::default_mail")]
    pub mail: Mail,

    /// The image proxy cache configuration.
    #[serde(default = "Settings::default_image_cache")]
    pub image_cache: ImageCache,

    /// The API configuration.
    #[serde(default = "Settings::default_api")]
    pub api: Api,

    /// The registration configuration.
    #[serde(default = "Settings::default_registration")]
    pub registration: Option<Registration>,

    /// The tracker statistics importer job configuration.
    #[serde(default = "Settings::default_tracker_statistics_importer")]
    pub tracker_statistics_importer: TrackerStatisticsImporter,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            metadata: Self::default_metadata(),
            logging: Self::default_logging(),
            website: Self::default_website(),
            tracker: Self::default_tracker(),
            net: Self::default_network(),
            auth: Self::default_auth(),
            database: Self::default_database(),
            mail: Self::default_mail(),
            image_cache: Self::default_image_cache(),
            api: Self::default_api(),
            registration: Self::default_registration(),
            tracker_statistics_importer: Self::default_tracker_statistics_importer(),
        }
    }
}

impl Settings {
    pub fn remove_secrets(&mut self) {
        self.tracker.token = ApiToken::new("***");
        if let Some(_password) = self.database.connect_url.password() {
            let _ = self.database.connect_url.set_password(Some("***"));
        }
        "***".clone_into(&mut self.mail.smtp.credentials.password);
        self.auth.user_claim_token_pepper = ClaimTokenPepper::new("***");
    }

    /// Encodes the configuration to TOML.
    ///
    /// # Panics
    ///
    /// Will panic if it can't be converted to TOML.
    #[must_use]
    pub fn to_toml(&self) -> String {
        toml::to_string(self).expect("Could not encode TOML value")
    }

    /// Encodes the configuration to JSON.
    ///
    /// # Panics
    ///
    /// Will panic if it can't be converted to JSON.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("Could not encode JSON value")
    }

    fn default_metadata() -> Metadata {
        Metadata::default()
    }

    fn default_logging() -> Logging {
        Logging::default()
    }

    fn default_website() -> Website {
        Website::default()
    }

    fn default_tracker() -> Tracker {
        Tracker::default()
    }

    fn default_network() -> Network {
        Network::default()
    }

    fn default_auth() -> Auth {
        Auth::default()
    }

    fn default_database() -> Database {
        Database::default()
    }

    fn default_mail() -> Mail {
        Mail::default()
    }

    fn default_image_cache() -> ImageCache {
        ImageCache::default()
    }

    fn default_api() -> Api {
        Api::default()
    }

    fn default_registration() -> Option<Registration> {
        None
    }

    fn default_tracker_statistics_importer() -> TrackerStatisticsImporter {
        TrackerStatisticsImporter::default()
    }
}

impl Validator for Settings {
    fn validate(&self) -> Result<(), ValidationError> {
        self.tracker.validate()
    }
}
