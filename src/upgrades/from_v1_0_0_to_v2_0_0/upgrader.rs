//! It updates the application from version v1.0.0 to v2.0.0.
//!
//! NOTES for `torrust_users` table transfer:
//!
//! - In v2, the table `torrust_user` contains a field `date_registered` non existing in v1.
//!   It's used the day when the upgrade command is executed.
//! - In v2, the table `torrust_user_profiles` contains two new fields: `bio` and `avatar`.
//!   Empty string is used as default value.

use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v2_0_0::SqliteDatabaseV2_0_0;
use chrono::prelude::{DateTime, Utc};
use std::{sync::Arc, time::SystemTime};

use crate::config::Configuration;

fn today_iso8601() -> String {
    let dt: DateTime<Utc> = SystemTime::now().into();
    format!("{}", dt.format("%Y-%m-%d %H:%M:%S"))
}

async fn current_db() -> Arc<SqliteDatabaseV1_0_0> {
    // Connect to the old v1.0.0 DB
    let cfg = match Configuration::load_from_file().await {
        Ok(config) => Arc::new(config),
        Err(error) => {
            panic!("{}", error)
        }
    };

    let settings = cfg.settings.read().await;

    Arc::new(SqliteDatabaseV1_0_0::new(&settings.database.connect_url).await)
}

async fn new_db(db_filename: String) -> Arc<SqliteDatabaseV2_0_0> {
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
    println!("result {:?}", result);

    for cat in &source_categories {
        println!(
            "[v2] adding category: {:?} {:?} ...",
            &cat.category_id, &cat.name
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
            "[v2][torrust_users] adding user: {:?} {:?} ...",
            &user.user_id, &user.username
        );

        let default_data_registered = today_iso8601();

        let id = dest_database
            .insert_user(user.user_id, &default_data_registered, user.administrator)
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
            "[v2][torrust_user_profiles] adding user: {:?} {:?} ...",
            &user.user_id, &user.username
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
            "[v2][torrust_user_profiles] user: {:?} {:?} added.",
            &user.user_id, &user.username
        );

        // [v2] table torrust_user_authentication

        println!(
            "[v2][torrust_user_authentication] adding password hash ({:?}) for user ({:?}) ...",
            &user.password, &user.user_id
        );

        dest_database
            .insert_user_password_hash(user.user_id, &user.password)
            .await
            .unwrap();

        println!(
            "[v2][torrust_user_authentication] password hash ({:?}) added for user ({:?}).",
            &user.password, &user.user_id
        );
    }
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
            "[v2][torrust_users] adding the tracker key: {:?} ...",
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
            "[v2][torrust_tracker_keys] tracker key: {:?} added.",
            &tracker_key.key_id
        );
    }
}

pub async fn upgrade() {
    // Get connections to source adn destiny databases
    let source_database = current_db().await;
    let dest_database = new_db("data_v2.db".to_string()).await;

    println!("Upgrading data from version v1.0.0 to v2.0.0 ...");

    migrate_destiny_database(dest_database.clone()).await;
    reset_destiny_database(dest_database.clone()).await;
    transfer_categories(source_database.clone(), dest_database.clone()).await;
    transfer_user_data(source_database.clone(), dest_database.clone()).await;
    transfer_tracker_keys(source_database.clone(), dest_database.clone()).await;

    // TODO: WIP. We have to transfer data from the 5 tables in V1 and the torrent files in folder `uploads`.
}
