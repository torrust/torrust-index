use sqlx::{Acquire, MySqlPool, query, query_as};
use async_trait::async_trait;
use chrono::{NaiveDateTime};
use sqlx::mysql::MySqlPoolOptions;

use crate::models::user::{User, UserAuthentication, UserCompact, UserProfile};
use crate::models::torrent::TorrentListing;
use crate::utils::time::current_time;
use crate::models::tracker_key::TrackerKey;
use crate::databases::database::{Category, Database, DatabaseError, Sorting, TorrentCompact};
use crate::handlers::torrent::TorrentCount;
use crate::models::response::{TorrentsResponse};

pub struct MysqlDatabase {
    pub pool: MySqlPool
}

impl MysqlDatabase {
    pub async fn new(database_url: &str) -> Self {
        let db = MySqlPoolOptions::new()
            .connect(database_url)
            .await
            .expect("Unable to create database pool.");

        sqlx::migrate!("migrations/mysql")
            .run(&db)
            .await
            .expect("Could not run database migrations.");

        Self {
            pool: db
        }
    }
}

#[async_trait]
impl Database for MysqlDatabase {
    async fn insert_user_and_get_id(&self, username: &str, email: &str, password_hash: &str) -> Result<i64, DatabaseError> {

        // open pool connection
        let mut conn = self.pool.acquire()
            .await
            .map_err(|_| DatabaseError::Error)?;

        // start db transaction
        let mut tx = conn.begin()
            .await
            .map_err(|_| DatabaseError::Error)?;

        // create the user account and get the user id
        let user_id = query("INSERT INTO torrust_users (date_registered) VALUES (UTC_TIMESTAMP())")
            .execute(&mut tx)
            .await
            .map(|v| v.last_insert_id())
            .map_err(|_| DatabaseError::Error)?;

        // add password hash for account
        let insert_user_auth_result = query("INSERT INTO torrust_user_authentication (user_id, password_hash) VALUES (?, ?)")
            .bind(user_id)
            .bind(password_hash)
            .execute(&mut tx)
            .await
            .map_err(|_| DatabaseError::Error);

        // rollback transaction on error
        if let Err(e) = insert_user_auth_result {
            let _ = tx.rollback().await;
            return Err(e)
        }

        // add account profile details
        let insert_user_profile_result = query(r#"INSERT INTO torrust_user_profiles (user_id, username, email, email_verified, bio, avatar) VALUES (?, ?, NULLIF(?, ""), 0, NULL, NULL)"#)
            .bind(user_id)
            .bind(username)
            .bind(email)
            .execute(&mut tx)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(err) => {
                    if err.message().contains("username") {
                        DatabaseError::UsernameTaken
                    } else if err.message().contains("email") {
                        DatabaseError::EmailTaken
                    } else {
                        DatabaseError::Error
                    }
                }
                _ => DatabaseError::Error
            });

        // commit or rollback transaction and return user_id on success
        match insert_user_profile_result {
            Ok(_) => {
                let _ = tx.commit().await;
                Ok(user_id as i64)
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Err(e)
            }
        }
    }

    async fn get_user_from_id(&self, user_id: i64) -> Result<User, DatabaseError> {
        query_as::<_, User>("SELECT * FROM torrust_users WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| DatabaseError::UserNotFound)
    }

    async fn get_user_authentication_from_id(&self, user_id: i64) -> Result<UserAuthentication, DatabaseError> {
        query_as::<_, UserAuthentication>("SELECT * FROM torrust_user_authentication WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| DatabaseError::UserNotFound)
    }

    async fn get_user_profile_from_username(&self, username: &str) -> Result<UserProfile, DatabaseError> {
        query_as::<_, UserProfile>(r#"SELECT user_id, username, COALESCE(email, "") as email, email_verified, COALESCE(bio, "") as bio, COALESCE(avatar, "") as avatar FROM torrust_user_profiles WHERE username = ?"#)
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| DatabaseError::UserNotFound)
    }

    async fn get_user_compact_from_id(&self, user_id: i64) -> Result<UserCompact, DatabaseError> {
        query_as::<_, UserCompact>("SELECT tu.user_id, tp.username, tu.administrator FROM torrust_users tu INNER JOIN torrust_user_profiles tp ON tu.user_id = tp.user_id WHERE tu.user_id = ?")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| DatabaseError::UserNotFound)
    }

    async fn get_user_tracker_key(&self, user_id: i64) -> Option<TrackerKey> {
        const HOUR_IN_SECONDS: i64 = 3600;

        // casting current_time() to i64 will overflow in the year 2262
        let current_time_plus_hour = (current_time() as i64) + HOUR_IN_SECONDS;

        // get tracker key that is valid for at least one hour from now
        query_as::<_, TrackerKey>("SELECT tracker_key, date_expiry FROM torrust_tracker_keys WHERE user_id = ? AND date_expiry > ? ORDER BY date_expiry DESC")
            .bind(user_id)
            .bind(current_time_plus_hour)
            .fetch_one(&self.pool)
            .await
            .ok()
    }

    async fn count_users(&self) -> Result<i64, DatabaseError> {
        query_as("SELECT COUNT(*) FROM torrust_users")
            .fetch_one(&self.pool)
            .await
            .map(|(v,)| v)
            .map_err(|_| DatabaseError::Error)
    }

    async fn ban_user(&self, user_id: i64, reason: &str, date_expiry: NaiveDateTime) -> Result<(), DatabaseError> {
        // date needs to be in ISO 8601 format
        let date_expiry_string = date_expiry.format("%Y-%m-%d %H:%M:%S").to_string();

        query("INSERT INTO torrust_user_bans (user_id, reason, date_expiry) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(reason)
            .bind(date_expiry_string)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| DatabaseError::Error)
    }

    async fn grant_admin_role(&self, user_id: i64) -> Result<(), DatabaseError> {
        query("UPDATE torrust_users SET administrator = TRUE WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|_| DatabaseError::Error)
            .and_then(|v| if v.rows_affected() > 0 {
                Ok(())
            } else {
                Err(DatabaseError::UserNotFound)
            })
    }

    async fn verify_email(&self, user_id: i64) -> Result<(), DatabaseError> {
        query("UPDATE torrust_user_profiles SET email_verified = TRUE WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|_| DatabaseError::Error)
            .and_then(|v| if v.rows_affected() > 0 {
                Ok(())
            } else {
                Err(DatabaseError::UserNotFound)
            })
    }

    async fn add_tracker_key(&self, user_id: i64, tracker_key: &TrackerKey) -> Result<(), DatabaseError> {
        let key = tracker_key.key.clone();

        // date needs to be in ISO 8601 format
        let date_expiry = NaiveDateTime::from_timestamp(tracker_key.valid_until, 0)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        query("INSERT INTO torrust_tracker_keys (user_id, tracker_key, date_expiry) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(key)
            .bind(date_expiry)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| DatabaseError::Error)
    }

    async fn delete_user(&self, user_id: i64) -> Result<(), DatabaseError> {
        query("DELETE FROM torrust_users WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|_| DatabaseError::Error)
            .and_then(|v| if v.rows_affected() > 0 {
                Ok(())
            } else {
                Err(DatabaseError::UserNotFound)
            })
    }

    async fn add_category(&self, category_name: &str) -> Result<(), DatabaseError> {
        query("INSERT INTO torrust_categories (name) VALUES (?)")
            .bind(category_name)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| match e {
                sqlx::Error::Database(err) => {
                    if err.message().contains("UNIQUE") {
                        DatabaseError::CategoryAlreadyExists
                    } else {
                        DatabaseError::Error
                    }
                },
                _ => DatabaseError::Error
            })
    }

    async fn get_category_from_id(&self, category_id: i64) -> Result<Category, DatabaseError> {
        query_as::<_, Category>("SELECT category_id, name, (SELECT COUNT(*) FROM torrust_torrents WHERE torrust_torrents.category_id = torrust_categories.category_id) AS num_torrents FROM torrust_categories WHERE category_id = ?")
            .bind(category_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| DatabaseError::CategoryNotFound)
    }

    async fn get_category_from_name(&self, category_name: &str) -> Result<Category, DatabaseError> {
        query_as::<_, Category>("SELECT category_id, name, (SELECT COUNT(*) FROM torrust_torrents WHERE torrust_torrents.category_id = torrust_categories.category_id) AS num_torrents FROM torrust_categories WHERE name = ?")
            .bind(category_name)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| DatabaseError::CategoryNotFound)
    }

    async fn get_categories(&self) -> Result<Vec<Category>, DatabaseError> {
        query_as::<_, Category>("SELECT tc.category_id, tc.name, COUNT(tt.category_id) as num_torrents FROM torrust_categories tc LEFT JOIN torrust_torrents tt on tc.category_id = tt.category_id GROUP BY tc.name")
            .fetch_all(&self.pool)
            .await
            .map_err(|_| DatabaseError::Error)
    }

    async fn delete_category(&self, category_name: &str) -> Result<(), DatabaseError> {
        query("DELETE FROM torrust_categories WHERE name = ?")
            .bind(category_name)
            .execute(&self.pool)
            .await
            .map_err(|_| DatabaseError::Error)
            .and_then(|v| if v.rows_affected() > 0 {
                Ok(())
            } else {
                Err(DatabaseError::CategoryNotFound)
            })
    }

    // todo: refactor this
    async fn get_torrents_search_sorted_paginated(&self, search: &Option<String>, categories: &Option<Vec<String>>, sort: &Sorting, offset: u64, page_size: u8) -> Result<TorrentsResponse, DatabaseError> {
        let title = match search {
            None => "%".to_string(),
            Some(v) => format!("%{}%", v)
        };

        let sort_query: String = match sort {
            Sorting::UploadedAsc => "upload_date ASC".to_string(),
            Sorting::UploadedDesc => "upload_date DESC".to_string(),
            Sorting::SeedersAsc => "seeders ASC".to_string(),
            Sorting::SeedersDesc => "seeders DESC".to_string(),
            Sorting::LeechersAsc => "leechers ASC".to_string(),
            Sorting::LeechersDesc => "leechers DESC".to_string(),
            Sorting::NameAsc => "title ASC".to_string(),
            Sorting::NameDesc => "title DESC".to_string(),
            Sorting::SizeAsc => "file_size ASC".to_string(),
            Sorting::SizeDesc => "file_size DESC".to_string(),
        };

        let category_filter_query = if let Some(c) = categories {
            let mut i = 0;
            let mut category_filters = String::new();
            for category in c.iter() {
                // don't take user input in the db query
                if let Ok(sanitized_category) = self.get_category_from_name(category).await {
                    let mut str = format!("tc.name = '{}'", sanitized_category.name);
                    if i > 0 { str = format!(" OR {}", str); }
                    category_filters.push_str(&str);
                    i += 1;
                }
            }
            if category_filters.len() > 0 {
                format!("INNER JOIN torrust_categories tc ON tt.category_id = tc.category_id AND ({}) ", category_filters)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let mut query_string = format!("SELECT tt.* FROM torrust_torrents tt {}WHERE title LIKE ?", category_filter_query);

        let count_query = format!("SELECT COUNT(*) as count FROM ({}) AS count_table", query_string);

        let count_result: Result<i64, DatabaseError> = query_as(&count_query)
            .bind(title.clone())
            .fetch_one(&self.pool)
            .await
            .map(|(v,)| v)
            .map_err(|_| DatabaseError::Error);

        let count = count_result?;

        query_string = format!("{} ORDER BY {} LIMIT ?, ?", query_string, sort_query);

        let res: Vec<TorrentListing> = sqlx::query_as::<_, TorrentListing>(&query_string)
            .bind(title)
            .bind(offset as i64)
            .bind(page_size)
            .fetch_all(&self.pool)
            .await
            .map_err(|_| DatabaseError::Error)?;

        Ok(TorrentsResponse {
            total: count as u32,
            results: res
        })
    }

    async fn insert_torrent_and_get_id(&self, username: String, info_hash: String, title: String, category_id: i64, description: String, file_size: i64, seeders: i64, leechers: i64) -> Result<i64, DatabaseError> {
        let current_time = current_time() as i64;

        query(r#"INSERT INTO torrust_torrents (uploader, info_hash, title, category_id, description, upload_date, file_size, seeders, leechers) VALUES (?, ?, ?, NULLIF(?, ""), ?, ?, ?, ?, ?)"#)
            .bind(username)
            .bind(info_hash)
            .bind(title)
            .bind(category_id)
            .bind(description)
            .bind(current_time)
            .bind(file_size)
            .bind(seeders)
            .bind(leechers)
            .execute(&self.pool)
            .await
            .map(|v| v.last_insert_id() as i64)
            .map_err(|e| match e {
                sqlx::Error::Database(err) => {
                    if err.message().contains("info_hash") {
                        DatabaseError::TorrentAlreadyExists
                    } else if err.message().contains("title") {
                        DatabaseError::TorrentTitleAlreadyExists
                    } else {
                        DatabaseError::Error
                    }
                }
                _ => DatabaseError::Error
            })
    }

    async fn get_torrent_from_id(&self, torrent_id: i64) -> Result<TorrentListing, DatabaseError> {
        query_as::<_, TorrentListing>("SELECT * FROM torrust_torrents WHERE torrent_id = ?")
            .bind(torrent_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| DatabaseError::TorrentNotFound)
    }

    async fn get_all_torrents_compact(&self) -> Result<Vec<TorrentCompact>, DatabaseError> {
        query_as::<_, TorrentCompact>("SELECT torrent_id, info_hash FROM torrust_torrents")
            .fetch_all(&self.pool)
            .await
            .map_err(|_| DatabaseError::Error)
    }

    async fn update_torrent_title(&self, torrent_id: i64, title: &str) -> Result<(), DatabaseError> {
        query("UPDATE torrust_torrents SET title = ? WHERE torrent_id = ?")
            .bind(title)
            .bind(torrent_id)
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(err) => {
                    if err.message().contains("UNIQUE") {
                        DatabaseError::TorrentTitleAlreadyExists
                    } else {
                        DatabaseError::Error
                    }
                }
                _ => DatabaseError::Error
            })
            .and_then(|v| if v.rows_affected() > 0 {
                Ok(())
            } else {
                Err(DatabaseError::TorrentNotFound)
            })
    }

    async fn update_torrent_description(&self, torrent_id: i64, description: &str) -> Result<(), DatabaseError> {
        query("UPDATE torrust_torrents SET description = ? WHERE torrent_id = ?")
            .bind(description)
            .bind(torrent_id)
            .execute(&self.pool)
            .await
            .map_err(|_| DatabaseError::Error)
            .and_then(|v| if v.rows_affected() > 0 {
                Ok(())
            } else {
                Err(DatabaseError::TorrentNotFound)
            })
    }

    async fn update_tracker_info(&self, info_hash: &str, seeders: i64, leechers: i64) -> Result<(), DatabaseError> {
        query("UPDATE torrust_torrents SET seeders = ?, leechers = ? WHERE info_hash = ?")
            .bind(seeders)
            .bind(leechers)
            .bind(info_hash)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| DatabaseError::TorrentNotFound)
    }

    async fn delete_torrent(&self, torrent_id: i64) -> Result<(), DatabaseError> {
        query("DELETE FROM torrust_torrents WHERE torrent_id = ?")
            .bind(torrent_id)
            .execute(&self.pool)
            .await
            .map_err(|_| DatabaseError::Error)
            .and_then(|v| if v.rows_affected() > 0 {
                Ok(())
            } else {
                Err(DatabaseError::TorrentNotFound)
            })
    }
}
