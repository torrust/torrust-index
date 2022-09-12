use serde_bytes::ByteBuf;
use torrust_index_backend::databases::database::{Database, DatabaseError};
use torrust_index_backend::models::torrent::TorrentListing;
use torrust_index_backend::models::torrent_file::{TorrentInfo, Torrent};
use torrust_index_backend::models::user::UserProfile;

// test user options
const TEST_USER_USERNAME: &str = "luckythelab";
const TEST_USER_EMAIL: &str = "lucky@labradormail.com";
const TEST_USER_PASSWORD: &str = "imagoodboy";

// test category options
const TEST_CATEGORY_NAME: &str = "Labrador Retrievers";

// test torrent options
const TEST_TORRENT_TITLE: &str = "Picture of dog treat";
const TEST_TORRENT_DESCRIPTION: &str = "This is a picture of a dog treat.";
const TEST_TORRENT_FILE_SIZE: i64 = 128_000;
const TEST_TORRENT_SEEDERS: i64 = 437;
const TEST_TORRENT_LEECHERS: i64 = 1289;

async fn add_test_user(db: &Box<dyn Database>) -> Result<i64, DatabaseError> {
    db.insert_user_and_get_id(TEST_USER_USERNAME, TEST_USER_EMAIL, TEST_USER_PASSWORD).await
}

async fn add_test_torrent_category(db: &Box<dyn Database>) -> Result<i64, DatabaseError> {
    db.insert_category_and_get_id(TEST_CATEGORY_NAME).await
}

pub async fn it_can_add_a_user(db: &Box<dyn Database>) {
    let add_test_user_result = add_test_user(&db).await;

    assert!(add_test_user_result.is_ok());

    let inserted_user_id = add_test_user_result.unwrap();

    let get_user_profile_from_username_result = db.get_user_profile_from_username(TEST_USER_USERNAME).await;

    // verify that we can grab the newly inserted user's profile data
    assert!(get_user_profile_from_username_result.is_ok());

    let returned_user_profile = get_user_profile_from_username_result.unwrap();

    // verify that the profile data is as we expect it to be
    assert_eq!(returned_user_profile, UserProfile {
        user_id: inserted_user_id,
        username: TEST_USER_USERNAME.to_string(),
        email: TEST_USER_EMAIL.to_string(),
        email_verified: returned_user_profile.email_verified.clone(),
        bio: returned_user_profile.bio.clone(),
        avatar: returned_user_profile.avatar.clone()
    });
}

pub async fn it_can_add_a_torrent_category(db: &Box<dyn Database>) {
    let add_test_torrent_category_result = add_test_torrent_category(&db).await;

    assert!(add_test_torrent_category_result.is_ok());

    let get_category_from_name_result = db.get_category_from_name(TEST_CATEGORY_NAME).await;

    assert!(get_category_from_name_result.is_ok());

    let category = get_category_from_name_result.unwrap();

    assert_eq!(category.name, TEST_CATEGORY_NAME.to_string());
}

pub async fn it_can_add_a_torrent_and_tracker_stats_to_that_torrent(db: &Box<dyn Database>) {
    // set pre-conditions
    let user_id = add_test_user(&db).await.expect("add_test_user failed.");
    let torrent_category_id = add_test_torrent_category(&db).await.expect("add_test_torrent_category failed.");

    let torrent = Torrent {
        info: TorrentInfo {
            name: TEST_TORRENT_TITLE.to_string(),
            pieces: Some(ByteBuf::from("1234567890123456789012345678901234567890".as_bytes())),
            piece_length: 256000,
            md5sum: None,
            length: Some(TEST_TORRENT_FILE_SIZE),
            files: None,
            private: Some(1),
            path: None,
            root_hash: None
        },
        announce: Some("https://tracker.dutchbits.nl/announce".to_string()),
        nodes: None,
        encoding: None,
        httpseeds: None,
        announce_list: None,
        creation_date: None,
        comment: None,
        created_by: None
    };

    let insert_torrent_and_get_id_result = db.insert_torrent_and_get_id(
        &torrent,
        user_id,
        torrent_category_id,
        TEST_TORRENT_TITLE,
        TEST_TORRENT_DESCRIPTION
    ).await;

    assert!(insert_torrent_and_get_id_result.is_ok());

    let torrent_id = insert_torrent_and_get_id_result.unwrap();

    // add tracker stats to the torrent
    let insert_torrent_tracker_stats_result = db.update_tracker_info(
        torrent_id,
        "https://tracker.torrust.com",
        TEST_TORRENT_SEEDERS,
        TEST_TORRENT_LEECHERS).await;

    assert!(insert_torrent_tracker_stats_result.is_ok());

    let get_torrent_listing_from_id_result = db.get_torrent_listing_from_id(torrent_id).await;

    assert!(get_torrent_listing_from_id_result.is_ok());

    let returned_torrent_listing = get_torrent_listing_from_id_result.unwrap();

    assert_eq!(returned_torrent_listing, TorrentListing {
        torrent_id,
        uploader: TEST_USER_USERNAME.to_string(),
        info_hash: returned_torrent_listing.info_hash.to_string(),
        title: TEST_TORRENT_TITLE.to_string(),
        description: Some(TEST_TORRENT_DESCRIPTION.to_string()),
        category_id: torrent_category_id,
        date_uploaded: returned_torrent_listing.date_uploaded.to_string(),
        file_size: TEST_TORRENT_FILE_SIZE,
        seeders: TEST_TORRENT_SEEDERS,
        leechers: TEST_TORRENT_LEECHERS
    });

    // check if we get the same info hash on the retrieved torrent from database
    let get_torrent_from_id_result = db.get_torrent_from_id(torrent_id).await;

    assert!(get_torrent_from_id_result.is_ok());

    let returned_torrent = get_torrent_from_id_result.unwrap();

    assert_eq!(returned_torrent.info_hash(), torrent.info_hash());
}
