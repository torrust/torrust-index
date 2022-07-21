use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::databases::sqlite::SqliteDatabase;
use crate::errors::ServiceError;
use crate::models::response::{CategoryResponse, TorrentsResponse};
use crate::models::torrent::TorrentListing;
use crate::models::tracker_key::TrackerKey;
use crate::models::user::User;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseDrivers {
    Sqlite3,
}

pub async fn connect_database(db_driver: &DatabaseDrivers, db_path: &str) -> Box<dyn Database> {
    match db_driver {
        DatabaseDrivers::Sqlite3 => {
            let db = SqliteDatabase::new(db_path).await;
            Box::new(db)
        }
    }
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

#[async_trait]
pub trait Database: Sync + Send {
    // add new user and get the newly inserted user_id
    async fn insert_user_and_get_id(&self, username: &str, email: &str, password: &str) -> Result<i64, Error>;

    // grant a user administrator rights
    async fn grant_admin_role(&self, user_id: i64) -> Result<(), Error>;

    // count users
    async fn count_users(&self) -> Result<i64, Error>;

    // todo: change username param to user_id
    // verify email
    async fn verify_email(&self, username: &str) -> Result<(), Error>;

    // ban user
    async fn ban_user(&self, username: &str) -> Result<(), Error>;

    // get all categories
    async fn get_categories(&self) -> Result<Vec<CategoryResponse>, Error>;

    // add new category
    async fn insert_category(&self, category_name: &str) -> Result<(), Error>;

    // delete category
    async fn delete_category(&self, category_name: &str) -> Result<(), Error>;

    // count results of given query
    async fn count_query_results(&self, query: &str) -> Result<i32, Error>;

    // get results of a torrent search in a paginated and sorted form
    async fn get_torrents_search_sorted_paginated(&self, search: &str, categories: &Option<Vec<String>>, sort: &Sorting, offset: u64, page_size: u8) -> Result<TorrentsResponse, Error>;

    // update a torrent's title
    async fn update_torrent_title(&self, torrent_id: i64, title: &str) -> Result<(), Error>;

    // update a torrent's description
    async fn update_torrent_description(&self, torrent_id: i64, description: &str) -> Result<(), Error>;

    // delete a torrent
    async fn delete_torrent(&self, torrent_id: i64) -> Result<(), Error>;

    async fn get_user_with_username(&self, username: &str) -> Option<User>;
    async fn delete_user(&self, user_id: i64) -> Result<(), sqlx::Error>;
    async fn insert_torrent_and_get_id(&self, username: String, info_hash: String, title: String, category_id: i64, description: String, file_size: i64, seeders: i64, leechers: i64) -> Result<i64, sqlx::Error>;
    async fn get_torrent_by_id(&self, torrent_id: i64) -> Result<TorrentListing, ServiceError>;
    async fn get_all_torrent_ids(&self) -> Result<Vec<TorrentCompact>, ()>;
    async fn update_tracker_info(&self, info_hash: &str, seeders: i64, leechers: i64) -> Result<(), ()>;
    async fn get_valid_tracker_key(&self, user_id: i64) -> Option<TrackerKey>;
    async fn issue_tracker_key(&self, tracker_key: &TrackerKey, user_id: i64) -> Result<(), ServiceError>;
    async fn get_category(&self, id: i64) -> Option<Category>;
    async fn get_category_by_name(&self, category: &str) -> Result<Category, Error>;
}
