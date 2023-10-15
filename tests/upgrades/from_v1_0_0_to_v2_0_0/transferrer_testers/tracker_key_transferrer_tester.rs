use std::sync::Arc;

use torrust_index::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v1_0_0::TrackerKeyRecordV1;

use crate::upgrades::from_v1_0_0_to_v2_0_0::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::sqlite_v2_0_0::SqliteDatabaseV2_0_0;

pub struct TrackerKeyTester {
    source_database: Arc<SqliteDatabaseV1_0_0>,
    target_database: Arc<SqliteDatabaseV2_0_0>,
    test_data: TestData,
}

pub struct TestData {
    pub tracker_key: TrackerKeyRecordV1,
}

impl TrackerKeyTester {
    pub fn new(source_database: Arc<SqliteDatabaseV1_0_0>, target_database: Arc<SqliteDatabaseV2_0_0>, user_id: i64) -> Self {
        let tracker_key = TrackerKeyRecordV1 {
            key_id: 1,
            user_id,
            key: "rRstSTM5rx0sgxjLkRSJf3rXODcRBI5T".to_string(),
            valid_until: 2_456_956_800, // 11-10-2047 00:00:00 UTC
        };

        Self {
            source_database,
            target_database,
            test_data: TestData { tracker_key },
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub async fn load_data_into_source_db(&self) {
        self.source_database
            .insert_tracker_key(&self.test_data.tracker_key)
            .await
            .unwrap();
    }

    #[allow(clippy::missing_panics_doc)]
    /// Table `torrust_tracker_keys`
    pub async fn assert_data_in_target_db(&self) {
        let imported_key = self
            .target_database
            .get_tracker_key(self.test_data.tracker_key.key_id)
            .await
            .unwrap();

        assert_eq!(imported_key.tracker_key_id, self.test_data.tracker_key.key_id);
        assert_eq!(imported_key.user_id, self.test_data.tracker_key.user_id);
        assert_eq!(imported_key.tracker_key, self.test_data.tracker_key.key);
        assert_eq!(imported_key.date_expiry, self.test_data.tracker_key.valid_until);
    }
}
