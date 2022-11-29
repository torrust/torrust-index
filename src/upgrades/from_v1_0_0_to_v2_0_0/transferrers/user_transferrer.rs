use std::sync::Arc;

use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v2_0_0::SqliteDatabaseV2_0_0;

pub async fn transfer_users(
    source_database: Arc<SqliteDatabaseV1_0_0>,
    dest_database: Arc<SqliteDatabaseV2_0_0>,
    date_imported: &str,
) {
    println!("Transferring users ...");

    // Transfer table `torrust_users`

    let users = source_database.get_users().await.unwrap();

    for user in &users {
        // [v2] table torrust_users

        println!(
            "[v2][torrust_users] adding user with username {:?} and id {:?} ...",
            &user.username, &user.user_id
        );

        let id = dest_database
            .insert_imported_user(user.user_id, date_imported, user.administrator)
            .await
            .unwrap();

        if id != user.user_id {
            panic!("Error copying user {:?} from source DB to destiny DB", &user.user_id);
        }

        println!("[v2][torrust_users] user: {:?} {:?} added.", &user.user_id, &user.username);

        // [v2] table torrust_user_profiles

        println!(
            "[v2][torrust_user_profiles] adding user profile for user with username {:?} and id {:?} ...",
            &user.username, &user.user_id
        );

        dest_database
            .insert_user_profile(user.user_id, &user.username, &user.email, user.email_verified)
            .await
            .unwrap();

        println!(
            "[v2][torrust_user_profiles] user profile added for user with username {:?} and id {:?}.",
            &user.username, &user.user_id
        );

        // [v2] table torrust_user_authentication

        println!(
            "[v2][torrust_user_authentication] adding password hash ({:?}) for user id ({:?}) ...",
            &user.password, &user.user_id
        );

        dest_database
            .insert_user_password_hash(user.user_id, &user.password)
            .await
            .unwrap();

        println!(
            "[v2][torrust_user_authentication] password hash ({:?}) added for user id ({:?}).",
            &user.password, &user.user_id
        );
    }
}
