use std::sync::Arc;

use tracing::info;

use self::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use self::sqlite_v2_0_0::SqliteDatabaseV2_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::error::UpgradeError;

pub mod sqlite_v1_0_0;
pub mod sqlite_v2_0_0;

/// Open the source v1.0.0 `SQLite` database in read-only mode.
///
/// # Errors
///
/// Returns an error if the database pool cannot be created.
pub async fn current_db(db_filename: &str) -> Result<Arc<SqliteDatabaseV1_0_0>, UpgradeError> {
    let source_database_connect_url = format!("sqlite://{db_filename}?mode=ro");
    let database = SqliteDatabaseV1_0_0::new(&source_database_connect_url)
        .await
        .map_err(|source| UpgradeError::Sqlx {
            context: "failed to open source database",
            source,
        })?;
    Ok(Arc::new(database))
}

/// Open the target v2.0.0 `SQLite` database.
///
/// # Errors
///
/// Returns an error if the database pool cannot be created.
pub async fn new_db(db_filename: &str) -> Result<Arc<SqliteDatabaseV2_0_0>, UpgradeError> {
    let target_database_connect_url = format!("sqlite://{db_filename}?mode=rwc");
    let database = SqliteDatabaseV2_0_0::new(&target_database_connect_url)
        .await
        .map_err(|source| UpgradeError::Sqlx {
            context: "failed to open target database",
            source,
        })?;
    Ok(Arc::new(database))
}

/// Run target database migrations.
///
/// # Errors
///
/// Returns an error if any migration fails.
pub async fn migrate_target_database(target_database: Arc<SqliteDatabaseV2_0_0>) -> Result<(), UpgradeError> {
    info!("running migrations in the target database");
    target_database.migrate().await.map_err(|source| UpgradeError::Migration {
        context: "failed to run target database migrations",
        source,
    })
}

/// It truncates all tables in the target database.
///
/// # Errors
///
/// Returns an error if any target database table cannot be truncated.
pub async fn truncate_target_database(target_database: Arc<SqliteDatabaseV2_0_0>) -> Result<(), UpgradeError> {
    info!("truncating all tables in target database");
    target_database
        .delete_all_database_rows()
        .await
        .map_err(|source| UpgradeError::Database {
            context: "failed to truncate target database",
            source,
        })
}
