use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{query_as, SqlitePool};

use crate::databases::database::DatabaseError;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub category_id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub email_verified: bool,
    pub password: String,
    pub administrator: bool,
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
        query_as::<_, Category>(
            "SELECT category_id, name FROM torrust_categories ORDER BY category_id ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| DatabaseError::Error)
    }

    pub async fn get_users(&self) -> Result<Vec<User>, sqlx::Error> {
        query_as::<_, User>(
            "SELECT * FROM torrust_users ORDER BY user_id ASC",
        )
        .fetch_all(&self.pool)
        .await
    }
}
