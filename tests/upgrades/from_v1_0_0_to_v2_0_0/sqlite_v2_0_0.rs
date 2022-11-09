use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{query_as, SqlitePool};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserRecordV2 {
    pub user_id: i64,
    pub date_registered: Option<String>,
    pub date_imported: Option<String>,
    pub administrator: bool,
}

pub struct SqliteDatabaseV2_0_0 {
    pub pool: SqlitePool,
}

impl SqliteDatabaseV2_0_0 {
    pub async fn db_connection(database_file: &str) -> Self {
        let connect_url = format!("sqlite://{}?mode=rwc", database_file);
        Self::new(&connect_url).await
    }

    pub async fn new(database_url: &str) -> Self {
        let db = SqlitePoolOptions::new()
            .connect(database_url)
            .await
            .expect("Unable to create database pool.");
        Self { pool: db }
    }

    pub async fn get_user(&self, user_id: i64) -> Result<UserRecordV2, sqlx::Error> {
        query_as::<_, UserRecordV2>("SELECT * FROM torrust_users WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
    }
}
