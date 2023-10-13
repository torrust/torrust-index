use std::sync::Arc;

use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand_core::OsRng;
use torrust_index::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v1_0_0::UserRecordV1;

use crate::upgrades::from_v1_0_0_to_v2_0_0::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::sqlite_v2_0_0::SqliteDatabaseV2_0_0;

pub struct UserTester {
    source_database: Arc<SqliteDatabaseV1_0_0>,
    target_database: Arc<SqliteDatabaseV2_0_0>,
    execution_time: String,
    pub test_data: TestData,
}

pub struct TestData {
    pub user: UserRecordV1,
}

impl UserTester {
    pub fn new(
        source_database: Arc<SqliteDatabaseV1_0_0>,
        target_database: Arc<SqliteDatabaseV2_0_0>,
        execution_time: &str,
    ) -> Self {
        let user = UserRecordV1 {
            user_id: 1,
            username: "user01".to_string(),
            email: "user01@torrust.com".to_string(),
            email_verified: true,
            password: hashed_valid_password(),
            administrator: true,
        };

        Self {
            source_database,
            target_database,
            execution_time: execution_time.to_owned(),
            test_data: TestData { user },
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub async fn load_data_into_source_db(&self) {
        self.source_database.insert_user(&self.test_data.user).await.unwrap();
    }

    pub async fn assert_data_in_target_db(&self) {
        self.assert_user().await;
        self.assert_user_profile().await;
        self.assert_user_authentication().await;
    }

    /// Table `torrust_users`
    async fn assert_user(&self) {
        let imported_user = self.target_database.get_user(self.test_data.user.user_id).await.unwrap();

        assert_eq!(imported_user.user_id, self.test_data.user.user_id);
        assert!(imported_user.date_registered.is_none());
        assert_eq!(imported_user.date_imported.unwrap(), self.execution_time);
        assert_eq!(imported_user.administrator, self.test_data.user.administrator);
    }

    /// Table `torrust_user_profiles`
    async fn assert_user_profile(&self) {
        let imported_user_profile = self
            .target_database
            .get_user_profile(self.test_data.user.user_id)
            .await
            .unwrap();

        assert_eq!(imported_user_profile.user_id, self.test_data.user.user_id);
        assert_eq!(imported_user_profile.username, self.test_data.user.username);
        assert_eq!(imported_user_profile.email, self.test_data.user.email);
        assert_eq!(imported_user_profile.email_verified, self.test_data.user.email_verified);
        assert!(imported_user_profile.bio.is_none());
        assert!(imported_user_profile.avatar.is_none());
    }

    /// Table `torrust_user_profiles`
    async fn assert_user_authentication(&self) {
        let imported_user_authentication = self
            .target_database
            .get_user_authentication(self.test_data.user.user_id)
            .await
            .unwrap();

        assert_eq!(imported_user_authentication.user_id, self.test_data.user.user_id);
        assert_eq!(imported_user_authentication.password_hash, self.test_data.user.password);
    }
}

fn hashed_valid_password() -> String {
    hash_password(&valid_password())
}

fn valid_password() -> String {
    "123456".to_string()
}

fn hash_password(plain_password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    argon2.hash_password(plain_password.as_bytes(), &salt).unwrap().to_string()
}
