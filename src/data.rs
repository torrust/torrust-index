use sqlx::SqlitePool;
use std::sync::Arc;
use sqlx::sqlite::SqlitePoolOptions;
use std::env;
use crate::models::user::User;
use crate::errors::ServiceError;
use crate::models::torrent_listing::TorrentListing;
use crate::utils::time::current_time;
use std::time::Duration;
use crate::models::tracker_key::TrackerKey;

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

    pub async fn update_torrent_info_hash(&self, torrent_id: i64, info_hash: String) -> Result<(), ServiceError> {
        let res = sqlx::query!(
            "UPDATE torrust_torrents SET info_hash = $1 WHERE torrent_id = $2",
            info_hash,
            torrent_id
        )
            .execute(&self.pool)
            .await;

        match res {
            Ok(_) => Ok(()),
            _ => Err(ServiceError::TorrentNotFound)
        }
    }

    pub async fn get_torrent_by_id(&self, torrent_id: i64) -> Option<TorrentListing> {
        let res = sqlx::query_as!(
            TorrentListing,
            r#"SELECT * FROM torrust_torrents
               WHERE torrent_id = ?"#,
            torrent_id
        )
            .fetch_one(&self.pool)
            .await;

        match res {
            Ok(torrent) => Some(torrent),
            _ => None
        }
    }

    pub async fn get_valid_tracker_key(&self, user_id: i64) -> Option<TrackerKey> {
        const WEEK: i64 = 604_800;
        let current_time_plus_week = (current_time() as i64) + WEEK;

        let res = sqlx::query_as!(
            TrackerKey,
            r#"SELECT key, valid_until FROM torrust_tracker_keys
               WHERE user_id = $1 AND valid_until > $2"#,
            user_id,
            current_time_plus_week
        )
            .fetch_one(&self.pool)
            .await;

        match res {
            Ok(tracker_key) => Some(tracker_key),
            _ => None
        }
    }

    pub async fn issue_tracker_key(&self, tracker_key: &TrackerKey, user_id: i64) -> Result<(), ServiceError> {
        let res = sqlx::query!(
            "INSERT INTO torrust_tracker_keys (user_id, key, valid_until) VALUES ($1, $2, $3)",
            user_id,
            tracker_key.key,
            tracker_key.valid_until,
        )
            .execute(&self.pool)
            .await;

        match res {
            Ok(_) => Ok(()),
            Err(_) => Err(ServiceError::InternalServerError)
        }
    }
}
