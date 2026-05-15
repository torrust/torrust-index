use std::sync::Arc;

use tracing::info;

use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v2_0_0::SqliteDatabaseV2_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::error::UpgradeError;

/// Transfer users and their related profile/authentication records.
///
/// # Errors
///
/// Returns an error if reading, inserting, or validating copied user data fails.
pub async fn transfer_users(
    source_database: Arc<SqliteDatabaseV1_0_0>,
    target_database: Arc<SqliteDatabaseV2_0_0>,
    date_imported: &str,
) -> Result<(), UpgradeError> {
    info!("transferring users");

    // Transfer table `torrust_users`

    let users = source_database.get_users().await.map_err(|source| UpgradeError::Sqlx {
        context: "failed to read source users",
        source,
    })?;

    for user in &users {
        // [v2] table torrust_users

        info!(user_id = user.user_id, username = %user.username, "adding user");

        let id = target_database
            .insert_imported_user(user.user_id, date_imported, user.administrator)
            .await
            .map_err(|source| UpgradeError::Sqlx {
                context: "failed to insert imported user",
                source,
            })?;

        if id != user.user_id {
            return Err(UpgradeError::IdMismatch {
                entity: "user",
                expected: user.user_id,
                actual: id,
            });
        }

        info!(user_id = user.user_id, username = %user.username, "user added");

        // [v2] table torrust_user_profiles

        info!(user_id = user.user_id, username = %user.username, "adding user profile");

        target_database
            .insert_user_profile(user.user_id, &user.username, &user.email, user.email_verified)
            .await
            .map_err(|source| UpgradeError::Sqlx {
                context: "failed to insert user profile",
                source,
            })?;

        info!(user_id = user.user_id, username = %user.username, "user profile added");

        // [v2] table torrust_user_authentication

        info!(user_id = user.user_id, "adding user authentication");

        target_database
            .insert_user_password_hash(user.user_id, &user.password)
            .await
            .map_err(|source| UpgradeError::Sqlx {
                context: "failed to insert user authentication",
                source,
            })?;

        info!(user_id = user.user_id, "user authentication added");
    }

    Ok(())
}
