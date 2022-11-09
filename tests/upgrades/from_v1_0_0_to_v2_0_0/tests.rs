//! You can run this test with:
//!
//! ```text
//! cargo test upgrade_data_from_version_v1_0_0_to_v2_0_0 -- --nocapture
//! ```
use crate::upgrades::from_v1_0_0_to_v2_0_0::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::sqlite_v2_0_0::SqliteDatabaseV2_0_0;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand_core::OsRng;
use std::fs;
use std::sync::Arc;
use torrust_index_backend::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v1_0_0::UserRecordV1;
use torrust_index_backend::upgrades::from_v1_0_0_to_v2_0_0::upgrader::{
    datetime_iso_8601, upgrade, Arguments,
};

#[tokio::test]
async fn upgrade_data_from_version_v1_0_0_to_v2_0_0() {
    // Directories
    let fixtures_dir = "./tests/upgrades/from_v1_0_0_to_v2_0_0/fixtures/".to_string();
    let output_dir = "./tests/upgrades/from_v1_0_0_to_v2_0_0/output/".to_string();

    // Files
    let source_database_file = format!("{}source.db", output_dir);
    let destiny_database_file = format!("{}destiny.db", output_dir);

    // Set up clean database
    reset_databases(&source_database_file, &destiny_database_file);
    let source_database = source_db_connection(&source_database_file).await;
    source_database.migrate(&fixtures_dir).await;

    // Load data into database v1

    // `torrust_users` table

    let user = UserRecordV1 {
        user_id: 1,
        username: "user01".to_string(),
        email: "user01@torrust.com".to_string(),
        email_verified: true,
        password: hashed_valid_password(),
        administrator: true,
    };
    let user_id = source_database.insert_user(&user).await.unwrap();

    // `torrust_tracker_keys` table

    // TODO

    // `torrust_torrents` table

    // TODO

    // Run the upgrader
    let args = Arguments {
        source_database_file: source_database_file.clone(),
        destiny_database_file: destiny_database_file.clone(),
        upload_path: format!("{}uploads/", fixtures_dir),
    };
    let now = datetime_iso_8601();
    upgrade(&args, &now).await;

    // Assertions in database v2

    let destiny_database = destiny_db_connection(&destiny_database_file).await;

    // `torrust_users` table

    let imported_user = destiny_database.get_user(user_id).await.unwrap();

    assert_eq!(imported_user.user_id, user.user_id);
    assert!(imported_user.date_registered.is_none());
    assert_eq!(imported_user.date_imported.unwrap(), now);
    assert_eq!(imported_user.administrator, user.administrator);

    // `torrust_user_authentication` table

    // TODO

    // `torrust_user_profiles` table

    // TODO

    // `torrust_tracker_keys` table

    // TODO

    // `torrust_torrents` table

    // TODO

    // `torrust_torrent_files` table

    // TODO

    // `torrust_torrent_info` table

    // TODO

    // `torrust_torrent_announce_urls` table

    // TODO
}

async fn source_db_connection(source_database_file: &str) -> Arc<SqliteDatabaseV1_0_0> {
    Arc::new(SqliteDatabaseV1_0_0::db_connection(&source_database_file).await)
}

async fn destiny_db_connection(destiny_database_file: &str) -> Arc<SqliteDatabaseV2_0_0> {
    Arc::new(SqliteDatabaseV2_0_0::db_connection(&destiny_database_file).await)
}

/// Reset databases from previous executions
fn reset_databases(source_database_file: &str, destiny_database_file: &str) {
    // TODO: use a unique temporary dir
    fs::remove_file(&source_database_file).expect("Can't remove source DB file.");
    fs::remove_file(&destiny_database_file).expect("Can't remove destiny DB file.");
}

fn hashed_valid_password() -> String {
    hash_password(&valid_password())
}

fn valid_password() -> String {
    "123456".to_string()
}

fn hash_password(plain_password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    argon2
        .hash_password(plain_password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}
