use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::databases::sqlite::SqliteDatabase;
use crate::errors::ServiceError;
use crate::models::response::{CategoryResponse, TorrentsResponse};
use crate::models::torrent::TorrentListing;
use crate::models::tracker_key::TrackerKey;
use crate::models::user::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseDrivers {
    Sqlite3,
}

#[derive(Debug, Serialize)]
pub struct TorrentCompact {
    pub torrent_id: i64,
    pub info_hash: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub category_id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub num_torrents: i64
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum Sorting {
    uploaded_ASC,
    uploaded_DESC,
    seeders_ASC,
    seeders_DESC,
    leechers_ASC,
    leechers_DESC,
    name_ASC,
    name_DESC,
    size_ASC,
    size_DESC,
}

pub enum Error {
    Error,
    UsernameTaken,
    EmailTaken,
    UserNotFound,
    CategoryAlreadyExists,
    CategoryNotFound,
    TorrentNotFound
}

pub async fn connect_database(db_driver: &DatabaseDrivers, db_path: &str) -> Box<dyn Database> {
    match db_driver {
        DatabaseDrivers::Sqlite3 => {
            let db = SqliteDatabase::new(db_path).await;
            Box::new(db)
        }
    }
}

#[async_trait]
pub trait Database: Sync + Send {
    // add new user and get the newly inserted user_id
    async fn insert_user_and_get_id(&self, username: &str, email: &str, password: &str) -> Result<i64, Error>;

    // get user by username
    async fn get_user_from_username(&self, username: &str) -> Option<User>;

    // get a user's tracker key
    async fn get_user_tracker_key(&self, user_id: i64) -> Option<TrackerKey>;

    // count users
    async fn count_users(&self) -> Result<i64, Error>;

    // grant a user administrator rights
    async fn grant_admin_role(&self, user_id: i64) -> Result<(), Error>;

    // ban user
    async fn ban_user(&self, username: &str) -> Result<(), Error>;

    // verify email
    async fn verify_email(&self, user_id: i64) -> Result<(), Error>;

    // create a new tracker key for a certain user
    async fn issue_tracker_key(&self, tracker_key: &TrackerKey, user_id: i64) -> Result<(), ServiceError>;

    // delete user
    async fn delete_user(&self, user_id: i64) -> Result<(), sqlx::Error>;

    // add new category
    async fn insert_category(&self, category_name: &str) -> Result<(), Error>;

    // get category by id
    async fn get_category_from_id(&self, id: i64) -> Option<Category>;

    // get category by name
    async fn get_category_from_name(&self, category: &str) -> Result<Category, Error>;

    // get all categories
    async fn get_categories(&self) -> Result<Vec<CategoryResponse>, Error>;

    // delete category
    async fn delete_category(&self, category_name: &str) -> Result<(), Error>;

    // count results of given query
    async fn count_query_results(&self, query: &str) -> Result<i32, Error>;

    // get results of a torrent search in a paginated and sorted form
    async fn get_torrents_search_sorted_paginated(&self, search: &Option<String>, categories: &Option<Vec<String>>, sort: &Sorting, offset: u64, page_size: u8) -> Result<TorrentsResponse, Error>;

    // add new torrent and get the newly inserted torrent_id
    async fn insert_torrent_and_get_id(&self, username: String, info_hash: String, title: String, category_id: i64, description: String, file_size: i64, seeders: i64, leechers: i64) -> Result<i64, sqlx::Error>;

    // get torrent by id
    async fn get_torrent_from_id(&self, torrent_id: i64) -> Result<TorrentListing, ServiceError>;

    // get all torrents (torrent_id + info_hash)
    async fn get_all_torrents_compact(&self) -> Result<Vec<TorrentCompact>, ()>;

    // update a torrent's title
    async fn update_torrent_title(&self, torrent_id: i64, title: &str) -> Result<(), Error>;

    // update a torrent's description
    async fn update_torrent_description(&self, torrent_id: i64, description: &str) -> Result<(), Error>;

    // update the seeders and leechers info for a particular torrent
    async fn update_tracker_info(&self, info_hash: &str, seeders: i64, leechers: i64) -> Result<(), ()>;

    // delete a torrent
    async fn delete_torrent(&self, torrent_id: i64) -> Result<(), Error>;
}
