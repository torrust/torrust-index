use sqlx::SqlitePool;
use std::sync::Arc;
use sqlx::sqlite::SqlitePoolOptions;

use crate::CONFIG;

pub struct Data {
    pub db: SqlitePool
}

impl Data {
    pub async fn new() -> Arc<Self> {
        let db = SqlitePoolOptions::new()
            .connect(&CONFIG.database.connect_url)
            .await
            .expect("Unable to create database pool");

        let data: Data = Data {
            db
        };

        Arc::new(data)
    }
}