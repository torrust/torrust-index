use std::str::FromStr;
use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{query, query_as, Acquire, ConnectOptions, SqlitePool};

use super::database::TABLES_TO_TRUNCATE;
use crate::databases::database;
use crate::databases::database::{Category, Database, Driver, Sorting, TorrentCompact};
use crate::models::category::CategoryId;
use crate::models::info_hash::InfoHash;
use crate::models::response::TorrentsResponse;
use crate::models::torrent::TorrentListing;
use crate::models::torrent_file::{DbTorrentAnnounceUrl, DbTorrentFile, DbTorrentInfo, Torrent, TorrentFile};
use crate::models::torrent_tag::{TagId, TorrentTag};
use crate::models::tracker_key::TrackerKey;
use crate::models::user::{User, UserAuthentication, UserCompact, UserId, UserProfile};
use crate::utils::clock;
use crate::utils::hex::from_bytes;

pub struct Sqlite {
    pub pool: SqlitePool,
}

#[async_trait]
impl Database for Sqlite {
    fn get_database_driver(&self) -> Driver {
        Driver::Sqlite3
    }

    async fn new(database_url: &str) -> Self {
        let mut connection_options = SqliteConnectOptions::from_str(database_url).expect("Unable to create connection options.");
        connection_options
            .log_statements(log::LevelFilter::Error)
            .log_slow_statements(log::LevelFilter::Warn, Duration::from_secs(1));

        let db = SqlitePoolOptions::new()
            .connect_with(connection_options)
            .await
            .expect("Unable to create database pool.");

        sqlx::migrate!("migrations/sqlite3")
            .run(&db)
            .await
            .expect("Could not run database migrations.");

        Self { pool: db }
    }

    async fn insert_user_and_get_id(&self, username: &str, email: &str, password_hash: &str) -> Result<i64, database::Error> {
        // open pool connection
        let mut conn = self.pool.acquire().await.map_err(|_| database::Error::Error)?;

        // start db transaction
        let mut tx = conn.begin().await.map_err(|_| database::Error::Error)?;

        // create the user account and get the user id
        let user_id =
            query("INSERT INTO torrust_users (date_registered) VALUES (strftime('%Y-%m-%d %H:%M:%S',DATETIME('now', 'utc')))")
                .execute(&mut tx)
                .await
                .map(|v| v.last_insert_rowid())
                .map_err(|_| database::Error::Error)?;

        // add password hash for account
        let insert_user_auth_result = query("INSERT INTO torrust_user_authentication (user_id, password_hash) VALUES (?, ?)")
            .bind(user_id)
            .bind(password_hash)
            .execute(&mut tx)
            .await
            .map_err(|_| database::Error::Error);

        // rollback transaction on error
        if let Err(e) = insert_user_auth_result {
            drop(tx.rollback().await);
            return Err(e);
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
                        database::Error::UsernameTaken
                    } else if err.message().contains("email") {
                        database::Error::EmailTaken
                    } else {
                        database::Error::Error
                    }
                }
                _ => database::Error::Error
            });

        // commit or rollback transaction and return user_id on success
        match insert_user_profile_result {
            Ok(_) => {
                drop(tx.commit().await);
                Ok(user_id)
            }
            Err(e) => {
                drop(tx.rollback().await);
                Err(e)
            }
        }
    }

    async fn get_user_from_id(&self, user_id: i64) -> Result<User, database::Error> {
        query_as::<_, User>("SELECT * FROM torrust_users WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| database::Error::UserNotFound)
    }

    async fn get_user_authentication_from_id(&self, user_id: UserId) -> Result<UserAuthentication, database::Error> {
        query_as::<_, UserAuthentication>("SELECT * FROM torrust_user_authentication WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| database::Error::UserNotFound)
    }

    async fn get_user_profile_from_username(&self, username: &str) -> Result<UserProfile, database::Error> {
        query_as::<_, UserProfile>("SELECT * FROM torrust_user_profiles WHERE username = ?")
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| database::Error::UserNotFound)
    }

    async fn get_user_compact_from_id(&self, user_id: i64) -> Result<UserCompact, database::Error> {
        query_as::<_, UserCompact>("SELECT tu.user_id, tp.username, tu.administrator FROM torrust_users tu INNER JOIN torrust_user_profiles tp ON tu.user_id = tp.user_id WHERE tu.user_id = ?")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| database::Error::UserNotFound)
    }

    async fn get_user_tracker_key(&self, user_id: i64) -> Option<TrackerKey> {
        const HOUR_IN_SECONDS: i64 = 3600;

        // casting current_time() to i64 will overflow in the year 2262
        let current_time_plus_hour = i64::try_from(clock::now()).unwrap().saturating_add(HOUR_IN_SECONDS);

        // get tracker key that is valid for at least one hour from now
        query_as::<_, TrackerKey>("SELECT tracker_key AS key, date_expiry AS valid_until FROM torrust_tracker_keys WHERE user_id = $1 AND date_expiry > $2 ORDER BY date_expiry DESC")
            .bind(user_id)
            .bind(current_time_plus_hour)
            .fetch_one(&self.pool)
            .await
            .ok()
    }

    async fn count_users(&self) -> Result<i64, database::Error> {
        query_as("SELECT COUNT(*) FROM torrust_users")
            .fetch_one(&self.pool)
            .await
            .map(|(v,)| v)
            .map_err(|_| database::Error::Error)
    }

    async fn ban_user(&self, user_id: i64, reason: &str, date_expiry: NaiveDateTime) -> Result<(), database::Error> {
        // date needs to be in ISO 8601 format
        let date_expiry_string = date_expiry.format("%Y-%m-%d %H:%M:%S").to_string();

        query("INSERT INTO torrust_user_bans (user_id, reason, date_expiry) VALUES ($1, $2, $3)")
            .bind(user_id)
            .bind(reason)
            .bind(date_expiry_string)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| database::Error::Error)
    }

    async fn grant_admin_role(&self, user_id: i64) -> Result<(), database::Error> {
        query("UPDATE torrust_users SET administrator = TRUE WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|_| database::Error::Error)
            .and_then(|v| {
                if v.rows_affected() > 0 {
                    Ok(())
                } else {
                    Err(database::Error::UserNotFound)
                }
            })
    }

    async fn verify_email(&self, user_id: i64) -> Result<(), database::Error> {
        query("UPDATE torrust_user_profiles SET email_verified = TRUE WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| database::Error::Error)
    }

    async fn add_tracker_key(&self, user_id: i64, tracker_key: &TrackerKey) -> Result<(), database::Error> {
        let key = tracker_key.key.clone();

        query("INSERT INTO torrust_tracker_keys (user_id, tracker_key, date_expiry) VALUES ($1, $2, $3)")
            .bind(user_id)
            .bind(key)
            .bind(tracker_key.valid_until)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| database::Error::Error)
    }

    async fn delete_user(&self, user_id: i64) -> Result<(), database::Error> {
        query("DELETE FROM torrust_users WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|_| database::Error::Error)
            .and_then(|v| {
                if v.rows_affected() > 0 {
                    Ok(())
                } else {
                    Err(database::Error::UserNotFound)
                }
            })
    }

    async fn insert_category_and_get_id(&self, category_name: &str) -> Result<i64, database::Error> {
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

    async fn get_category_from_id(&self, category_id: i64) -> Result<Category, database::Error> {
        query_as::<_, Category>("SELECT category_id, name, (SELECT COUNT(*) FROM torrust_torrents WHERE torrust_torrents.category_id = torrust_categories.category_id) AS num_torrents FROM torrust_categories WHERE category_id = ?")
            .bind(category_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| database::Error::CategoryNotFound)
    }

    async fn get_category_from_name(&self, category_name: &str) -> Result<Category, database::Error> {
        query_as::<_, Category>("SELECT category_id, name, (SELECT COUNT(*) FROM torrust_torrents WHERE torrust_torrents.category_id = torrust_categories.category_id) AS num_torrents FROM torrust_categories WHERE name = ?")
            .bind(category_name)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| database::Error::CategoryNotFound)
    }

    async fn get_categories(&self) -> Result<Vec<Category>, database::Error> {
        query_as::<_, Category>("SELECT tc.category_id, tc.name, COUNT(tt.category_id) as num_torrents FROM torrust_categories tc LEFT JOIN torrust_torrents tt on tc.category_id = tt.category_id GROUP BY tc.name")
            .fetch_all(&self.pool)
            .await
            .map_err(|_| database::Error::Error)
    }

    async fn delete_category(&self, category_name: &str) -> Result<(), database::Error> {
        query("DELETE FROM torrust_categories WHERE name = ?")
            .bind(category_name)
            .execute(&self.pool)
            .await
            .map_err(|_| database::Error::Error)
            .and_then(|v| {
                if v.rows_affected() > 0 {
                    Ok(())
                } else {
                    Err(database::Error::CategoryNotFound)
                }
            })
    }

    // TODO: refactor this
    async fn get_torrents_search_sorted_paginated(
        &self,
        search: &Option<String>,
        categories: &Option<Vec<String>>,
        tags: &Option<Vec<String>>,
        sort: &Sorting,
        offset: u64,
        limit: u8,
    ) -> Result<TorrentsResponse, database::Error> {
        let title = match search {
            None => "%".to_string(),
            Some(v) => format!("%{v}%"),
        };

        let sort_query: String = match sort {
            Sorting::UploadedAsc => "date_uploaded ASC".to_string(),
            Sorting::UploadedDesc => "date_uploaded DESC".to_string(),
            Sorting::SeedersAsc => "seeders ASC".to_string(),
            Sorting::SeedersDesc => "seeders DESC".to_string(),
            Sorting::LeechersAsc => "leechers ASC".to_string(),
            Sorting::LeechersDesc => "leechers DESC".to_string(),
            Sorting::NameAsc => "title ASC".to_string(),
            Sorting::NameDesc => "title DESC".to_string(),
            Sorting::SizeAsc => "size ASC".to_string(),
            Sorting::SizeDesc => "size DESC".to_string(),
        };

        let category_filter_query = if let Some(c) = categories {
            let mut i = 0;
            let mut category_filters = String::new();
            for category in c {
                // don't take user input in the db query
                if let Ok(sanitized_category) = self.get_category_from_name(category).await {
                    let mut str = format!("tc.name = '{}'", sanitized_category.name);
                    if i > 0 {
                        str = format!(" OR {str}");
                    }
                    category_filters.push_str(&str);
                    i += 1;
                }
            }
            if category_filters.is_empty() {
                String::new()
            } else {
                format!("INNER JOIN torrust_categories tc ON tt.category_id = tc.category_id AND ({category_filters}) ")
            }
        } else {
            String::new()
        };

        let tag_filter_query = if let Some(t) = tags {
            let mut i = 0;
            let mut tag_filters = String::new();
            for tag in t {
                // don't take user input in the db query
                if let Ok(sanitized_tag) = self.get_tag_from_name(tag).await {
                    let mut str = format!("tl.tag_id = '{}'", sanitized_tag.tag_id);
                    if i > 0 {
                        str = format!(" OR {str}");
                    }
                    tag_filters.push_str(&str);
                    i += 1;
                }
            }
            if tag_filters.is_empty() {
                String::new()
            } else {
                format!("INNER JOIN torrust_torrent_tag_links tl ON tt.torrent_id = tl.torrent_id AND ({tag_filters}) ")
            }
        } else {
            String::new()
        };

        let mut query_string = format!(
            "SELECT tt.torrent_id, tp.username AS uploader, tt.info_hash, ti.title, ti.description, tt.category_id, tt.date_uploaded, tt.size AS file_size, tt.name, 
            CAST(COALESCE(sum(ts.seeders),0) as signed) as seeders,
            CAST(COALESCE(sum(ts.leechers),0) as signed) as leechers
            FROM torrust_torrents tt
            {category_filter_query}
            {tag_filter_query}
            INNER JOIN torrust_user_profiles tp ON tt.uploader_id = tp.user_id
            INNER JOIN torrust_torrent_info ti ON tt.torrent_id = ti.torrent_id
            LEFT JOIN torrust_torrent_tracker_stats ts ON tt.torrent_id = ts.torrent_id
            WHERE title LIKE ?
            GROUP BY tt.torrent_id"
        );

        let count_query = format!("SELECT COUNT(*) as count FROM ({query_string}) AS count_table");

        let count_result: Result<i64, database::Error> = query_as(&count_query)
            .bind(title.clone())
            .fetch_one(&self.pool)
            .await
            .map(|(v,)| v)
            .map_err(|_| database::Error::Error);

        let count = count_result?;

        query_string = format!("{query_string} ORDER BY {sort_query} LIMIT ?, ?");

        let res: Vec<TorrentListing> = sqlx::query_as::<_, TorrentListing>(&query_string)
            .bind(title)
            .bind(i64::saturating_add_unsigned(0, offset))
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|_| database::Error::Error)?;

        Ok(TorrentsResponse {
            total: u32::try_from(count).expect("variable `count` is larger than u32"),
            results: res,
        })
    }

    #[allow(clippy::too_many_lines)]
    async fn insert_torrent_and_get_id(
        &self,
        original_info_hash: &InfoHash,
        torrent: &Torrent,
        uploader_id: UserId,
        category_id: i64,
        title: &str,
        description: &str,
    ) -> Result<i64, database::Error> {
        let info_hash = torrent.info_hash();

        // open pool connection
        let mut conn = self.pool.acquire().await.map_err(|_| database::Error::Error)?;

        // start db transaction
        let mut tx = conn.begin().await.map_err(|_| database::Error::Error)?;

        // torrent file can only hold a pieces key or a root hash key: http://www.bittorrent.org/beps/bep_0030.html
        let (pieces, root_hash): (String, bool) = if let Some(pieces) = &torrent.info.pieces {
            (from_bytes(pieces.as_ref()), false)
        } else {
            let root_hash = torrent.info.root_hash.as_ref().ok_or(database::Error::Error)?;
            (root_hash.to_string(), true)
        };

        let private = torrent.info.private.unwrap_or(0);

        // add torrent
        let torrent_id = query("INSERT INTO torrust_torrents (uploader_id, category_id, info_hash, size, name, pieces, piece_length, private, root_hash, `source`, original_info_hash, date_uploaded) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, strftime('%Y-%m-%d %H:%M:%S',DATETIME('now', 'utc')))")
            .bind(uploader_id)
            .bind(category_id)
            .bind(info_hash.to_lowercase())
            .bind(torrent.file_size())
            .bind(torrent.info.name.to_string())
            .bind(pieces)
            .bind(torrent.info.piece_length)
            .bind(private)
            .bind(root_hash)
            .bind(torrent.info.source.clone())
            .bind(original_info_hash.to_hex_string())
            .execute(&self.pool)
            .await
            .map(|v| v.last_insert_rowid())
            .map_err(|e| match e {
                sqlx::Error::Database(err) => {
                    if err.message().contains("info_hash") {
                        database::Error::TorrentAlreadyExists
                    } else if err.message().contains("title") {
                        database::Error::TorrentTitleAlreadyExists
                    } else {
                        database::Error::Error
                    }
                }
                _ => database::Error::Error
            })?;

        let insert_torrent_files_result = if let Some(length) = torrent.info.length {
            query("INSERT INTO torrust_torrent_files (md5sum, torrent_id, length) VALUES (?, ?, ?)")
                .bind(torrent.info.md5sum.clone())
                .bind(torrent_id)
                .bind(length)
                .execute(&mut tx)
                .await
                .map(|_| ())
                .map_err(|_| database::Error::Error)
        } else {
            let files = torrent.info.files.as_ref().unwrap();

            for file in files {
                let path = file.path.join("/");

                let _ = query("INSERT INTO torrust_torrent_files (md5sum, torrent_id, length, path) VALUES (?, ?, ?, ?)")
                    .bind(file.md5sum.clone())
                    .bind(torrent_id)
                    .bind(file.length)
                    .bind(path)
                    .execute(&mut tx)
                    .await
                    .map_err(|_| database::Error::Error)?;
            }

            Ok(())
        };

        // rollback transaction on error
        if let Err(e) = insert_torrent_files_result {
            drop(tx.rollback().await);
            return Err(e);
        }

        let insert_torrent_announce_urls_result: Result<(), database::Error> = if let Some(announce_urls) = &torrent.announce_list
        {
            // flatten the nested vec (this will however remove the)
            let announce_urls = announce_urls.iter().flatten().collect::<Vec<&String>>();

            for tracker_url in &announce_urls {
                let () = query("INSERT INTO torrust_torrent_announce_urls (torrent_id, tracker_url) VALUES (?, ?)")
                    .bind(torrent_id)
                    .bind(tracker_url)
                    .execute(&mut tx)
                    .await
                    .map(|_| ())
                    .map_err(|_| database::Error::Error)?;
            }

            Ok(())
        } else {
            let tracker_url = torrent.announce.as_ref().unwrap();

            query("INSERT INTO torrust_torrent_announce_urls (torrent_id, tracker_url) VALUES (?, ?)")
                .bind(torrent_id)
                .bind(tracker_url)
                .execute(&mut tx)
                .await
                .map(|_| ())
                .map_err(|_| database::Error::Error)
        };

        // rollback transaction on error
        if let Err(e) = insert_torrent_announce_urls_result {
            drop(tx.rollback().await);
            return Err(e);
        }

        let insert_torrent_info_result =
            query(r#"INSERT INTO torrust_torrent_info (torrent_id, title, description) VALUES (?, ?, NULLIF(?, ""))"#)
                .bind(torrent_id)
                .bind(title)
                .bind(description)
                .execute(&mut tx)
                .await
                .map_err(|e| match e {
                    sqlx::Error::Database(err) => {
                        if err.message().contains("info_hash") {
                            database::Error::TorrentAlreadyExists
                        } else if err.message().contains("title") {
                            database::Error::TorrentTitleAlreadyExists
                        } else {
                            database::Error::Error
                        }
                    }
                    _ => database::Error::Error,
                });

        // commit or rollback transaction and return user_id on success
        match insert_torrent_info_result {
            Ok(_) => {
                drop(tx.commit().await);
                Ok(torrent_id)
            }
            Err(e) => {
                drop(tx.rollback().await);
                Err(e)
            }
        }
    }

    async fn get_torrent_info_from_id(&self, torrent_id: i64) -> Result<DbTorrentInfo, database::Error> {
        query_as::<_, DbTorrentInfo>(
            "SELECT torrent_id, info_hash, name, pieces, piece_length, private, root_hash FROM torrust_torrents WHERE torrent_id = ?",
        )
        .bind(torrent_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| database::Error::TorrentNotFound)
    }

    async fn get_torrent_info_from_info_hash(&self, info_hash: &InfoHash) -> Result<DbTorrentInfo, database::Error> {
        query_as::<_, DbTorrentInfo>(
            "SELECT torrent_id, info_hash, name, pieces, piece_length, private, root_hash FROM torrust_torrents WHERE info_hash = ?",
        )
        .bind(info_hash.to_hex_string().to_lowercase())
        .fetch_one(&self.pool)
        .await
        .map_err(|_| database::Error::TorrentNotFound)
    }

    async fn get_torrent_files_from_id(&self, torrent_id: i64) -> Result<Vec<TorrentFile>, database::Error> {
        let db_torrent_files =
            query_as::<_, DbTorrentFile>("SELECT md5sum, length, path FROM torrust_torrent_files WHERE torrent_id = ?")
                .bind(torrent_id)
                .fetch_all(&self.pool)
                .await
                .map_err(|_| database::Error::TorrentNotFound)?;

        let torrent_files: Vec<TorrentFile> = db_torrent_files
            .into_iter()
            .map(|tf| TorrentFile {
                path: tf
                    .path
                    .unwrap_or_default()
                    .split('/')
                    .map(std::string::ToString::to_string)
                    .collect(),
                length: tf.length,
                md5sum: tf.md5sum,
            })
            .collect();

        Ok(torrent_files)
    }

    async fn get_torrent_announce_urls_from_id(&self, torrent_id: i64) -> Result<Vec<Vec<String>>, database::Error> {
        query_as::<_, DbTorrentAnnounceUrl>("SELECT tracker_url FROM torrust_torrent_announce_urls WHERE torrent_id = ?")
            .bind(torrent_id)
            .fetch_all(&self.pool)
            .await
            .map(|v| v.iter().map(|a| vec![a.tracker_url.to_string()]).collect())
            .map_err(|_| database::Error::TorrentNotFound)
    }

    async fn get_torrent_listing_from_id(&self, torrent_id: i64) -> Result<TorrentListing, database::Error> {
        query_as::<_, TorrentListing>(
            "SELECT tt.torrent_id, tp.username AS uploader, tt.info_hash, ti.title, ti.description, tt.category_id, tt.date_uploaded, tt.size AS file_size, tt.name,
            CAST(COALESCE(sum(ts.seeders),0) as signed) as seeders,
            CAST(COALESCE(sum(ts.leechers),0) as signed) as leechers
            FROM torrust_torrents tt
            INNER JOIN torrust_user_profiles tp ON tt.uploader_id = tp.user_id
            INNER JOIN torrust_torrent_info ti ON tt.torrent_id = ti.torrent_id
            LEFT JOIN torrust_torrent_tracker_stats ts ON tt.torrent_id = ts.torrent_id
            WHERE tt.torrent_id = ?
            GROUP BY ts.torrent_id"
        )
            .bind(torrent_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| database::Error::TorrentNotFound)
    }

    async fn get_torrent_listing_from_info_hash(&self, info_hash: &InfoHash) -> Result<TorrentListing, database::Error> {
        query_as::<_, TorrentListing>(
            "SELECT tt.torrent_id, tp.username AS uploader, tt.info_hash, ti.title, ti.description, tt.category_id, tt.date_uploaded, tt.size AS file_size, tt.name,
            CAST(COALESCE(sum(ts.seeders),0) as signed) as seeders,
            CAST(COALESCE(sum(ts.leechers),0) as signed) as leechers
            FROM torrust_torrents tt
            INNER JOIN torrust_user_profiles tp ON tt.uploader_id = tp.user_id
            INNER JOIN torrust_torrent_info ti ON tt.torrent_id = ti.torrent_id
            LEFT JOIN torrust_torrent_tracker_stats ts ON tt.torrent_id = ts.torrent_id
            WHERE tt.info_hash = ?
            GROUP BY ts.torrent_id"
        )
            .bind(info_hash.to_string().to_lowercase())
            .fetch_one(&self.pool)
            .await
            .map_err(|_| database::Error::TorrentNotFound)
    }

    async fn get_all_torrents_compact(&self) -> Result<Vec<TorrentCompact>, database::Error> {
        query_as::<_, TorrentCompact>("SELECT torrent_id, info_hash FROM torrust_torrents")
            .fetch_all(&self.pool)
            .await
            .map_err(|_| database::Error::Error)
    }

    async fn update_torrent_title(&self, torrent_id: i64, title: &str) -> Result<(), database::Error> {
        query("UPDATE torrust_torrent_info SET title = $1 WHERE torrent_id = $2")
            .bind(title)
            .bind(torrent_id)
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(err) => {
                    if err.message().contains("UNIQUE") {
                        database::Error::TorrentTitleAlreadyExists
                    } else {
                        database::Error::Error
                    }
                }
                _ => database::Error::Error,
            })
            .and_then(|v| {
                if v.rows_affected() > 0 {
                    Ok(())
                } else {
                    Err(database::Error::TorrentNotFound)
                }
            })
    }

    async fn update_torrent_description(&self, torrent_id: i64, description: &str) -> Result<(), database::Error> {
        query("UPDATE torrust_torrent_info SET description = $1 WHERE torrent_id = $2")
            .bind(description)
            .bind(torrent_id)
            .execute(&self.pool)
            .await
            .map_err(|_| database::Error::Error)
            .and_then(|v| {
                if v.rows_affected() > 0 {
                    Ok(())
                } else {
                    Err(database::Error::TorrentNotFound)
                }
            })
    }

    async fn update_torrent_category(&self, torrent_id: i64, category_id: CategoryId) -> Result<(), database::Error> {
        query("UPDATE torrust_torrents SET category_id = $1 WHERE torrent_id = $2")
            .bind(category_id)
            .bind(torrent_id)
            .execute(&self.pool)
            .await
            .map_err(|_| database::Error::Error)
            .and_then(|v| {
                if v.rows_affected() > 0 {
                    Ok(())
                } else {
                    Err(database::Error::TorrentNotFound)
                }
            })
    }

    async fn add_tag(&self, name: &str) -> Result<(), database::Error> {
        query("INSERT INTO torrust_torrent_tags (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|err| database::Error::ErrorWithText(err.to_string()))
    }

    async fn delete_tag(&self, tag_id: TagId) -> Result<(), database::Error> {
        query("DELETE FROM torrust_torrent_tags WHERE tag_id = ?")
            .bind(tag_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|err| database::Error::ErrorWithText(err.to_string()))
    }

    async fn add_torrent_tag_link(&self, torrent_id: i64, tag_id: TagId) -> Result<(), database::Error> {
        query("INSERT INTO torrust_torrent_tag_links (torrent_id, tag_id) VALUES (?, ?)")
            .bind(torrent_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| database::Error::Error)
    }

    async fn add_torrent_tag_links(&self, torrent_id: i64, tag_ids: &[TagId]) -> Result<(), database::Error> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|err| database::Error::ErrorWithText(err.to_string()))?;

        for tag_id in tag_ids {
            query("INSERT INTO torrust_torrent_tag_links (torrent_id, tag_id) VALUES (?, ?)")
                .bind(torrent_id)
                .bind(tag_id)
                .execute(&mut transaction)
                .await
                .map_err(|err| database::Error::ErrorWithText(err.to_string()))?;
        }

        transaction
            .commit()
            .await
            .map_err(|err| database::Error::ErrorWithText(err.to_string()))
    }

    async fn delete_torrent_tag_link(&self, torrent_id: i64, tag_id: TagId) -> Result<(), database::Error> {
        query("DELETE FROM torrust_torrent_tag_links WHERE torrent_id = ? AND tag_id = ?")
            .bind(torrent_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| database::Error::Error)
    }

    async fn delete_all_torrent_tag_links(&self, torrent_id: i64) -> Result<(), database::Error> {
        query("DELETE FROM torrust_torrent_tag_links WHERE torrent_id = ?")
            .bind(torrent_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|err| database::Error::ErrorWithText(err.to_string()))
    }

    async fn get_tag_from_name(&self, name: &str) -> Result<TorrentTag, database::Error> {
        query_as::<_, TorrentTag>("SELECT tag_id, name FROM torrust_torrent_tags WHERE name = ?")
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| database::Error::TagNotFound)
    }

    async fn get_tags(&self) -> Result<Vec<TorrentTag>, database::Error> {
        query_as::<_, TorrentTag>("SELECT tag_id, name FROM torrust_torrent_tags")
            .fetch_all(&self.pool)
            .await
            .map_err(|_| database::Error::Error)
    }

    async fn get_tags_for_torrent_id(&self, torrent_id: i64) -> Result<Vec<TorrentTag>, database::Error> {
        query_as::<_, TorrentTag>(
            "SELECT torrust_torrent_tags.tag_id, torrust_torrent_tags.name
            FROM torrust_torrent_tags
            JOIN torrust_torrent_tag_links ON torrust_torrent_tags.tag_id = torrust_torrent_tag_links.tag_id
            WHERE torrust_torrent_tag_links.torrent_id = ?",
        )
        .bind(torrent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| database::Error::Error)
    }

    async fn update_tracker_info(
        &self,
        torrent_id: i64,
        tracker_url: &str,
        seeders: i64,
        leechers: i64,
    ) -> Result<(), database::Error> {
        query("REPLACE INTO torrust_torrent_tracker_stats (torrent_id, tracker_url, seeders, leechers) VALUES ($1, $2, $3, $4)")
            .bind(torrent_id)
            .bind(tracker_url)
            .bind(seeders)
            .bind(leechers)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| database::Error::TorrentNotFound)
    }

    async fn delete_torrent(&self, torrent_id: i64) -> Result<(), database::Error> {
        query("DELETE FROM torrust_torrents WHERE torrent_id = ?")
            .bind(torrent_id)
            .execute(&self.pool)
            .await
            .map_err(|_| database::Error::Error)
            .and_then(|v| {
                if v.rows_affected() > 0 {
                    Ok(())
                } else {
                    Err(database::Error::TorrentNotFound)
                }
            })
    }

    async fn delete_all_database_rows(&self) -> Result<(), database::Error> {
        for table in TABLES_TO_TRUNCATE {
            query(&format!("DELETE FROM {table};"))
                .execute(&self.pool)
                .await
                .map_err(|_| database::Error::Error)?;
        }

        Ok(())
    }
}
