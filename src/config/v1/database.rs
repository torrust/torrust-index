use serde::{Deserialize, Serialize};
use url::Url;

/// Database configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Database {
    /// The connection URL for the database. For example:
    ///
    /// Sqlite: `sqlite://data.db?mode=rwc`.
    /// Mysql: `mysql://root:root_secret_password@mysql:3306/torrust_index_e2e_testing`.
    #[serde(default = "Database::default_connect_url")]
    pub connect_url: Url,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            connect_url: Self::default_connect_url(),
        }
    }
}

impl Database {
    fn default_connect_url() -> Url {
        Url::parse("sqlite://data.db?mode=rwc").unwrap()
    }
}
