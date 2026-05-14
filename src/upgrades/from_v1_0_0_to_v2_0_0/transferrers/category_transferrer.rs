use std::sync::Arc;

use tracing::{debug, info};

use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v2_0_0::{CategoryRecordV2, SqliteDatabaseV2_0_0};
use crate::upgrades::from_v1_0_0_to_v2_0_0::error::UpgradeError;

/// Transfer categories from the source database to the target database.
///
/// # Errors
///
/// Returns an error if reading, inserting, or validating copied category data
/// fails.
pub async fn transfer_categories(
    source_database: Arc<SqliteDatabaseV1_0_0>,
    target_database: Arc<SqliteDatabaseV2_0_0>,
) -> Result<(), UpgradeError> {
    info!("transferring categories");

    let source_categories = source_database
        .get_categories_order_by_id()
        .await
        .map_err(|source| UpgradeError::Database {
            context: "failed to read source categories",
            source,
        })?;
    debug!(?source_categories, "read source categories");

    let result = target_database
        .reset_categories_sequence()
        .await
        .map_err(|source| UpgradeError::Database {
            context: "failed to reset target category sequence",
            source,
        })?;
    debug!(?result, "reset target category sequence");

    for category in &source_categories {
        info!(category_id = category.category_id, name = %category.name, "adding category");
        let id = target_database
            .insert_category(&CategoryRecordV2 {
                category_id: category.category_id,
                name: category.name.clone(),
            })
            .await
            .map_err(|source| UpgradeError::Sqlx {
                context: "failed to insert target category",
                source,
            })?;

        if id != category.category_id {
            return Err(UpgradeError::IdMismatch {
                entity: "category",
                expected: category.category_id,
                actual: id,
            });
        }

        info!(category_id = id, name = %category.name, "category added");
    }

    let target_categories = target_database
        .get_categories()
        .await
        .map_err(|source| UpgradeError::Database {
            context: "failed to read target categories",
            source,
        })?;
    debug!(?target_categories, "read target categories");

    Ok(())
}
