//! You can run this test with:
//!
//! //! ```text
//! cargo test upgrades_data_from_version_v1_0_0_to_v2_0_0
//! ```
//!
//! or:
//!
//! ```text
//! cargo test upgrades_data_from_version_v1_0_0_to_v2_0_0 -- --nocapture
//! ```
//!
//! to see the "upgrader" command output.
use crate::upgrades::from_v1_0_0_to_v2_0_0::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::sqlite_v2_0_0::SqliteDatabaseV2_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::testers::tracker_keys_tester::TrackerKeysTester;
use crate::upgrades::from_v1_0_0_to_v2_0_0::testers::user_data_tester::UserDataTester;
use std::fs;
use std::sync::Arc;
use torrust_index_backend::upgrades::from_v1_0_0_to_v2_0_0::upgrader::{
    datetime_iso_8601, upgrade, Arguments,
};

#[tokio::test]
async fn upgrades_data_from_version_v1_0_0_to_v2_0_0() {
    // Directories
    let fixtures_dir = "./tests/upgrades/from_v1_0_0_to_v2_0_0/fixtures/".to_string();
    let output_dir = "./tests/upgrades/from_v1_0_0_to_v2_0_0/output/".to_string();

    // Files
    let source_database_file = format!("{}source.db", output_dir);
    let destiny_database_file = format!("{}destiny.db", output_dir);

    // Set up clean source database
    reset_databases(&source_database_file, &destiny_database_file);
    let source_database = source_db_connection(&source_database_file).await;
    source_database.migrate(&fixtures_dir).await;

    // Set up connection for the destiny database
    let destiny_database = destiny_db_connection(&destiny_database_file).await;

    // The datetime when the upgrader is executed
    let execution_time = datetime_iso_8601();

    // Load data into database v1

    // `torrust_users`, `torrust_user_profiles` and `torrust_user_authentication` tables

    let user_data_tester = UserDataTester::new(
        source_database.clone(),
        destiny_database.clone(),
        &execution_time,
    );

    user_data_tester.load_data_into_source_db().await;

    // `torrust_tracker_keys` table

    let tracker_keys_tester = TrackerKeysTester::new(
        source_database.clone(),
        destiny_database.clone(),
        user_data_tester.test_data.user.user_id,
    );

    tracker_keys_tester.load_data_into_source_db().await;

    // `torrust_torrents` table

    // TODO

    // Run the upgrader
    let args = Arguments {
        source_database_file: source_database_file.clone(),
        destiny_database_file: destiny_database_file.clone(),
        upload_path: format!("{}uploads/", fixtures_dir),
    };
    upgrade(&args, &execution_time).await;

    // Assertions in database v2

    // `torrust_users`, `torrust_user_profiles` and `torrust_user_authentication` tables

    user_data_tester.assert().await;

    // `torrust_tracker_keys` table

    tracker_keys_tester.assert().await;

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
