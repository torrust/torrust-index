use serde::{Deserialize, Serialize};
use url::Url;

/// Database configuration.
///
/// `connect_url` is mandatory: there is no schema-level default and no
/// `impl Default for Database`. A missing value fails at
/// deserialisation with a precise serde `missing field 'connect_url'`
/// error (ADR-T-009 §D2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Database {
    /// The connection URL for the database. For example:
    ///
    /// Sqlite: `sqlite://data.db?mode=rwc`.
    /// Mysql: `mysql://root:root_secret_password@mysql:3306/torrust_index_e2e_testing`.
    pub connect_url: Url,
}
