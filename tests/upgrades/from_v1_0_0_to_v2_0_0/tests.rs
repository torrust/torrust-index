//! You can run this test with:
//!
//! ```text
//! cargo test upgrade_data_from_version_v1_0_0_to_v2_0_0 -- --nocapture
//! ```
use std::fs;
use std::sync::Arc;
use torrust_index_backend::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
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

    migrate(source_database.clone(), &fixtures_dir).await;

    let args = Arguments {
        source_database_file,
        destiny_database_file,
        upload_path: format!("{}uploads/", fixtures_dir),
    };

    upgrade(&args).await;
}

async fn source_db_connection(source_database_file: &str) -> Arc<SqliteDatabaseV1_0_0> {
    let source_database_connect_url = format!("sqlite://{}?mode=rwc", source_database_file);
    Arc::new(SqliteDatabaseV1_0_0::new(&source_database_connect_url).await)
}

/// Execute migrations for database in version v1.0.0
async fn migrate(source_database: Arc<SqliteDatabaseV1_0_0>, fixtures_dir: &str) {
    let migrations_dir = format!("{}database/v1.0.0/migrations/", fixtures_dir);

    let migrations = vec![
        "20210831113004_torrust_users.sql",
        "20210904135524_torrust_tracker_keys.sql",
        "20210905160623_torrust_categories.sql",
        "20210907083424_torrust_torrent_files.sql",
        "20211208143338_torrust_users.sql",
        "20220308083424_torrust_torrents.sql",
        "20220308170028_torrust_categories.sql",
    ];

    for migration_file_name in &migrations {
        let migration_file_path = format!("{}{}", &migrations_dir, &migration_file_name);
        run_migration_from_file(source_database.clone(), &migration_file_path).await;
    }
}

async fn run_migration_from_file(
    source_database: Arc<SqliteDatabaseV1_0_0>,
    migration_file_path: &str,
) {
    println!("Executing migration: {:?}", migration_file_path);

    let sql =
        fs::read_to_string(migration_file_path).expect("Should have been able to read the file");

    let res = sqlx::query(&sql).execute(&source_database.pool).await;

    println!("Migration result {:?}", res);
}
