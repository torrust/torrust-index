use std::sync::Arc;

use self::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use self::sqlite_v2_0_0::SqliteDatabaseV2_0_0;

pub mod sqlite_v1_0_0;
pub mod sqlite_v2_0_0;

pub async fn current_db(db_filename: &str) -> Arc<SqliteDatabaseV1_0_0> {
    let source_database_connect_url = format!("sqlite://{db_filename}?mode=ro");
    Arc::new(SqliteDatabaseV1_0_0::new(&source_database_connect_url).await)
}

pub async fn new_db(db_filename: &str) -> Arc<SqliteDatabaseV2_0_0> {
    let target_database_connect_url = format!("sqlite://{db_filename}?mode=rwc");
    Arc::new(SqliteDatabaseV2_0_0::new(&target_database_connect_url).await)
}

pub async fn migrate_target_database(target_database: Arc<SqliteDatabaseV2_0_0>) {
    println!("Running migrations in the target database...");
    target_database.migrate().await;
}

/// It truncates all tables in the target database.
///
/// # Panics
///
/// It panics if it cannot truncate the tables.
pub async fn truncate_target_database(target_database: Arc<SqliteDatabaseV2_0_0>) {
    println!("Truncating all tables in target database ...");
    target_database
        .delete_all_database_rows()
        .await
        .expect("Can't reset the target database.");
}
