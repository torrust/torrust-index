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
use crate::upgrades::from_v1_0_0_to_v2_0_0::testers::torrent_tester::TorrentTester;
use crate::upgrades::from_v1_0_0_to_v2_0_0::testers::tracker_key_tester::TrackerKeyTester;
use crate::upgrades::from_v1_0_0_to_v2_0_0::testers::user_tester::UserTester;
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
    let upload_path = format!("{}uploads/", &fixtures_dir);

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

    let user_tester = UserTester::new(
        source_database.clone(),
        destiny_database.clone(),
        &execution_time,
    );
    user_tester.load_data_into_source_db().await;

    let tracker_key_tester = TrackerKeyTester::new(
        source_database.clone(),
        destiny_database.clone(),
        user_tester.test_data.user.user_id,
    );
    tracker_key_tester.load_data_into_source_db().await;

    let torrent_tester = TorrentTester::new(
        source_database.clone(),
        destiny_database.clone(),
        &user_tester.test_data.user,
    );
    torrent_tester.load_data_into_source_db().await;

    // Run the upgrader
    let args = Arguments {
        source_database_file: source_database_file.clone(),
        destiny_database_file: destiny_database_file.clone(),
        upload_path: upload_path.clone(),
    };
    upgrade(&args, &execution_time).await;

    // Assertions in database v2

    user_tester.assert().await;
    tracker_key_tester.assert().await;
    torrent_tester.assert(&upload_path).await;
}

async fn source_db_connection(source_database_file: &str) -> Arc<SqliteDatabaseV1_0_0> {
    Arc::new(SqliteDatabaseV1_0_0::db_connection(&source_database_file).await)
}

async fn destiny_db_connection(destiny_database_file: &str) -> Arc<SqliteDatabaseV2_0_0> {
    Arc::new(SqliteDatabaseV2_0_0::db_connection(&destiny_database_file).await)
}

/// Reset databases from previous executions
fn reset_databases(source_database_file: &str, destiny_database_file: &str) {
    fs::remove_file(&source_database_file).expect("Can't remove source DB file.");
    fs::remove_file(&destiny_database_file).expect("Can't remove destiny DB file.");
}
