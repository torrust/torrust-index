pub mod responses;

use serde::{Deserialize, Serialize};
use torrust_index::config::{
    Api as DomainApi, ApiToken, Auth as DomainAuth, Credentials as DomainCredentials, Database as DomainDatabase,
    ImageCache as DomainImageCache, Logging as DomainLogging, Mail as DomainMail, Network as DomainNetwork,
    PasswordConstraints as DomainPasswordConstraints, Settings as DomainSettings, Smtp as DomainSmtp, Tracker as DomainTracker,
    TrackerStatisticsImporter as DomainTrackerStatisticsImporter, Website as DomainWebsite,
};
use url::Url;

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Settings {
    pub logging: Logging,
    pub website: Website,
    pub tracker: Tracker,
    pub net: Network,
    pub auth: Auth,
    pub database: Database,
    pub mail: Mail,
    pub image_cache: ImageCache,
    pub api: Api,
    pub tracker_statistics_importer: TrackerStatisticsImporter,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Logging {
    pub log_level: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Website {
    pub name: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Tracker {
    pub url: Url,
    pub mode: String,
    pub api_url: Url,
    pub token: ApiToken,
    pub token_valid_seconds: u64,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Network {
    pub port: u16,
    pub base_url: Option<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Auth {
    pub email_on_signup: String,
    pub secret_key: String,
    pub password_constraints: PasswordConstraints,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct PasswordConstraints {
    pub min_password_length: usize,
    pub max_password_length: usize,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Database {
    pub connect_url: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Mail {
    pub email_verification_enabled: bool,
    pub from: String,
    pub reply_to: String,
    pub smtp: Smtp,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Smtp {
    pub server: String,
    pub port: u16,
    pub credentials: Credentials,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct ImageCache {
    pub max_request_timeout_ms: u64,
    pub capacity: usize,
    pub entry_size_limit: usize,
    pub user_quota_period_seconds: u64,
    pub user_quota_bytes: usize,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Api {
    pub default_torrent_page_size: u8,
    pub max_torrent_page_size: u8,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct TrackerStatisticsImporter {
    pub torrent_info_update_interval: u64,
    port: u16,
}

impl From<DomainSettings> for Settings {
    fn from(settings: DomainSettings) -> Self {
        Settings {
            logging: Logging::from(settings.logging),
            website: Website::from(settings.website),
            tracker: Tracker::from(settings.tracker),
            net: Network::from(settings.net),
            auth: Auth::from(settings.auth),
            database: Database::from(settings.database),
            mail: Mail::from(settings.mail),
            image_cache: ImageCache::from(settings.image_cache),
            api: Api::from(settings.api),
            tracker_statistics_importer: TrackerStatisticsImporter::from(settings.tracker_statistics_importer),
        }
    }
}

impl From<DomainLogging> for Logging {
    fn from(logging: DomainLogging) -> Self {
        Self {
            log_level: logging.log_level.to_string(),
        }
    }
}

impl From<DomainWebsite> for Website {
    fn from(website: DomainWebsite) -> Self {
        Self { name: website.name }
    }
}

impl From<DomainTracker> for Tracker {
    fn from(tracker: DomainTracker) -> Self {
        Self {
            url: tracker.url,
            mode: tracker.mode.to_string(),
            api_url: tracker.api_url,
            token: tracker.token,
            token_valid_seconds: tracker.token_valid_seconds,
        }
    }
}

impl From<DomainNetwork> for Network {
    fn from(net: DomainNetwork) -> Self {
        Self {
            port: net.port,
            base_url: net.base_url.as_ref().map(std::string::ToString::to_string),
        }
    }
}

impl From<DomainAuth> for Auth {
    fn from(auth: DomainAuth) -> Self {
        Self {
            email_on_signup: auth.email_on_signup.to_string(),
            secret_key: auth.secret_key.to_string(),
            password_constraints: auth.password_constraints.into(),
        }
    }
}

impl From<DomainPasswordConstraints> for PasswordConstraints {
    fn from(password_constraints: DomainPasswordConstraints) -> Self {
        Self {
            min_password_length: password_constraints.min_password_length,
            max_password_length: password_constraints.max_password_length,
        }
    }
}

impl From<DomainDatabase> for Database {
    fn from(database: DomainDatabase) -> Self {
        Self {
            connect_url: database.connect_url.to_string(),
        }
    }
}

impl From<DomainMail> for Mail {
    fn from(mail: DomainMail) -> Self {
        Self {
            email_verification_enabled: mail.email_verification_enabled,
            from: mail.from.to_string(),
            reply_to: mail.reply_to.to_string(),
            smtp: mail.smtp.into(),
        }
    }
}

impl From<DomainSmtp> for Smtp {
    fn from(smtp: DomainSmtp) -> Self {
        Self {
            server: smtp.server,
            port: smtp.port,
            credentials: smtp.credentials.into(),
        }
    }
}

impl From<DomainCredentials> for Credentials {
    fn from(credentials: DomainCredentials) -> Self {
        Self {
            username: credentials.username,
            password: credentials.password,
        }
    }
}

impl From<DomainImageCache> for ImageCache {
    fn from(image_cache: DomainImageCache) -> Self {
        Self {
            max_request_timeout_ms: image_cache.max_request_timeout_ms,
            capacity: image_cache.capacity,
            entry_size_limit: image_cache.entry_size_limit,
            user_quota_period_seconds: image_cache.user_quota_period_seconds,
            user_quota_bytes: image_cache.user_quota_bytes,
        }
    }
}

impl From<DomainApi> for Api {
    fn from(api: DomainApi) -> Self {
        Self {
            default_torrent_page_size: api.default_torrent_page_size,
            max_torrent_page_size: api.max_torrent_page_size,
        }
    }
}

impl From<DomainTrackerStatisticsImporter> for TrackerStatisticsImporter {
    fn from(tracker_statistics_importer: DomainTrackerStatisticsImporter) -> Self {
        Self {
            torrent_info_update_interval: tracker_statistics_importer.torrent_info_update_interval,
            port: tracker_statistics_importer.port,
        }
    }
}
