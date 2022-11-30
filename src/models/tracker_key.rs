use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TrackerKey {
    pub key: String,
    pub valid_until: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewTrackerKey {
    pub key: String,
    pub valid_until: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Duration {
    pub secs: i64,
    pub nanos: i64,
}
