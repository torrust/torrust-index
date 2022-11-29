use async_trait::async_trait;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::databases::mysql::MysqlDatabase;
use crate::databases::sqlite::SqliteDatabase;
use crate::models::response::TorrentsResponse;
use crate::models::torrent::TorrentListing;
use crate::models::torrent_file::{DbTorrentInfo, Torrent, TorrentFile};
use crate::models::tracker_key::TrackerKey;
use crate::models::user::{User, UserAuthentication, UserCompact, UserProfile};

/// Database drivers.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseDriver {
    Sqlite3,
    Mysql,
}

/// Compact representation of torrent.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TorrentCompact {
    pub torrent_id: i64,
    pub info_hash: String,
}

/// Torrent category.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub category_id: i64,
    pub name: String,
    pub num_torrents: i64,
}

/// Sorting options for torrents.
#[derive(Clone, Copy, Debug, Deserialize)]
pub enum Sorting {
    UploadedAsc,
    UploadedDesc,
    SeedersAsc,
    SeedersDesc,
    LeechersAsc,
    LeechersDesc,
    NameAsc,
    NameDesc,
    SizeAsc,
    SizeDesc,
}

/// Database errors.
#[derive(Debug)]
pub enum DatabaseError {
    Error,
    UnrecognizedDatabaseDriver, // when the db path does not start with sqlite or mysql
    UsernameTaken,
    EmailTaken,
    UserNotFound,
    CategoryAlreadyExists,
    CategoryNotFound,
    TorrentNotFound,
    TorrentAlreadyExists, // when uploading an already uploaded info_hash
    TorrentTitleAlreadyExists,
}

/// Connect to a database.
pub async fn connect_database(db_path: &str) -> Result<Box<dyn Database>, DatabaseError> {
    match &db_path.chars().collect::<Vec<char>>() as &[char] {
        ['s', 'q', 'l', 'i', 't', 'e', ..] => {
            let db = SqliteDatabase::new(db_path).await;
            Ok(Box::new(db))
        }
        ['m', 'y', 's', 'q', 'l', ..] => {
            let db = MysqlDatabase::new(db_path).await;
            Ok(Box::new(db))
        }
        _ => Err(DatabaseError::UnrecognizedDatabaseDriver),
    }
}

/// Trait for database implementations.
#[async_trait]
pub trait Database: Sync + Send {
    /// Return current database driver.
    fn get_database_driver(&self) -> DatabaseDriver;

    /// Add new user and return the newly inserted `user_id`.
    async fn insert_user_and_get_id(&self, username: &str, email: &str, password: &str) -> Result<i64, DatabaseError>;

    /// Get `User` from `user_id`.
    async fn get_user_from_id(&self, user_id: i64) -> Result<User, DatabaseError>;

    /// Get `UserAuthentication` from `user_id`.
    async fn get_user_authentication_from_id(&self, user_id: i64) -> Result<UserAuthentication, DatabaseError>;

    /// Get `UserProfile` from `username`.
    async fn get_user_profile_from_username(&self, username: &str) -> Result<UserProfile, DatabaseError>;

    /// Get `UserCompact` from `user_id`.
    async fn get_user_compact_from_id(&self, user_id: i64) -> Result<UserCompact, DatabaseError>;

    /// Get a user's `TrackerKey`.
    async fn get_user_tracker_key(&self, user_id: i64) -> Option<TrackerKey>;

    /// Get total user count.
    async fn count_users(&self) -> Result<i64, DatabaseError>;

    /// Ban user with `user_id`, `reason` and `date_expiry`.
    async fn ban_user(&self, user_id: i64, reason: &str, date_expiry: NaiveDateTime) -> Result<(), DatabaseError>;

    /// Grant a user the administrator role.
    async fn grant_admin_role(&self, user_id: i64) -> Result<(), DatabaseError>;

    /// Verify a user's email with `user_id`.
    async fn verify_email(&self, user_id: i64) -> Result<(), DatabaseError>;

    /// Link a `TrackerKey` to a certain user with `user_id`.
    async fn add_tracker_key(&self, user_id: i64, tracker_key: &TrackerKey) -> Result<(), DatabaseError>;

    /// Delete user and all related user data with `user_id`.
    async fn delete_user(&self, user_id: i64) -> Result<(), DatabaseError>;

    /// Add a new category and return `category_id`.
    async fn insert_category_and_get_id(&self, category_name: &str) -> Result<i64, DatabaseError>;

    /// Get `Category` from `category_id`.
    async fn get_category_from_id(&self, category_id: i64) -> Result<Category, DatabaseError>;

    /// Get `Category` from `category_name`.
    async fn get_category_from_name(&self, category_name: &str) -> Result<Category, DatabaseError>;

    /// Get all categories as `Vec<Category>`.
    async fn get_categories(&self) -> Result<Vec<Category>, DatabaseError>;

    /// Delete category with `category_name`.
    async fn delete_category(&self, category_name: &str) -> Result<(), DatabaseError>;

    /// Get results of a torrent search in a paginated and sorted form as `TorrentsResponse` from `search`, `categories`, `sort`, `offset` and `page_size`.
    async fn get_torrents_search_sorted_paginated(
        &self,
        search: &Option<String>,
        categories: &Option<Vec<String>>,
        sort: &Sorting,
        offset: u64,
        page_size: u8,
    ) -> Result<TorrentsResponse, DatabaseError>;

    /// Add new torrent and return the newly inserted `torrent_id` with `torrent`, `uploader_id`, `category_id`, `title` and `description`.
    async fn insert_torrent_and_get_id(
        &self,
        torrent: &Torrent,
        uploader_id: i64,
        category_id: i64,
        title: &str,
        description: &str,
    ) -> Result<i64, DatabaseError>;

    /// Get `Torrent` from `torrent_id`.
    async fn get_torrent_from_id(&self, torrent_id: i64) -> Result<Torrent, DatabaseError>;

    /// Get torrent's info as `DbTorrentInfo` from `torrent_id`.
    async fn get_torrent_info_from_id(&self, torrent_id: i64) -> Result<DbTorrentInfo, DatabaseError>;

    /// Get all torrent's files as `Vec<TorrentFile>` from `torrent_id`.
    async fn get_torrent_files_from_id(&self, torrent_id: i64) -> Result<Vec<TorrentFile>, DatabaseError>;

    /// Get all torrent's announce urls as `Vec<Vec<String>>` from `torrent_id`.
    async fn get_torrent_announce_urls_from_id(&self, torrent_id: i64) -> Result<Vec<Vec<String>>, DatabaseError>;

    /// Get `TorrentListing` from `torrent_id`.
    async fn get_torrent_listing_from_id(&self, torrent_id: i64) -> Result<TorrentListing, DatabaseError>;

    /// Get all torrents as `Vec<TorrentCompact>`.
    async fn get_all_torrents_compact(&self) -> Result<Vec<TorrentCompact>, DatabaseError>;

    /// Update a torrent's title with `torrent_id` and `title`.
    async fn update_torrent_title(&self, torrent_id: i64, title: &str) -> Result<(), DatabaseError>;

    /// Update a torrent's description with `torrent_id` and `description`.
    async fn update_torrent_description(&self, torrent_id: i64, description: &str) -> Result<(), DatabaseError>;

    /// Update the seeders and leechers info for a torrent with `torrent_id`, `tracker_url`, `seeders` and `leechers`.
    async fn update_tracker_info(
        &self,
        torrent_id: i64,
        tracker_url: &str,
        seeders: i64,
        leechers: i64,
    ) -> Result<(), DatabaseError>;

    /// Delete a torrent with `torrent_id`.
    async fn delete_torrent(&self, torrent_id: i64) -> Result<(), DatabaseError>;

    /// DELETES ALL DATABASE ROWS, ONLY CALL THIS IF YOU KNOW WHAT YOU'RE DOING!
    async fn delete_all_database_rows(&self) -> Result<(), DatabaseError>;
}
