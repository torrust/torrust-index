use sqlx::SqlitePool;
use std::sync::Arc;
use sqlx::sqlite::SqlitePoolOptions;
use std::env;
use crate::models::user::User;

pub struct Database {
    pub pool: SqlitePool
}

impl Database {
    pub async fn new(database_url: &str) -> Database {
        let db = SqlitePoolOptions::new()
            .connect(database_url)
            .await
            .expect("Unable to create database pool");

        Database {
            pool: db
        }
    }

    pub async fn get_user_with_username(&self, username: &str) -> Option<User> {
        let res = sqlx::query_as!(
            User,
            "SELECT * FROM torrust_users WHERE username = ?",
            username,
        )
            .fetch_one(&self.pool)
            .await;

        match res {
            Ok(user) => Some(user),
            _ => None
        }
    }

    pub async fn get_user_with_email(&self, email: &str) -> Option<User> {
        let res = sqlx::query_as!(
            User,
            "SELECT * FROM torrust_users WHERE email = ?",
            email,
        )
            .fetch_one(&self.pool)
            .await;

        match res {
            Ok(user) => Some(user),
            _ => None
        }
    }
}
