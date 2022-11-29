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
use std::fs;
use std::sync::Arc;

use torrust_index_backend::upgrades::from_v1_0_0_to_v2_0_0::upgrader::{datetime_iso_8601, upgrade, Arguments};

use crate::upgrades::from_v1_0_0_to_v2_0_0::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::sqlite_v2_0_0::SqliteDatabaseV2_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::testers::category_tester::CategoryTester;
use crate::upgrades::from_v1_0_0_to_v2_0_0::testers::torrent_tester::TorrentTester;
use crate::upgrades::from_v1_0_0_to_v2_0_0::testers::tracker_key_tester::TrackerKeyTester;
use crate::upgrades::from_v1_0_0_to_v2_0_0::testers::user_tester::UserTester;

struct TestConfig {
    // Directories
    pub fixtures_dir: String,
    pub upload_path: String,
    // Files
    pub source_database_file: String,
    pub destiny_database_file: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        let fixtures_dir = "./tests/upgrades/from_v1_0_0_to_v2_0_0/fixtures/".to_string();
        let upload_path = format!("{}uploads/", &fixtures_dir);
        let output_dir = "./tests/upgrades/from_v1_0_0_to_v2_0_0/output/".to_string();
        let source_database_file = format!("{}source.db", output_dir);
        let destiny_database_file = format!("{}destiny.db", output_dir);
        Self {
            fixtures_dir,
            upload_path,
            source_database_file,
            destiny_database_file,
        }
    }
}

#[tokio::test]
async fn upgrades_data_from_version_v1_0_0_to_v2_0_0() {
    let config = TestConfig::default();

    let (source_db, dest_db) = setup_databases(&config).await;

    // The datetime when the upgrader is executed
    let execution_time = datetime_iso_8601();

    let category_tester = CategoryTester::new(source_db.clone(), dest_db.clone());
    let user_tester = UserTester::new(source_db.clone(), dest_db.clone(), &execution_time);
    let tracker_key_tester = TrackerKeyTester::new(source_db.clone(), dest_db.clone(), user_tester.test_data.user.user_id);
    let torrent_tester = TorrentTester::new(
        source_db.clone(),
        dest_db.clone(),
        &user_tester.test_data.user,
        category_tester.get_valid_category_id(),
    );

    // Load data into source database in version v1.0.0
    category_tester.load_data_into_source_db().await;
    user_tester.load_data_into_source_db().await;
    tracker_key_tester.load_data_into_source_db().await;
    torrent_tester.load_data_into_source_db().await;

    // Run the upgrader
    upgrade(
        &Arguments {
            source_database_file: config.source_database_file.clone(),
            destiny_database_file: config.destiny_database_file.clone(),
            upload_path: config.upload_path.clone(),
        },
        &execution_time,
    )
    .await;

    // Assertions for data transferred to the new database in version v2.0.0
    category_tester.assert_data_in_destiny_db().await;
    user_tester.assert_data_in_destiny_db().await;
    tracker_key_tester.assert_data_in_destiny_db().await;
    torrent_tester.assert_data_in_destiny_db(&config.upload_path).await;
}

async fn setup_databases(config: &TestConfig) -> (Arc<SqliteDatabaseV1_0_0>, Arc<SqliteDatabaseV2_0_0>) {
    // Set up clean source database
    reset_databases(&config.source_database_file, &config.destiny_database_file);
    let source_database = source_db_connection(&config.source_database_file).await;
    source_database.migrate(&config.fixtures_dir).await;

    // Set up connection for the destiny database
    let destiny_database = destiny_db_connection(&config.destiny_database_file).await;

    (source_database, destiny_database)
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
