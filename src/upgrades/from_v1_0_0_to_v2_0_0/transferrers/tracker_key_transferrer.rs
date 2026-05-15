use std::sync::Arc;

use tracing::info;

use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v2_0_0::SqliteDatabaseV2_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::error::UpgradeError;

/// Transfer tracker keys from the source database to the target database.
///
/// # Errors
///
/// Returns an error if reading, inserting, or validating copied tracker-key data
/// fails.
pub async fn transfer_tracker_keys(
    source_database: Arc<SqliteDatabaseV1_0_0>,
    target_database: Arc<SqliteDatabaseV2_0_0>,
) -> Result<(), UpgradeError> {
    info!("transferring tracker keys");

    // Transfer table `torrust_tracker_keys`

    let tracker_keys = source_database
        .get_tracker_keys()
        .await
        .map_err(|source| UpgradeError::Sqlx {
            context: "failed to read source tracker keys",
            source,
        })?;

    for tracker_key in &tracker_keys {
        // [v2] table torrust_tracker_keys

        info!(
            tracker_key_id = tracker_key.key_id,
            user_id = tracker_key.user_id,
            "adding tracker key"
        );

        let id = target_database
            .insert_tracker_key(
                tracker_key.key_id,
                tracker_key.user_id,
                &tracker_key.key,
                tracker_key.valid_until,
            )
            .await
            .map_err(|source| UpgradeError::Sqlx {
                context: "failed to insert tracker key",
                source,
            })?;

        if id != tracker_key.key_id {
            return Err(UpgradeError::IdMismatch {
                entity: "tracker key",
                expected: tracker_key.key_id,
                actual: id,
            });
        }

        info!(
            tracker_key_id = tracker_key.key_id,
            user_id = tracker_key.user_id,
            "tracker key added"
        );
    }

    Ok(())
}
