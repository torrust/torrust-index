use serde::{Deserialize, Serialize};

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
