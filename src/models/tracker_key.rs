use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TrackerKey {
    pub key: String,
    pub valid_until: i64,
}
