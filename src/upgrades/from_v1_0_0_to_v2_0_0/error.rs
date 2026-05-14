use thiserror::Error;

use crate::databases::database;

#[derive(Debug, Error)]
pub enum UpgradeError {
    #[error("{context}: {source}")]
    Database { context: &'static str, source: database::Error },

    #[error("{context}: {source}")]
    Sqlx { context: &'static str, source: sqlx::Error },

    #[error("{context}: {source}")]
    Migration {
        context: &'static str,
        source: sqlx::migrate::MigrateError,
    },

    #[error("{context}: {source}")]
    Io { context: &'static str, source: std::io::Error },

    #[error("failed to decode torrent file {path}: {message}")]
    DecodeTorrent { path: String, message: String },

    #[error("{entity} id mismatch while copying data: expected {expected}, got {actual}")]
    IdMismatch {
        entity: &'static str,
        expected: i64,
        actual: i64,
    },

    #[error("uploader mismatch for torrent {torrent_id}: expected {expected}, got {actual}")]
    UploaderMismatch {
        torrent_id: i64,
        expected: String,
        actual: String,
    },

    #[error("missing {field} for torrent {torrent_id}")]
    MissingTorrentField { torrent_id: i64, field: &'static str },

    #[error("invalid timestamp {timestamp}")]
    InvalidTimestamp { timestamp: i64 },
}
