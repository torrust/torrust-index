#![allow(clippy::missing_errors_doc)]

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePoolOptions, SqliteQueryResult};
use sqlx::{query, query_as, SqlitePool};

use super::sqlite_v1_0_0::{TorrentRecordV1, UserRecordV1};
use crate::databases::database;
use crate::models::torrent_file::{TorrentFile, TorrentInfo};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CategoryRecordV2 {
    pub category_id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TorrentRecordV2 {
    pub torrent_id: i64,
    pub uploader_id: i64,
    pub category_id: i64,
    pub info_hash: String,
    pub size: i64,
    pub name: String,
    pub pieces: String,
    pub piece_length: i64,
    pub private: Option<u8>,
    pub root_hash: i64,
    pub date_uploaded: String,
}

impl TorrentRecordV2 {
    #[must_use]
    pub fn from_v1_data(torrent: &TorrentRecordV1, torrent_info: &TorrentInfo, uploader: &UserRecordV1) -> Self {
        Self {
            torrent_id: torrent.torrent_id,
            uploader_id: uploader.user_id,
            category_id: torrent.category_id,
            info_hash: torrent.info_hash.clone(),
            size: torrent.file_size,
            name: torrent_info.name.clone(),
            pieces: torrent_info.get_pieces_as_string(),
            piece_length: torrent_info.piece_length,
            private: torrent_info.private,
            root_hash: torrent_info.get_root_hash_as_i64(),
            date_uploaded: convert_timestamp_to_datetime(torrent.upload_date),
        }
    }
}

#[must_use]
pub fn convert_timestamp_to_datetime(timestamp: i64) -> String {
    // The expected format in database is: 2022-11-04 09:53:57
    // MySQL uses a DATETIME column and SQLite uses a TEXT column.

    let naive_datetime = NaiveDateTime::from_timestamp_opt(timestamp, 0).expect("Overflow of i64 seconds, very future!");
    let datetime_again: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);

    // Format without timezone
    datetime_again.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub struct SqliteDatabaseV2_0_0 {
    pub pool: SqlitePool,
}

impl SqliteDatabaseV2_0_0 {
    pub async fn new(database_url: &str) -> Self {
        let db = SqlitePoolOptions::new()
            .connect(database_url)
            .await
            .expect("Unable to create database pool.");
        Self { pool: db }
    }

    pub async fn migrate(&self) {
        sqlx::migrate!("migrations/sqlite3")
            .run(&self.pool)
            .await
            .expect("Could not run database migrations.");
    }

    pub async fn reset_categories_sequence(&self) -> Result<SqliteQueryResult, database::Error> {
        query("DELETE FROM `sqlite_sequence` WHERE `name` = 'torrust_categories'")
            .execute(&self.pool)
            .await
            .map_err(|_| database::Error::Error)
    }

    pub async fn get_categories(&self) -> Result<Vec<CategoryRecordV2>, database::Error> {
        query_as::<_, CategoryRecordV2>("SELECT tc.category_id, tc.name, COUNT(tt.category_id) as num_torrents FROM torrust_categories tc LEFT JOIN torrust_torrents tt on tc.category_id = tt.category_id GROUP BY tc.name")
            .fetch_all(&self.pool)
            .await
            .map_err(|_| database::Error::Error)
    }

    pub async fn insert_category_and_get_id(&self, category_name: &str) -> Result<i64, database::Error> {
        query("INSERT INTO torrust_categories (name) VALUES (?)")
            .bind(category_name)
            .execute(&self.pool)
            .await
            .map(|v| v.last_insert_rowid())
            .map_err(|e| match e {
                sqlx::Error::Database(err) => {
                    if err.message().contains("UNIQUE") {
                        database::Error::CategoryAlreadyExists
                    } else {
                        database::Error::Error
                    }
                }
                _ => database::Error::Error,
            })
    }

    pub async fn insert_category(&self, category: &CategoryRecordV2) -> Result<i64, sqlx::Error> {
        query("INSERT INTO torrust_categories (category_id, name) VALUES (?, ?)")
            .bind(category.category_id)
            .bind(category.name.clone())
            .execute(&self.pool)
            .await
            .map(|v| v.last_insert_rowid())
    }

    pub async fn insert_imported_user(&self, user_id: i64, date_imported: &str, administrator: bool) -> Result<i64, sqlx::Error> {
        query("INSERT INTO torrust_users (user_id, date_imported, administrator) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(date_imported)
            .bind(administrator)
            .execute(&self.pool)
            .await
            .map(|v| v.last_insert_rowid())
    }

    pub async fn insert_user_profile(
        &self,
        user_id: i64,
        username: &str,
        email: &str,
        email_verified: bool,
    ) -> Result<i64, sqlx::Error> {
        query(
            "INSERT INTO torrust_user_profiles (user_id, username, email, email_verified, bio, avatar) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind(username)
        .bind(email)
        .bind(email_verified)
        .execute(&self.pool)
        .await
        .map(|v| v.last_insert_rowid())
    }

    pub async fn insert_user_password_hash(&self, user_id: i64, password_hash: &str) -> Result<i64, sqlx::Error> {
        query("INSERT INTO torrust_user_authentication (user_id, password_hash) VALUES (?, ?)")
            .bind(user_id)
            .bind(password_hash)
            .execute(&self.pool)
            .await
            .map(|v| v.last_insert_rowid())
    }

    pub async fn insert_tracker_key(
        &self,
        tracker_key_id: i64,
        user_id: i64,
        tracker_key: &str,
        date_expiry: i64,
    ) -> Result<i64, sqlx::Error> {
        query("INSERT INTO torrust_tracker_keys (tracker_key_id, user_id, tracker_key, date_expiry) VALUES (?, ?, ?, ?)")
            .bind(tracker_key_id)
            .bind(user_id)
            .bind(tracker_key)
            .bind(date_expiry)
            .execute(&self.pool)
            .await
            .map(|v| v.last_insert_rowid())
    }

    pub async fn insert_torrent(&self, torrent: &TorrentRecordV2) -> Result<i64, sqlx::Error> {
        query(
            "
            INSERT INTO torrust_torrents (
                torrent_id,
                uploader_id,
                category_id,
                info_hash,
                size,
                name,
                pieces,
                piece_length,
                private,
                root_hash,
                date_uploaded
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(torrent.torrent_id)
        .bind(torrent.uploader_id)
        .bind(torrent.category_id)
        .bind(torrent.info_hash.clone())
        .bind(torrent.size)
        .bind(torrent.name.clone())
        .bind(torrent.pieces.clone())
        .bind(torrent.piece_length)
        .bind(torrent.private.unwrap_or(0))
        .bind(torrent.root_hash)
        .bind(torrent.date_uploaded.clone())
        .execute(&self.pool)
        .await
        .map(|v| v.last_insert_rowid())
    }

    pub async fn insert_torrent_file_for_torrent_with_one_file(
        &self,
        torrent_id: i64,
        md5sum: &Option<String>,
        length: i64,
    ) -> Result<i64, sqlx::Error> {
        query("INSERT INTO torrust_torrent_files (md5sum, torrent_id, LENGTH) VALUES (?, ?, ?)")
            .bind(md5sum)
            .bind(torrent_id)
            .bind(length)
            .execute(&self.pool)
            .await
            .map(|v| v.last_insert_rowid())
    }

    pub async fn insert_torrent_file_for_torrent_with_multiple_files(
        &self,
        torrent: &TorrentRecordV1,
        file: &TorrentFile,
    ) -> Result<i64, sqlx::Error> {
        query("INSERT INTO torrust_torrent_files (md5sum, torrent_id, LENGTH, PATH) VALUES (?, ?, ?, ?)")
            .bind(file.md5sum.clone())
            .bind(torrent.torrent_id)
            .bind(file.length)
            .bind(file.path.join("/"))
            .execute(&self.pool)
            .await
            .map(|v| v.last_insert_rowid())
    }

    pub async fn insert_torrent_info(&self, torrent: &TorrentRecordV1) -> Result<i64, sqlx::Error> {
        query("INSERT INTO torrust_torrent_info (torrent_id, title, description) VALUES (?, ?, ?)")
            .bind(torrent.torrent_id)
            .bind(torrent.title.clone())
            .bind(torrent.description.clone())
            .execute(&self.pool)
            .await
            .map(|v| v.last_insert_rowid())
    }

    pub async fn insert_torrent_announce_url(&self, torrent_id: i64, tracker_url: &str) -> Result<i64, sqlx::Error> {
        query("INSERT INTO torrust_torrent_announce_urls (torrent_id, tracker_url) VALUES (?, ?)")
            .bind(torrent_id)
            .bind(tracker_url)
            .execute(&self.pool)
            .await
            .map(|v| v.last_insert_rowid())
    }

    #[allow(clippy::missing_panics_doc)]
    pub async fn delete_all_database_rows(&self) -> Result<(), database::Error> {
        query("DELETE FROM torrust_categories").execute(&self.pool).await.unwrap();

        query("DELETE FROM torrust_torrents").execute(&self.pool).await.unwrap();

        query("DELETE FROM torrust_tracker_keys").execute(&self.pool).await.unwrap();

        query("DELETE FROM torrust_users").execute(&self.pool).await.unwrap();

        query("DELETE FROM torrust_user_authentication")
            .execute(&self.pool)
            .await
            .unwrap();

        query("DELETE FROM torrust_user_bans").execute(&self.pool).await.unwrap();

        query("DELETE FROM torrust_user_invitations")
            .execute(&self.pool)
            .await
            .unwrap();

        query("DELETE FROM torrust_user_profiles").execute(&self.pool).await.unwrap();

        query("DELETE FROM torrust_torrents").execute(&self.pool).await.unwrap();

        query("DELETE FROM torrust_user_public_keys")
            .execute(&self.pool)
            .await
            .unwrap();

        Ok(())
    }
}
