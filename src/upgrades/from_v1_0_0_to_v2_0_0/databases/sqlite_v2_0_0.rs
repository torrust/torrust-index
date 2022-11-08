use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePoolOptions, SqliteQueryResult};
use sqlx::{query, query_as, SqlitePool};

use crate::databases::database::DatabaseError;
use crate::models::torrent_file::TorrentFile;

use super::sqlite_v1_0_0::Torrent;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub category_id: i64,
    pub name: String,
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
            .expect("Could not run database migrations.")
    }

    pub async fn reset_categories_sequence(&self) -> Result<SqliteQueryResult, DatabaseError> {
        query("DELETE FROM `sqlite_sequence` WHERE `name` = 'torrust_categories'")
            .execute(&self.pool)
            .await
            .map_err(|_| DatabaseError::Error)
    }

    pub async fn get_categories(&self) -> Result<Vec<Category>, DatabaseError> {
        query_as::<_, Category>("SELECT tc.category_id, tc.name, COUNT(tt.category_id) as num_torrents FROM torrust_categories tc LEFT JOIN torrust_torrents tt on tc.category_id = tt.category_id GROUP BY tc.name")
            .fetch_all(&self.pool)
            .await
            .map_err(|_| DatabaseError::Error)
    }

    pub async fn insert_category_and_get_id(
        &self,
        category_name: &str,
    ) -> Result<i64, DatabaseError> {
        query("INSERT INTO torrust_categories (name) VALUES (?)")
            .bind(category_name)
            .execute(&self.pool)
            .await
            .map(|v| v.last_insert_rowid())
            .map_err(|e| match e {
                sqlx::Error::Database(err) => {
                    if err.message().contains("UNIQUE") {
                        DatabaseError::CategoryAlreadyExists
                    } else {
                        DatabaseError::Error
                    }
                }
                _ => DatabaseError::Error,
            })
    }

    pub async fn insert_user(
        &self,
        user_id: i64,
        date_registered: &str,
        administrator: bool,
    ) -> Result<i64, sqlx::Error> {
        query(
            "INSERT INTO torrust_users (user_id, date_registered, administrator) VALUES (?, ?, ?)",
        )
        .bind(user_id)
        .bind(date_registered)
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
        bio: &str,
        avatar: &str,
    ) -> Result<i64, sqlx::Error> {
        query("INSERT INTO torrust_user_profiles (user_id, username, email, email_verified, bio, avatar) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(user_id)
            .bind(username)
            .bind(email)
            .bind(email_verified)
            .bind(bio)
            .bind(avatar)
            .execute(&self.pool)
            .await
            .map(|v| v.last_insert_rowid())
    }

    pub async fn insert_user_password_hash(
        &self,
        user_id: i64,
        password_hash: &str,
    ) -> Result<i64, sqlx::Error> {
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

    pub async fn insert_torrent(
        &self,
        torrent_id: i64,
        uploader_id: i64,
        category_id: i64,
        info_hash: &str,
        size: i64,
        name: &str,
        pieces: &str,
        piece_length: i64,
        private: bool,
        root_hash: i64,
        date_uploaded: &str,
    ) -> Result<i64, sqlx::Error> {
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
        .bind(torrent_id)
        .bind(uploader_id)
        .bind(category_id)
        .bind(info_hash)
        .bind(size)
        .bind(name)
        .bind(pieces)
        .bind(piece_length)
        .bind(private)
        .bind(root_hash)
        .bind(date_uploaded)
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
        query(
            "
            INSERT INTO torrust_torrent_files (md5sum, torrent_id, LENGTH)
            VALUES (?, ?, ?)",
        )
        .bind(md5sum)
        .bind(torrent_id)
        .bind(length)
        .execute(&self.pool)
        .await
        .map(|v| v.last_insert_rowid())
    }

    pub async fn insert_torrent_file_for_torrent_with_multiple_files(
        &self,
        torrent: &Torrent,
        file: &TorrentFile,
    ) -> Result<i64, sqlx::Error> {
        query(
            "INSERT INTO torrust_torrent_files (md5sum, torrent_id, LENGTH, PATH)
        VALUES (?, ?, ?, ?)",
        )
        .bind(file.md5sum.clone())
        .bind(torrent.torrent_id)
        .bind(file.length)
        .bind(file.path.join("/"))
        .execute(&self.pool)
        .await
        .map(|v| v.last_insert_rowid())
    }

    pub async fn delete_all_database_rows(&self) -> Result<(), DatabaseError> {
        query("DELETE FROM torrust_categories;")
            .execute(&self.pool)
            .await
            .unwrap();

        query("DELETE FROM torrust_torrents;")
            .execute(&self.pool)
            .await
            .unwrap();

        query("DELETE FROM torrust_tracker_keys;")
            .execute(&self.pool)
            .await
            .unwrap();

        query("DELETE FROM torrust_users;")
            .execute(&self.pool)
            .await
            .unwrap();

        query("DELETE FROM torrust_user_authentication;")
            .execute(&self.pool)
            .await
            .unwrap();

        query("DELETE FROM torrust_user_bans;")
            .execute(&self.pool)
            .await
            .unwrap();

        query("DELETE FROM torrust_user_invitations;")
            .execute(&self.pool)
            .await
            .unwrap();

        query("DELETE FROM torrust_user_profiles;")
            .execute(&self.pool)
            .await
            .unwrap();

        query("DELETE FROM torrust_torrents;")
            .execute(&self.pool)
            .await
            .unwrap();

        query("DELETE FROM torrust_user_public_keys;")
            .execute(&self.pool)
            .await
            .unwrap();

        Ok(())
    }
}
