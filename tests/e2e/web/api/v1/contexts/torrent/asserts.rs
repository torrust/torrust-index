use std::sync::Arc;

use torrust_index_backend::databases::database;
use torrust_index_backend::models::torrent_file::Torrent;
use torrust_index_backend::models::tracker_key::TrackerKey;

use crate::common::contexts::user::responses::LoggedInUserData;
use crate::e2e::environment::TestEnv;

/// The backend does not generate exactly the same torrent that was uploaded.
///
/// The backend stores the canonical version of the uploaded torrent. So we need
/// to update the expected torrent to match the one generated by the backend.
pub async fn canonical_torrent_for(
    mut uploaded_torrent: Torrent,
    env: &TestEnv,
    downloader: &Option<LoggedInUserData>,
) -> Torrent {
    let tracker_url = env.server_settings().unwrap().tracker.url.to_string();

    let tracker_key = match downloader {
        Some(logged_in_user) => get_user_tracker_key(logged_in_user, env).await,
        None => None,
    };

    uploaded_torrent.announce = Some(build_announce_url(&tracker_url, &tracker_key));
    uploaded_torrent.announce_list = Some(build_announce_list(&tracker_url, &tracker_key));

    // These fields are not persisted in the database yet.
    // See https://github.com/torrust/torrust-index-backend/issues/284
    // They are ignore when the user uploads the torrent. So the stored
    // canonical torrent does not contain them.
    uploaded_torrent.encoding = None;
    uploaded_torrent.creation_date = None;
    uploaded_torrent.created_by = None;

    uploaded_torrent
}

pub async fn get_user_tracker_key(logged_in_user: &LoggedInUserData, env: &TestEnv) -> Option<TrackerKey> {
    // code-review: could we add a new endpoint to get the user's tracker key?
    // `/user/keys/recent` or `/user/keys/latest
    // We could use that endpoint to get the user's tracker key instead of
    // querying the database.

    let database = Arc::new(
        database::connect(&env.database_connect_url().unwrap())
            .await
            .expect("database connection to be established."),
    );

    // Get the logged-in user id
    let user_profile = database
        .get_user_profile_from_username(&logged_in_user.username)
        .await
        .unwrap();

    // Get the user's tracker key
    let tracker_key = database
        .get_user_tracker_key(user_profile.user_id)
        .await
        .expect("user to have a tracker key");

    Some(tracker_key)
}

pub fn build_announce_url(tracker_url: &str, tracker_key: &Option<TrackerKey>) -> String {
    if let Some(key) = &tracker_key {
        format!("{tracker_url}/{}", key.key)
    } else {
        tracker_url.to_string()
    }
}

fn build_announce_list(tracker_url: &str, tracker_key: &Option<TrackerKey>) -> Vec<Vec<String>> {
    if let Some(key) = &tracker_key {
        vec![vec![format!("{tracker_url}/{}", key.key)], vec![format!("{tracker_url}")]]
    } else {
        vec![vec![format!("{tracker_url}")]]
    }
}
