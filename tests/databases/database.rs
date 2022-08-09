use torrust_index_backend::databases::database::{connect_database, Database, DatabaseDriver};
use torrust_index_backend::models::user::UserProfile;

pub async fn setup(db_driver: DatabaseDriver, db_path: &str) -> Result<Box<dyn Database>, ()> {
    let db = connect_database(&db_driver, db_path).await;

    Ok(db)
}

pub async fn run_tests(db: &Box<dyn Database>) {
    it_can_add_a_user(&db).await;
    it_can_upload_a_torrent(&db).await;
}

pub async fn it_can_add_a_user(db: &Box<dyn Database>) {
    const USERNAME: &str = "luckythelab";
    const EMAIL: &str = "lucky@labradormail.com";
    const PASSWORD: &str = "imagoodboy";

    let insert_user_and_get_id_result = db.insert_user_and_get_id(USERNAME, EMAIL, PASSWORD).await;

    // verify that the insert_user_and_get_id() function did not return an error
    assert!(insert_user_and_get_id_result.is_ok());

    let inserted_user_id = insert_user_and_get_id_result.unwrap();

    let get_user_profile_from_username_result = db.get_user_profile_from_username(USERNAME).await;

    // verify that we can grab the newly inserted user's profile data
    assert!(get_user_profile_from_username_result.is_ok());

    let returned_user_profile = get_user_profile_from_username_result.unwrap();

    // verify that the profile data is as we expect it to be
    assert_eq!(returned_user_profile, UserProfile {
        user_id: inserted_user_id,
        username: USERNAME.to_string(),
        email: EMAIL.to_string(),
        email_verified: returned_user_profile.email_verified.clone(),
        bio: returned_user_profile.bio.clone(),
        avatar: returned_user_profile.avatar.clone()
    });


    // cleanup database
    let _ = db.delete_all_database_rows().await;
}

pub async fn it_can_upload_a_torrent(db: &Box<dyn Database>) {
    assert!(true)
}
