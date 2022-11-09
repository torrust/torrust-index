//! It updates the application from version v1.0.0 to v2.0.0.
//!
//! NOTES for `torrust_users` table transfer:
//!
//! - In v2, the table `torrust_user` contains a field `date_registered` non existing in v1.
//!   We changed that columns to allow NULL. WE also added the new column `date_imported` with
//!   the datetime when the upgrader was executed.
//!
//! NOTES for `torrust_user_profiles` table transfer:
//!
//! - In v2, the table `torrust_user_profiles` contains two new fields: `bio` and `avatar`.
//!   Empty string is used as default value.

use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v2_0_0::SqliteDatabaseV2_0_0;
use crate::utils::parse_torrent::decode_torrent;
use crate::{
    models::torrent_file::Torrent,
    upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v1_0_0::SqliteDatabaseV1_0_0,
};
use chrono::prelude::{DateTime, Utc};
use chrono::NaiveDateTime;

use std::{error, fs};
use std::{sync::Arc, time::SystemTime};

use crate::config::Configuration;

pub async fn upgrade() {
    // TODO: get from command arguments
    let database_file = "data_v2.db".to_string(); // The new database
    let upload_path = "./uploads".to_string(); // The relative dir where torrent files are stored

    let cfg = match Configuration::load_from_file().await {
        Ok(config) => Arc::new(config),
        Err(error) => {
            panic!("{}", error)
        }
    };

    let settings = cfg.settings.read().await;

    // Get connection to source database (current DB in settings)
    let source_database = current_db(&settings.database.connect_url).await;

    // Get connection to destiny database
    let dest_database = new_db(&database_file).await;

    println!("Upgrading data from version v1.0.0 to v2.0.0 ...");

    migrate_destiny_database(dest_database.clone()).await;
    reset_destiny_database(dest_database.clone()).await;
    transfer_categories(source_database.clone(), dest_database.clone()).await;
    transfer_user_data(source_database.clone(), dest_database.clone()).await;
    transfer_tracker_keys(source_database.clone(), dest_database.clone()).await;
    transfer_torrents(source_database.clone(), dest_database.clone(), &upload_path).await;
}

async fn current_db(connect_url: &str) -> Arc<SqliteDatabaseV1_0_0> {
    Arc::new(SqliteDatabaseV1_0_0::new(connect_url).await)
}

async fn new_db(db_filename: &str) -> Arc<SqliteDatabaseV2_0_0> {
    let dest_database_connect_url = format!("sqlite://{}?mode=rwc", db_filename);
    Arc::new(SqliteDatabaseV2_0_0::new(&dest_database_connect_url).await)
}

async fn migrate_destiny_database(dest_database: Arc<SqliteDatabaseV2_0_0>) {
    println!("Running migrations in destiny database...");
    dest_database.migrate().await;
}

async fn reset_destiny_database(dest_database: Arc<SqliteDatabaseV2_0_0>) {
    println!("Truncating all tables in destiny database ...");
    dest_database
        .delete_all_database_rows()
        .await
        .expect("Can't reset destiny database.");
}

async fn transfer_categories(
    source_database: Arc<SqliteDatabaseV1_0_0>,
    dest_database: Arc<SqliteDatabaseV2_0_0>,
) {
    println!("Transferring categories ...");

    let source_categories = source_database.get_categories_order_by_id().await.unwrap();
    println!("[v1] categories: {:?}", &source_categories);

    let result = dest_database.reset_categories_sequence().await.unwrap();
    println!("[v2] reset categories sequence result {:?}", result);

    for cat in &source_categories {
        println!(
            "[v2] adding category {:?} with id {:?} ...",
            &cat.name, &cat.category_id
        );
        let id = dest_database
            .insert_category_and_get_id(&cat.name)
            .await
            .unwrap();

        if id != cat.category_id {
            panic!(
                "Error copying category {:?} from source DB to destiny DB",
                &cat.category_id
            );
        }

        println!("[v2] category: {:?} {:?} added.", id, &cat.name);
    }

    let dest_categories = dest_database.get_categories().await.unwrap();
    println!("[v2] categories: {:?}", &dest_categories);
}

async fn transfer_user_data(
    source_database: Arc<SqliteDatabaseV1_0_0>,
    dest_database: Arc<SqliteDatabaseV2_0_0>,
) {
    println!("Transferring users ...");

    // Transfer table `torrust_users`

    let users = source_database.get_users().await.unwrap();

    for user in &users {
        // [v2] table torrust_users

        println!(
            "[v2][torrust_users] adding user with username {:?} and id {:?} ...",
            &user.username, &user.user_id
        );

        let date_imported = today_iso8601();

        let id = dest_database
            .insert_imported_user(user.user_id, &date_imported, user.administrator)
            .await
            .unwrap();

        if id != user.user_id {
            panic!(
                "Error copying user {:?} from source DB to destiny DB",
                &user.user_id
            );
        }

        println!(
            "[v2][torrust_users] user: {:?} {:?} added.",
            &user.user_id, &user.username
        );

        // [v2] table torrust_user_profiles

        println!(
            "[v2][torrust_user_profiles] adding user profile for user with username {:?} and id {:?} ...",
            &user.username, &user.user_id
        );

        let default_user_bio = "".to_string();
        let default_user_avatar = "".to_string();

        dest_database
            .insert_user_profile(
                user.user_id,
                &user.username,
                &user.email,
                user.email_verified,
                &default_user_bio,
                &default_user_avatar,
            )
            .await
            .unwrap();

        println!(
            "[v2][torrust_user_profiles] user profile added for user with username {:?} and id {:?}.",
            &user.username, &user.user_id
        );

        // [v2] table torrust_user_authentication

        println!(
            "[v2][torrust_user_authentication] adding password hash ({:?}) for user id ({:?}) ...",
            &user.password, &user.user_id
        );

        dest_database
            .insert_user_password_hash(user.user_id, &user.password)
            .await
            .unwrap();

        println!(
            "[v2][torrust_user_authentication] password hash ({:?}) added for user id ({:?}).",
            &user.password, &user.user_id
        );
    }
}

fn today_iso8601() -> String {
    let dt: DateTime<Utc> = SystemTime::now().into();
    format!("{}", dt.format("%Y-%m-%d %H:%M:%S"))
}

async fn transfer_tracker_keys(
    source_database: Arc<SqliteDatabaseV1_0_0>,
    dest_database: Arc<SqliteDatabaseV2_0_0>,
) {
    println!("Transferring tracker keys ...");

    // Transfer table `torrust_tracker_keys`

    let tracker_keys = source_database.get_tracker_keys().await.unwrap();

    for tracker_key in &tracker_keys {
        // [v2] table torrust_tracker_keys

        println!(
            "[v2][torrust_users] adding the tracker key with id {:?} ...",
            &tracker_key.key_id
        );

        let id = dest_database
            .insert_tracker_key(
                tracker_key.key_id,
                tracker_key.user_id,
                &tracker_key.key,
                tracker_key.valid_until,
            )
            .await
            .unwrap();

        if id != tracker_key.key_id {
            panic!(
                "Error copying tracker key {:?} from source DB to destiny DB",
                &tracker_key.key_id
            );
        }

        println!(
            "[v2][torrust_tracker_keys] tracker key with id {:?} added.",
            &tracker_key.key_id
        );
    }
}

async fn transfer_torrents(
    source_database: Arc<SqliteDatabaseV1_0_0>,
    dest_database: Arc<SqliteDatabaseV2_0_0>,
    upload_path: &str,
) {
    println!("Transferring torrents ...");

    // Transfer table `torrust_torrents_files`

    // Although the The table `torrust_torrents_files` existed in version v1.0.0
    // it was was not used.

    // Transfer table `torrust_torrents`

    let torrents = source_database.get_torrents().await.unwrap();

    for torrent in &torrents {
        // [v2] table torrust_torrents

        println!(
            "[v2][torrust_torrents] adding the torrent: {:?} ...",
            &torrent.torrent_id
        );

        // All torrents were public in version v1.0.0
        let private = false;

        let uploader = source_database
            .get_user_by_username(&torrent.uploader)
            .await
            .unwrap();

        if uploader.username != torrent.uploader {
            panic!(
                "Error copying torrent with id {:?}.
                Username (`uploader`) in `torrust_torrents` table does not match `username` in `torrust_users` table",
                &torrent.torrent_id
            );
        }

        let filepath = format!("{}/{}.torrent", upload_path, &torrent.torrent_id);

        let torrent_from_file = read_torrent_from_file(&filepath).unwrap();

        let pieces = torrent_from_file.info.get_pieces_as_string();
        let root_hash = torrent_from_file.info.get_root_hash_as_i64();

        let id = dest_database
            .insert_torrent(
                torrent.torrent_id,
                uploader.user_id,
                torrent.category_id,
                &torrent_from_file.info_hash(),
                torrent.file_size,
                &torrent_from_file.info.name,
                &pieces,
                torrent_from_file.info.piece_length,
                private,
                root_hash,
                &convert_timestamp_to_datetime(torrent.upload_date),
            )
            .await
            .unwrap();

        if id != torrent.torrent_id {
            panic!(
                "Error copying torrent {:?} from source DB to destiny DB",
                &torrent.torrent_id
            );
        }

        println!(
            "[v2][torrust_torrents] torrent with id {:?} added.",
            &torrent.torrent_id
        );

        // [v2] table torrust_torrent_files

        println!("[v2][torrust_torrent_files] adding torrent files");

        let _is_torrent_with_multiple_files = torrent_from_file.info.files.is_some();
        let is_torrent_with_a_single_file = torrent_from_file.info.length.is_some();

        if is_torrent_with_a_single_file {
            // The torrent contains only one file then:
            // - "path" is NULL
            // - "md5sum" can be NULL

            println!(
                "[v2][torrust_torrent_files][single-file-torrent] adding torrent file {:?} with length {:?} ...",
                &torrent_from_file.info.name, &torrent_from_file.info.length,
            );

            let file_id = dest_database
                .insert_torrent_file_for_torrent_with_one_file(
                    torrent.torrent_id,
                    // TODO: it seems med5sum can be None. Why? When?
                    &torrent_from_file.info.md5sum.clone(),
                    torrent_from_file.info.length.unwrap(),
                )
                .await;

            println!(
                "[v2][torrust_torrent_files][single-file-torrent] torrent file insert result: {:?}",
                &file_id
            );
        } else {
            // Multiple files are being shared
            let files = torrent_from_file.info.files.as_ref().unwrap();

            for file in files.iter() {
                println!(
                    "[v2][torrust_torrent_files][multiple-file-torrent] adding torrent file: {:?} ...",
                    &file
                );

                let file_id = dest_database
                    .insert_torrent_file_for_torrent_with_multiple_files(torrent, file)
                    .await;

                println!(
                    "[v2][torrust_torrent_files][multiple-file-torrent] torrent file insert result: {:?}",
                    &file_id
                );
            }
        }

        // [v2] table torrust_torrent_info

        println!(
            "[v2][torrust_torrent_info] adding the torrent info for torrent id {:?} ...",
            &torrent.torrent_id
        );

        let id = dest_database.insert_torrent_info(torrent).await;

        println!(
            "[v2][torrust_torrents] torrent info insert result: {:?}.",
            &id
        );

        // [v2] table torrust_torrent_announce_urls

        println!(
            "[v2][torrust_torrent_announce_urls] adding the torrent announce url for torrent id {:?} ...",
            &torrent.torrent_id
        );

        if torrent_from_file.announce.is_some() {
            println!("[v2][torrust_torrent_announce_urls][announce] adding the torrent announce url for torrent id {:?} ...", &torrent.torrent_id);

            let announce_url_id = dest_database
                .insert_torrent_announce_url(
                    torrent.torrent_id,
                    &torrent_from_file.announce.unwrap(),
                )
                .await;

            println!(
                "[v2][torrust_torrent_announce_urls][announce] torrent announce url insert result {:?} ...",
                &announce_url_id
            );
        } else if torrent_from_file.announce_list.is_some() {
            // BEP-0012. Multiple trackers.

            println!("[v2][torrust_torrent_announce_urls][announce-list] adding the torrent announce url for torrent id {:?} ...", &torrent.torrent_id);

            // flatten the nested vec (this will however remove the)
            let announce_urls = torrent_from_file
                .announce_list
                .clone()
                .unwrap()
                .into_iter()
                .flatten()
                .collect::<Vec<String>>();

            for tracker_url in announce_urls.iter() {
                println!("[v2][torrust_torrent_announce_urls][announce-list] adding the torrent announce url for torrent id {:?} ...", &torrent.torrent_id);

                let announce_url_id = dest_database
                    .insert_torrent_announce_url(torrent.torrent_id, tracker_url)
                    .await;

                println!("[v2][torrust_torrent_announce_urls][announce-list] torrent announce url insert result {:?} ...", &announce_url_id);
            }
        }
    }
    println!("Torrents transferred");
}

fn read_torrent_from_file(path: &str) -> Result<Torrent, Box<dyn error::Error>> {
    let contents = match fs::read(path) {
        Ok(contents) => contents,
        Err(e) => return Err(e.into()),
    };

    match decode_torrent(&contents) {
        Ok(torrent) => Ok(torrent),
        Err(e) => Err(e),
    }
}

fn convert_timestamp_to_datetime(timestamp: i64) -> String {
    // The expected format in database is: 2022-11-04 09:53:57
    // MySQL uses a DATETIME column and SQLite uses a TEXT column.

    let naive_datetime = NaiveDateTime::from_timestamp(timestamp, 0);
    let datetime_again: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);

    // Format without timezone
    datetime_again.format("%Y-%m-%d %H:%M:%S").to_string()
}
