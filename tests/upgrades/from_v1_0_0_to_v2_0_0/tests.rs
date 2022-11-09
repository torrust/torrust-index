//! You can run this test with:
//!
//! ```text
//! cargo test upgrade_data_from_version_v1_0_0_to_v2_0_0 -- --nocapture
//! ```
use crate::upgrades::from_v1_0_0_to_v2_0_0::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use std::fs;
use std::sync::Arc;
use torrust_index_backend::upgrades::from_v1_0_0_to_v2_0_0::upgrader::{upgrade, Arguments};

#[tokio::test]
async fn upgrade_data_from_version_v1_0_0_to_v2_0_0() {
    /* TODO:
     * - Insert data: user, tracker key and torrent
     * - Assertions
     */
    let fixtures_dir = "./tests/upgrades/from_v1_0_0_to_v2_0_0/fixtures/".to_string();
    let debug_output_dir = "./tests/upgrades/from_v1_0_0_to_v2_0_0/output/".to_string();

    let source_database_file = format!("{}source.db", debug_output_dir);
    let destiny_database_file = format!("{}destiny.db", debug_output_dir);

    // TODO: use a unique temporary dir
    fs::remove_file(&source_database_file).expect("Can't remove source DB file.");
    fs::remove_file(&destiny_database_file).expect("Can't remove destiny DB file.");

    let source_database = source_db_connection(&source_database_file).await;

    source_database.migrate(&fixtures_dir).await;

    let args = Arguments {
        source_database_file,
        destiny_database_file,
        upload_path: format!("{}uploads/", fixtures_dir),
    };

    upgrade(&args).await;
}

async fn source_db_connection(source_database_file: &str) -> Arc<SqliteDatabaseV1_0_0> {
    Arc::new(SqliteDatabaseV1_0_0::db_connection(&source_database_file).await)
}
