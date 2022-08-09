use async_trait::async_trait;
use chrono::{NaiveDateTime};
use serde::{Serialize, Deserialize};
use crate::databases::mysql::MysqlDatabase;
use crate::databases::sqlite::SqliteDatabase;
use crate::models::response::{TorrentsResponse};
use crate::models::torrent::TorrentListing;
use crate::models::tracker_key::TrackerKey;
use crate::models::user::{User, UserAuthentication, UserCompact, UserProfile};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseDriver {
    Sqlite3,
    Mysql
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TorrentCompact {
    pub torrent_id: i64,
    pub info_hash: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub category_id: i64,
    pub name: String,
    pub num_torrents: i64
}

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

#[derive(Debug)]
pub enum DatabaseError {
    Error,
    UsernameTaken,
    EmailTaken,
    UserNotFound,
    CategoryAlreadyExists,
    CategoryNotFound,
    TorrentNotFound,
    TorrentAlreadyExists, // when uploading an already uploaded info_hash
    TorrentTitleAlreadyExists,
}

pub async fn connect_database(db_driver: &DatabaseDriver, db_path: &str) -> Box<dyn Database> {
    // match &db_path.chars().collect::<Vec<char>>() as &[char] {
    //     ['s', 'q', 'l', 'i', 't', 'e', ..] => {
    //         let db = SqliteDatabase::new(db_path).await;
    //         Ok(Box::new(db))
    //     }
    //     ['m', 'y', 's', 'q', 'l', ..] => {
    //         let db = MysqlDatabase::new(db_path).await;
    //         Ok(Box::new(db))
    //     }
    //     _ => {
    //         Err(())
    //     }
    // }

    match db_driver {
        DatabaseDriver::Sqlite3 => {
            let db = SqliteDatabase::new(db_path).await;
            Box::new(db)
        }
        DatabaseDriver::Mysql => {
            let db = MysqlDatabase::new(db_path).await;
            Box::new(db)
        }
    }
}

#[async_trait]
pub trait Database: Sync + Send {
    // return current database driver
    fn get_database_driver(&self) -> DatabaseDriver;

    // add new user and get the newly inserted user_id
    async fn insert_user_and_get_id(&self, username: &str, email: &str, password: &str) -> Result<i64, DatabaseError>;

    // get user profile by user_id
    async fn get_user_from_id(&self, user_id: i64) -> Result<User, DatabaseError>;

    // get user authentication by user_id
    async fn get_user_authentication_from_id(&self, user_id: i64) -> Result<UserAuthentication, DatabaseError>;

    // get user profile by username
    async fn get_user_profile_from_username(&self, username: &str) -> Result<UserProfile, DatabaseError>;

    // get user compact by user_id
    async fn get_user_compact_from_id(&self, user_id: i64) -> Result<UserCompact, DatabaseError>;

    // todo: change to get all tracker keys of user, no matter if they are still valid
    // get a user's tracker key
    async fn get_user_tracker_key(&self, user_id: i64) -> Option<TrackerKey>;

    // count users
    async fn count_users(&self) -> Result<i64, DatabaseError>;

    // todo: make DateTime struct for the date_expiry
    // ban user
    async fn ban_user(&self, user_id: i64, reason: &str, date_expiry: NaiveDateTime) -> Result<(), DatabaseError>;

    // give a user administrator rights
    async fn grant_admin_role(&self, user_id: i64) -> Result<(), DatabaseError>;

    // verify email
    async fn verify_email(&self, user_id: i64) -> Result<(), DatabaseError>;

    // create a new tracker key for a certain user
    async fn add_tracker_key(&self, user_id: i64, tracker_key: &TrackerKey) -> Result<(), DatabaseError>;

    // delete user
    async fn delete_user(&self, user_id: i64) -> Result<(), DatabaseError>;

    // add new category
    async fn add_category(&self, category_name: &str) -> Result<(), DatabaseError>;

    // get category by id
    async fn get_category_from_id(&self, id: i64) -> Result<Category, DatabaseError>;

    // get category by name
    async fn get_category_from_name(&self, category: &str) -> Result<Category, DatabaseError>;

    // get all categories
    async fn get_categories(&self) -> Result<Vec<Category>, DatabaseError>;

    // delete category
    async fn delete_category(&self, category_name: &str) -> Result<(), DatabaseError>;

    // get results of a torrent search in a paginated and sorted form
    async fn get_torrents_search_sorted_paginated(&self, search: &Option<String>, categories: &Option<Vec<String>>, sort: &Sorting, offset: u64, page_size: u8) -> Result<TorrentsResponse, DatabaseError>;

    // add new torrent and get the newly inserted torrent_id
    async fn insert_torrent_and_get_id(&self, username: String, info_hash: String, title: String, category_id: i64, description: String, file_size: i64, seeders: i64, leechers: i64) -> Result<i64, DatabaseError>;

    // get torrent by id
    async fn get_torrent_from_id(&self, torrent_id: i64) -> Result<TorrentListing, DatabaseError>;

    // get all torrents (torrent_id + info_hash)
    async fn get_all_torrents_compact(&self) -> Result<Vec<TorrentCompact>, DatabaseError>;

    // update a torrent's title
    async fn update_torrent_title(&self, torrent_id: i64, title: &str) -> Result<(), DatabaseError>;

    // update a torrent's description
    async fn update_torrent_description(&self, torrent_id: i64, description: &str) -> Result<(), DatabaseError>;

    // update the seeders and leechers info for a particular torrent
    async fn update_tracker_info(&self, info_hash: &str, seeders: i64, leechers: i64) -> Result<(), DatabaseError>;

    // delete a torrent
    async fn delete_torrent(&self, torrent_id: i64) -> Result<(), DatabaseError>;
}
