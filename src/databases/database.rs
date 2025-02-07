use async_trait::async_trait;
use bittorrent_primitives::info_hash::InfoHash;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::databases::mysql::Mysql;
use crate::databases::sqlite::Sqlite;
use crate::models::category::CategoryId;
use crate::models::response::{TorrentsResponse, UserProfilesResponse};
use crate::models::torrent::{Metadata, TorrentListing};
use crate::models::torrent_file::{DbTorrent, Torrent, TorrentFile};
use crate::models::torrent_tag::{TagId, TorrentTag};
use crate::models::tracker_key::TrackerKey;
use crate::models::user::{User, UserAuthentication, UserCompact, UserId, UserProfile};
use crate::services::torrent::CanonicalInfoHashGroup;

/// Database tables to be truncated when upgrading from v1.0.0 to v2.0.0.
/// They must be in the correct order to avoid foreign key errors.
pub const TABLES_TO_TRUNCATE: &[&str] = &[
    "torrust_torrent_announce_urls",
    "torrust_torrent_files",
    "torrust_torrent_info",
    "torrust_torrent_tag_links",
    "torrust_torrent_tracker_stats",
    "torrust_torrents",
    "torrust_tracker_keys",
    "torrust_user_authentication",
    "torrust_user_bans",
    "torrust_user_invitation_uses",
    "torrust_user_invitations",
    "torrust_user_profiles",
    "torrust_user_public_keys",
    "torrust_users",
    "torrust_categories",
    "torrust_torrent_tags",
];

/// Database drivers.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum Driver {
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

/// Sorting options for users.
#[derive(Clone, Copy, Debug, Deserialize)]
pub enum UsersSorting {
    DateRegisteredNewest,
    DateRegisteredOldest,
    UsernameAZ,
    UsernameZA,
}

/// Database errors.
#[derive(Debug)]
pub enum Error {
    Error,
    ErrorWithText(String),
    UnrecognizedDatabaseDriver, // when the db path does not start with sqlite or mysql
    UsernameTaken,
    EmailTaken,
    UserNotFound,
    CategoryNotFound,
    TagAlreadyExists,
    TagNotFound,
    TorrentNotFound,
    TorrentAlreadyExists, // when uploading an already uploaded info_hash
    TorrentTitleAlreadyExists,
    TorrentInfoHashNotFound,
}

/// Get the Driver of the Database from the Connection String
///
/// # Errors
///
/// This function will return an `Error::UnrecognizedDatabaseDriver` if unable to match database type.
pub fn get_driver(db_path: &str) -> Result<Driver, Error> {
    match &db_path.chars().collect::<Vec<char>>() as &[char] {
        ['s', 'q', 'l', 'i', 't', 'e', ..] => Ok(Driver::Sqlite3),
        ['m', 'y', 's', 'q', 'l', ..] => Ok(Driver::Mysql),
        _ => Err(Error::UnrecognizedDatabaseDriver),
    }
}

/// Connect to a database.
///
/// # Errors
///
/// This function will return an `Error::UnrecognizedDatabaseDriver` if unable to match database type.
pub async fn connect(db_path: &str) -> Result<Box<dyn Database>, Error> {
    let db_driver = self::get_driver(db_path)?;

    Ok(match db_driver {
        self::Driver::Sqlite3 => Box::new(Sqlite::new(db_path).await),
        self::Driver::Mysql => Box::new(Mysql::new(db_path).await),
    })
}

/// Trait for database implementations.
#[async_trait]
pub trait Database: Sync + Send {
    /// Return current database driver.
    fn get_database_driver(&self) -> Driver;

    async fn new(db_path: &str) -> Self
    where
        Self: Sized;

    /// Add new user and return the newly inserted `user_id`.
    async fn insert_user_and_get_id(&self, username: &str, email: &str, password: &str) -> Result<UserId, Error>;

    /// Change user's password.
    async fn change_user_password(&self, user_id: i64, new_password: &str) -> Result<(), Error>;

    /// Get `User` from `user_id`.
    async fn get_user_from_id(&self, user_id: i64) -> Result<User, Error>;

    /// Get `UserAuthentication` from `user_id`.
    async fn get_user_authentication_from_id(&self, user_id: UserId) -> Result<UserAuthentication, Error>;

    /// Get `UserProfile` from `username`.
    async fn get_user_profile_from_username(&self, username: &str) -> Result<UserProfile, Error>;

    /// Get all user profiles in a paginated and sorted form as `UserProfilesResponse` from `search`, `filters`, `sort`, `offset` and `page_size`.
    async fn get_user_profiles_search_paginated(
        &self,
        search: &Option<String>,
        filters: &Option<Vec<String>>,
        sort: &UsersSorting,
        offset: u64,
        page_size: u8,
    ) -> Result<UserProfilesResponse, Error>;

    /// Get `UserCompact` from `user_id`.
    async fn get_user_compact_from_id(&self, user_id: i64) -> Result<UserCompact, Error>;

    /// Get a user's `TrackerKey`.
    async fn get_user_tracker_key(&self, user_id: i64) -> Option<TrackerKey>;

    /// Get total user count.
    async fn count_users(&self) -> Result<i64, Error>;

    /// Ban user with `user_id`, `reason` and `date_expiry`.
    async fn ban_user(&self, user_id: i64, reason: &str, date_expiry: NaiveDateTime) -> Result<(), Error>;

    /// Grant a user the administrator role.
    async fn grant_admin_role(&self, user_id: i64) -> Result<(), Error>;

    /// Verify a user's email with `user_id`.
    async fn verify_email(&self, user_id: i64) -> Result<(), Error>;

    /// Link a `TrackerKey` to a certain user with `user_id`.
    async fn add_tracker_key(&self, user_id: i64, tracker_key: &TrackerKey) -> Result<(), Error>;

    /// Delete user and all related user data with `user_id`.
    async fn delete_user(&self, user_id: i64) -> Result<(), Error>;

    /// Add a new category and return `category_id`.
    async fn insert_category_and_get_id(&self, category_name: &str) -> Result<i64, Error>;

    /// Get `Category` from `category_id`.
    async fn get_category_from_id(&self, category_id: i64) -> Result<Category, Error>;

    /// Get `Category` from `category_name`.
    async fn get_category_from_name(&self, category_name: &str) -> Result<Category, Error>;

    /// Get all categories as `Vec<Category>`.
    async fn get_categories(&self) -> Result<Vec<Category>, Error>;

    /// Delete category with `category_name`.
    async fn delete_category(&self, category_name: &str) -> Result<(), Error>;

    /// Get results of a torrent search in a paginated and sorted form as `TorrentsResponse` from `search`, `categories`, `sort`, `offset` and `page_size`.
    async fn get_torrents_search_sorted_paginated(
        &self,
        search: &Option<String>,
        categories: &Option<Vec<String>>,
        tags: &Option<Vec<String>>,
        sort: &Sorting,
        offset: u64,
        page_size: u8,
    ) -> Result<TorrentsResponse, Error>;

    /// Add new torrent and return the newly inserted `torrent_id` with `torrent`, `uploader_id`, `category_id`, `title` and `description`.
    async fn insert_torrent_and_get_id(
        &self,
        original_info_hash: &InfoHash,
        torrent: &Torrent,
        uploader_id: UserId,
        metadata: &Metadata,
    ) -> Result<i64, Error>;

    /// Get `Torrent` from `InfoHash`.
    async fn get_torrent_from_info_hash(&self, info_hash: &InfoHash) -> Result<Torrent, Error> {
        let db_torrent = self.get_torrent_info_from_info_hash(info_hash).await?;

        let torrent_files = self.get_torrent_files_from_id(db_torrent.torrent_id).await?;

        let torrent_announce_urls = self.get_torrent_announce_urls_from_id(db_torrent.torrent_id).await?;

        let torrent_http_seed_urls = self.get_torrent_http_seed_urls_from_id(db_torrent.torrent_id).await?;

        let torrent_nodes = self.get_torrent_nodes_from_id(db_torrent.torrent_id).await?;

        Ok(Torrent::from_database(
            &db_torrent,
            &torrent_files,
            torrent_announce_urls,
            torrent_http_seed_urls,
            torrent_nodes,
        ))
    }

    /// Get `Torrent` from `torrent_id`.
    async fn get_torrent_from_id(&self, torrent_id: i64) -> Result<Torrent, Error> {
        let db_torrent = self.get_torrent_info_from_id(torrent_id).await?;

        let torrent_files = self.get_torrent_files_from_id(torrent_id).await?;

        let torrent_announce_urls = self.get_torrent_announce_urls_from_id(torrent_id).await?;

        let torrent_http_seed_urls = self.get_torrent_http_seed_urls_from_id(db_torrent.torrent_id).await?;

        let torrent_nodes = self.get_torrent_nodes_from_id(db_torrent.torrent_id).await?;

        Ok(Torrent::from_database(
            &db_torrent,
            &torrent_files,
            torrent_announce_urls,
            torrent_http_seed_urls,
            torrent_nodes,
        ))
    }

    /// It returns the list of all infohashes producing the same canonical
    /// infohash.
    ///
    /// If the original infohash was unknown, it returns the canonical infohash.
    ///
    /// # Errors
    ///
    /// Returns an error is there was a problem with the database.
    async fn get_torrent_canonical_info_hash_group(&self, canonical: &InfoHash) -> Result<CanonicalInfoHashGroup, Error>;

    /// It returns the [`CanonicalInfoHashGroup`] the info-hash belongs to, if
    /// the info-hash belongs to a group. Otherwise, returns `None`.
    ///
    /// # Errors
    ///
    /// Returns an error is there was a problem with the database.
    async fn find_canonical_info_hash_for(&self, info_hash: &InfoHash) -> Result<Option<InfoHash>, Error>;

    /// It adds a new info-hash to the canonical info-hash group.
    ///
    /// # Errors
    ///
    /// Returns an error is there was a problem with the database.
    async fn add_info_hash_to_canonical_info_hash_group(&self, original: &InfoHash, canonical: &InfoHash) -> Result<(), Error>;

    /// Get torrent's info as `DbTorrentInfo` from `torrent_id`.
    async fn get_torrent_info_from_id(&self, torrent_id: i64) -> Result<DbTorrent, Error>;

    /// Get torrent's info as `DbTorrentInfo` from torrent `InfoHash`.
    async fn get_torrent_info_from_info_hash(&self, info_hash: &InfoHash) -> Result<DbTorrent, Error>;

    /// Get all torrent's files as `Vec<TorrentFile>` from `torrent_id`.
    async fn get_torrent_files_from_id(&self, torrent_id: i64) -> Result<Vec<TorrentFile>, Error>;

    /// Get all torrent's announce urls as `Vec<Vec<String>>` from `torrent_id`.
    async fn get_torrent_announce_urls_from_id(&self, torrent_id: i64) -> Result<Vec<Vec<String>>, Error>;

    /// Get all torrent's HTTP seed urls as `Vec<Vec<String>>` from `torrent_id`.
    async fn get_torrent_http_seed_urls_from_id(&self, torrent_id: i64) -> Result<Vec<String>, Error>;

    /// Get all torrent's nodes as `Vec<(String, i64)>` from `torrent_id`.
    async fn get_torrent_nodes_from_id(&self, torrent_id: i64) -> Result<Vec<(String, i64)>, Error>;

    /// Get `TorrentListing` from `torrent_id`.
    async fn get_torrent_listing_from_id(&self, torrent_id: i64) -> Result<TorrentListing, Error>;

    /// Get `TorrentListing` from `InfoHash`.
    async fn get_torrent_listing_from_info_hash(&self, info_hash: &InfoHash) -> Result<TorrentListing, Error>;

    /// Get all torrents as `Vec<TorrentCompact>`.
    async fn get_all_torrents_compact(&self) -> Result<Vec<TorrentCompact>, Error>;

    /// Get torrents whose stats have not been imported from the tracker at least since a given datetime.
    async fn get_torrents_with_stats_not_updated_since(
        &self,
        datetime: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<TorrentCompact>, Error>;

    /// Update a torrent's title with `torrent_id` and `title`.
    async fn update_torrent_title(&self, torrent_id: i64, title: &str) -> Result<(), Error>;

    /// Update a torrent's description with `torrent_id` and `description`.
    async fn update_torrent_description(&self, torrent_id: i64, description: &str) -> Result<(), Error>;

    /// Update a torrent's category with `torrent_id` and `category_id`.
    async fn update_torrent_category(&self, torrent_id: i64, category_id: CategoryId) -> Result<(), Error>;

    /// Add a new tag.
    async fn insert_tag_and_get_id(&self, name: &str) -> Result<i64, Error>;

    /// Delete a tag.
    async fn delete_tag(&self, tag_id: TagId) -> Result<(), Error>;

    /// Add a tag to torrent.
    async fn add_torrent_tag_link(&self, torrent_id: i64, tag_id: TagId) -> Result<(), Error>;

    /// Add multiple tags to a torrent at once.
    async fn add_torrent_tag_links(&self, torrent_id: i64, tag_ids: &[TagId]) -> Result<(), Error>;

    /// Remove a tag from torrent.
    async fn delete_torrent_tag_link(&self, torrent_id: i64, tag_id: TagId) -> Result<(), Error>;

    /// Remove all tags from torrent.
    async fn delete_all_torrent_tag_links(&self, torrent_id: i64) -> Result<(), Error>;

    /// Get tag from name.
    async fn get_tag_from_name(&self, name: &str) -> Result<TorrentTag, Error>;

    /// Get all tags as `Vec<TorrentTag>`.
    async fn get_tags(&self) -> Result<Vec<TorrentTag>, Error>;

    /// Get tags for `torrent_id`.
    async fn get_tags_for_torrent_id(&self, torrent_id: i64) -> Result<Vec<TorrentTag>, Error>;

    /// Update the seeders and leechers info for a torrent with `torrent_id`, `tracker_url`, `seeders` and `leechers`.
    async fn update_tracker_info(&self, torrent_id: i64, tracker_url: &Url, seeders: i64, leechers: i64) -> Result<(), Error>;

    /// Delete a torrent with `torrent_id`.
    async fn delete_torrent(&self, torrent_id: i64) -> Result<(), Error>;

    /// DELETES ALL DATABASE ROWS, ONLY CALL THIS IF YOU KNOW WHAT YOU'RE DOING!
    async fn delete_all_database_rows(&self) -> Result<(), Error>;
}
