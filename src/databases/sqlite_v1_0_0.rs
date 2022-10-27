use super::database::DatabaseError;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{query_as, SqlitePool};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub category_id: i64,
    pub name: String,
}
pub struct SqliteDatabaseV1_0_0 {
    pub pool: SqlitePool,
}

impl SqliteDatabaseV1_0_0 {
    pub async fn new(database_url: &str) -> Self {
        let db = SqlitePoolOptions::new()
            .connect(database_url)
            .await
            .expect("Unable to create database pool.");
        Self { pool: db }
    }

    pub async fn get_categories_order_by_id(&self) -> Result<Vec<Category>, DatabaseError> {
        query_as::<_, Category>("SELECT category_id, name FROM torrust_categories ORDER BY category_id ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(|_| DatabaseError::Error)
    }
}
