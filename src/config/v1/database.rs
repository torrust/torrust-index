use std::fmt;

use serde::{Deserialize, Serialize};

/// Database configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Database {
    /// The connection string for the database. For example:
    ///
    /// Masked: `***`.
    /// Sqlite: `sqlite://data.db?mode=rwc`.
    /// Mysql: `mysql://root:root_secret_password@mysql:3306/torrust_index_e2e_testing`.
    pub connect_url: ConnectOptions,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            connect_url: ConnectOptions::new("sqlite://data.db?mode=rwc"),
        }
    }
}

/// This allows a particular case when we want to hide the connection options
/// because it contains secrets we don't want to show.
const DB_CONNECT_MASKED: &str = "***";

/// Prefix for connection to `SQLite` database.
const DB_CONNECT_SQLITE_PREFIX: &str = "sqlite://";

/// Prefix for connection to `MySQL` database.
const DB_CONNECT_MYSQL_PREFIX: &str = "mysql://";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConnectOptions(String);

impl ConnectOptions {
    /// # Panics
    ///
    /// Will panic if the connect options are empty.
    #[must_use]
    pub fn new(connect_options: &str) -> Self {
        assert!(!connect_options.is_empty(), "database connect options cannot be empty");
        assert!(
            connect_options.starts_with(DB_CONNECT_SQLITE_PREFIX)
                || connect_options.starts_with(DB_CONNECT_MYSQL_PREFIX)
                || connect_options.starts_with(DB_CONNECT_MASKED),
            "database driver not supported"
        );

        Self(connect_options.to_owned())
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Display for ConnectOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::ConnectOptions;

    #[test]
    #[should_panic(expected = "database connect options cannot be empty")]
    fn database_connect_options_can_not_be_empty() {
        drop(ConnectOptions::new(""));
    }

    #[test]
    #[should_panic(expected = "database driver not supported")]
    fn database_connect_options_only_supports_sqlite_and_mysql() {
        drop(ConnectOptions::new("not-supported://"));
    }

    #[test]
    fn database_connect_options_can_be_masked() {
        drop(ConnectOptions::new("***"));
    }
}
