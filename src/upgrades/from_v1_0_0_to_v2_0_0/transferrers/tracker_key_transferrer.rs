use std::sync::Arc;

use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v2_0_0::SqliteDatabaseV2_0_0;

#[allow(clippy::missing_panics_doc)]
pub async fn transfer_tracker_keys(source_database: Arc<SqliteDatabaseV1_0_0>, target_database: Arc<SqliteDatabaseV2_0_0>) {
    println!("Transferring tracker keys ...");

    // Transfer table `torrust_tracker_keys`

    let tracker_keys = source_database.get_tracker_keys().await.unwrap();

    for tracker_key in &tracker_keys {
        // [v2] table torrust_tracker_keys

        println!(
            "[v2][torrust_users] adding the tracker key with id {:?} ...",
            &tracker_key.key_id
        );

        let id = target_database
            .insert_tracker_key(
                tracker_key.key_id,
                tracker_key.user_id,
                &tracker_key.key,
                tracker_key.valid_until,
            )
            .await
            .unwrap();

        assert!(
            id == tracker_key.key_id,
            "Error copying tracker key {:?} from source DB to the target DB",
            &tracker_key.key_id
        );

        println!(
            "[v2][torrust_tracker_keys] tracker key with id {:?} added.",
            &tracker_key.key_id
        );
    }
}
