use std::borrow::Cow;
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use async_trait::async_trait;

use crate::models::user::User;
use crate::errors::ServiceError;
use crate::models::torrent::TorrentListing;
use crate::utils::time::current_time;
use crate::models::tracker_key::TrackerKey;
use crate::databases::database::{Category, Database, Error, Sorting, TorrentCompact};
use crate::handlers::torrent::TorrentCount;
use crate::models::response::{CategoryResponse, TorrentsResponse};

pub struct SqliteDatabase {
    pub pool: SqlitePool
}

impl SqliteDatabase {
    pub async fn new(database_url: &str) -> Self {
        let db = SqlitePoolOptions::new()
            .connect(database_url)
            .await
            .expect("Unable to create database pool.");

        // create/update database tables
        let _ = sqlx::migrate!().run(&db).await.expect("Could not create/update database.");

        Self {
            pool: db
        }
    }
}

#[async_trait]
impl Database for SqliteDatabase {
    async fn insert_user_and_get_id(&self, username: &str, email: &str, password_hash: &str) -> Result<i64, Error> {
        let res = sqlx::query!(
            "INSERT INTO torrust_users (username, email, password) VALUES ($1, $2, $3)",
            username,
            email,
            password_hash,
        )
            .execute(&self.pool)
            .await;

        match res {
            Err(sqlx::Error::Database(err)) => {
                return if err.code() == Some(Cow::from("2067")) {
                    if err.message().contains("torrust_users.username") {
                        Err(Error::UsernameTaken)
                    } else if err.message().contains("torrust_users.email") {
                        Err(Error::EmailTaken)
                    } else {
                        Err(Error::Error)
                    }
                } else {
                    Err(Error::Error)
                };
            },
            Err(_) => Err(Error::Error),
            Ok(v) => Ok(v.last_insert_rowid())
        }
    }

    async fn get_user_from_username(&self, username: &str) -> Option<User> {
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

    async fn get_user_tracker_key(&self, user_id: i64) -> Option<TrackerKey> {
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

    async fn count_users(&self) -> Result<i64, Error> {
        let res: Result<(i64,), Error> = sqlx::query_as("SELECT COUNT(*) FROM torrust_users")
            .fetch_one(&self.pool)
            .await
            .map_err(|_| Error::Error);

        match res {
            Err(_) => Err(Error::Error),
            Ok((v,)) => Ok(v)
        }
    }

    async fn grant_admin_role(&self, user_id: i64) -> Result<(), Error> {
        let res = sqlx::query!(
            "UPDATE torrust_users SET administrator = 1 WHERE user_id = ?",
            user_id
        )
            .execute(&self.pool)
            .await;

        match res {
            Err(_) => Err(Error::Error),
            Ok(_) => Ok(())
        }
    }

    async fn ban_user(&self, username: &str) -> Result<(), Error> {
        let res = sqlx::query!(
            "DELETE FROM torrust_users WHERE username = ? AND administrator = 0",
            username
        )
            .execute(&self.pool)
            .await;

        match res {
            Err(_) => Err(Error::UserNotFound),
            Ok(v) => {
                if v.rows_affected() == 0 {
                    Err(Error::Error)
                } else {
                    Ok(())
                }
            }
        }
    }

    async fn verify_email(&self, user_id: i64) -> Result<(), Error> {
        let res = sqlx::query!(
            "UPDATE torrust_users SET email_verified = TRUE WHERE user_id = ?",
            user_id
        )
            .execute(&self.pool)
            .await;

        match res {
            Err(_) => Err(Error::Error),
            Ok(_) => Ok(())
        }
    }

    async fn issue_tracker_key(&self, tracker_key: &TrackerKey, user_id: i64) -> Result<(), ServiceError> {
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

    async fn delete_user(&self, user_id: i64) -> Result<(), sqlx::Error> {
        let _res = sqlx::query!(
            "DELETE FROM torrust_users WHERE rowid = ?",
            user_id
        )
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn insert_category(&self, category_name: &str) -> Result<(), Error> {
        let res = sqlx::query!(
            "INSERT INTO torrust_categories (name) VALUES ($1)",
            category_name,
        )
            .execute(&self.pool)
            .await;

        if let Err(sqlx::Error::Database(err)) = res {
            return if err.message().contains("UNIQUE") {
                Err(Error::CategoryAlreadyExists)
            } else {
                Err(Error::Error)
            }
        }

        Ok(())
    }

    async fn get_category_from_id(&self, id: i64) -> Option<Category> {
        let res = sqlx::query_as!(
            Category,
            "SELECT category_id, name, icon, (SELECT COUNT(*) FROM torrust_torrents WHERE torrust_torrents.category_id = torrust_categories.category_id) AS num_torrents FROM torrust_categories WHERE category_id = ?",
            id
        )
            .fetch_one(&self.pool)
            .await;

        match res {
            Ok(v) => Some(v),
            Err(_) => None
        }
    }

    async fn get_category_from_name(&self, category: &str) -> Result<Category, Error> {
        let res = sqlx::query_as!(
            Category,
            "SELECT category_id, name, icon, (SELECT COUNT(*) FROM torrust_torrents WHERE torrust_torrents.category_id = torrust_categories.category_id) AS num_torrents FROM torrust_categories WHERE name = ?",
            category
        )
            .fetch_one(&self.pool)
            .await;

        match res {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::CategoryNotFound)
        }
    }

    async fn get_categories(&self) -> Result<Vec<CategoryResponse>, Error> {
        let res = sqlx::query_as::<_, CategoryResponse>(
            r#"SELECT tc.category_id, tc.name, tc.icon, COUNT(tt.category_id) as num_torrents
           FROM torrust_categories tc
           LEFT JOIN torrust_torrents tt on tc.category_id = tt.category_id
           GROUP BY tc.name"#
        )
            .fetch_all(&self.pool)
            .await;

        match res {
            Err(_) => Err(Error::Error),
            Ok(v) => Ok(v)
        }
    }

    async fn delete_category(&self, category_name: &str) -> Result<(), Error> {
        let res = sqlx::query!(
            "DELETE FROM torrust_categories WHERE name = $1",
            category_name,
        )
            .execute(&self.pool)
            .await;

        match res {
            Err(_) => Err(Error::Error),
            Ok(_) => Ok(())
        }
    }

    async fn count_query_results(&self, query: &str) -> Result<i32, Error> {
        let count_query = format!("SELECT COUNT(*) as count FROM ({})", query);

        let res: Result<(i32,), Error> = sqlx::query_as(&count_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| Error::Error);

        match res {
            Err(_) => Err(Error::Error),
            Ok((v,)) => Ok(v)
        }
    }

    // todo: refactor this
    async fn get_torrents_search_sorted_paginated(&self, search: &Option<String>, categories: &Option<Vec<String>>, sort: &Sorting, offset: u64, page_size: u8) -> Result<TorrentsResponse, Error> {
        let title = match search {
            None => "%".to_string(),
            Some(v) => format!("%{}%", v)
        };

        let sort_query: String = match sort {
            Sorting::uploaded_ASC => "upload_date ASC".to_string(),
            Sorting::uploaded_DESC => "upload_date DESC".to_string(),
            Sorting::seeders_ASC => "seeders ASC".to_string(),
            Sorting::seeders_DESC => "seeders DESC".to_string(),
            Sorting::leechers_ASC => "leechers ASC".to_string(),
            Sorting::leechers_DESC => "leechers DESC".to_string(),
            Sorting::name_ASC => "title ASC".to_string(),
            Sorting::name_DESC => "title DESC".to_string(),
            Sorting::size_ASC => "file_size ASC".to_string(),
            Sorting::size_DESC => "file_size DESC".to_string(),
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
                format!("INNER JOIN torrust_categories tc ON tt.category_id = tc.category_id AND ({})", category_filters)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let mut query_string = format!("SELECT tt.* FROM torrust_torrents tt {} WHERE title LIKE ?", category_filter_query);

        let count: TorrentCount = self.count_query_results(&query_string)
            .await
            .map(|count| TorrentCount { count })?;

        query_string = format!("{} ORDER BY {} LIMIT ?, ?", query_string, sort_query);

        let res: Vec<TorrentListing> = sqlx::query_as::<_, TorrentListing>(&query_string)
            .bind(title)
            .bind(offset as i64)
            .bind(page_size)
            .fetch_all(&self.pool)
            .await
            .map_err(|_| Error::Error)?;

        Ok(TorrentsResponse {
            total: count.count as u32,
            results: res
        })
    }

    async fn insert_torrent_and_get_id(&self, username: String, info_hash: String, title: String, category_id: i64, description: String, file_size: i64, seeders: i64, leechers: i64) -> Result<i64, sqlx::Error> {
        let current_time = current_time() as i64;

        let res = sqlx::query!(
            r#"INSERT INTO torrust_torrents (uploader, info_hash, title, category_id, description, upload_date, file_size, seeders, leechers)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING torrent_id as "torrent_id: i64""#,
            username,
            info_hash,
            title,
            category_id,
            description,
            current_time,
            file_size,
            seeders,
            leechers
        )
            .fetch_one(&self.pool)
            .await?;

        Ok(res.torrent_id)
    }

    async fn get_torrent_from_id(&self, torrent_id: i64) -> Result<TorrentListing, ServiceError> {
        let res = sqlx::query_as!(
            TorrentListing,
            r#"SELECT * FROM torrust_torrents
               WHERE torrent_id = ?"#,
            torrent_id
        )
            .fetch_one(&self.pool)
            .await;

        match res {
            Ok(torrent) => Ok(torrent),
            _ => Err(ServiceError::TorrentNotFound)
        }
    }

    async fn get_all_torrents_compact(&self) -> Result<Vec<TorrentCompact>, ()> {
        let res = sqlx::query_as!(
            TorrentCompact,
            r#"SELECT torrent_id, info_hash FROM torrust_torrents"#
        )
            .fetch_all(&self.pool)
            .await;

        match res {
            Ok(torrents) => Ok(torrents),
            Err(_) => Err(())
        }
    }

    async fn update_torrent_title(&self, torrent_id: i64, title: &str) -> Result<(), Error> {
        let res = sqlx::query!(
            "UPDATE torrust_torrents SET title = $1 WHERE torrent_id = $2",
            title,
            torrent_id
        )
            .execute(&self.pool)
            .await;

        match res {
            Ok(v) => {
                if v.rows_affected() == 0 {
                    Err(Error::TorrentNotFound)
                } else {
                    Ok(())
                }
            },
            Err(_) => Err(Error::Error)
        }
    }

    async fn update_torrent_description(&self, torrent_id: i64, description: &str) -> Result<(), Error> {
        let res = sqlx::query!(
            "UPDATE torrust_torrents SET description = $1 WHERE torrent_id = $2",
            description,
            torrent_id
        )
            .execute(&self.pool)
            .await;

        match res {
            Ok(v) => {
                if v.rows_affected() == 0 {
                    Err(Error::TorrentNotFound)
                } else {
                    Ok(())
                }
            },
            Err(_) => Err(Error::Error)
        }
    }

    async fn update_tracker_info(&self, info_hash: &str, seeders: i64, leechers: i64) -> Result<(), ()> {
        let res = sqlx::query!(
            "UPDATE torrust_torrents SET seeders = $1, leechers = $2 WHERE info_hash = $3",
            seeders,
            leechers,
            info_hash
        )
            .execute(&self.pool)
            .await;

        match res {
            Ok(_) => Ok(()),
            _ => Err(())
        }
    }

    async fn delete_torrent(&self, torrent_id: i64) -> Result<(), Error> {
        let res = sqlx::query!(
            "DELETE FROM torrust_torrents WHERE torrent_id = ?",
            torrent_id
        )
            .execute(&self.pool)
            .await;

        match res {
            Ok(v) => {
                if v.rows_affected() == 0 {
                    Err(Error::TorrentNotFound)
                } else {
                    Ok(())
                }
            },
            Err(_) => Err(Error::Error)
        }
    }
}
