use std::str::FromStr;
use std::sync::Arc;

use torrust_index::databases::database;
use torrust_index::models::info_hash::InfoHash;
use torrust_index::services::torrent::Index;
use torrust_index::web::api::server::v1::responses::ErrorResponseData;

use crate::common::client::Client;
use crate::common::contexts::torrent::fixtures::{random_torrent, TestTorrent, TorrentIndexInfo, TorrentListedInIndex};
use crate::common::contexts::torrent::forms::UploadTorrentMultipartForm;
use crate::common::contexts::torrent::responses::UploadedTorrentResponse;
use crate::common::contexts::user::responses::LoggedInUserData;
use crate::e2e::environment::TestEnv;

/// Add a new random torrent to the index
pub async fn upload_random_torrent_to_index(uploader: &LoggedInUserData, env: &TestEnv) -> (TestTorrent, TorrentListedInIndex) {
    let random_torrent = random_torrent();
    let indexed_torrent = upload_torrent(uploader, &random_torrent.index_info, env).await;
    (random_torrent, indexed_torrent)
}

/// Upload a torrent to the index
pub async fn upload_torrent(uploader: &LoggedInUserData, torrent: &TorrentIndexInfo, env: &TestEnv) -> TorrentListedInIndex {
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

    let form: UploadTorrentMultipartForm = torrent.clone().into();

    let response = client.upload_torrent(form.into()).await;

    let res = serde_json::from_str::<UploadedTorrentResponse>(&response.body);

    if res.is_err() {
        println!("Error deserializing response: {res:?}. Body: {0}", response.body);
    }

    TorrentListedInIndex::from(torrent.clone(), res.unwrap().data.torrent_id)
}

/// Upload a torrent to the index.
///
/// # Errors
///
/// Returns an `ErrorResponseData` if the response is not a 200.
pub async fn upload_test_torrent(client: &Client, test_torrent: &TestTorrent) -> Result<InfoHash, ErrorResponseData> {
    let form: UploadTorrentMultipartForm = test_torrent.clone().index_info.into();
    let response = client.upload_torrent(form.into()).await;

    if response.status != 200 {
        let error: ErrorResponseData = serde_json::from_str(&response.body)
            .unwrap_or_else(|_| panic!("response {:#?} should be a ErrorResponseData", response.body));
        return Err(error);
    }

    let uploaded_torrent_response: UploadedTorrentResponse = serde_json::from_str(&response.body).unwrap();
    let canonical_info_hash_hex = uploaded_torrent_response.data.canonical_info_hash.to_lowercase();

    let canonical_info_hash = InfoHash::from_str(&canonical_info_hash_hex)
        .unwrap_or_else(|_| panic!("Invalid info-hash in database: {canonical_info_hash_hex}"));

    Ok(canonical_info_hash)
}

/// Gets tracker announce urls.
///
/// # Errors
///
/// Returns an `ErrorResponseData` if the response is not a 200.
pub async fn get_trackers(env: &TestEnv, user_id: i64, torrent_id: i64) -> () {
    let database = Arc::new(
        database::connect(&env.database_connect_url().unwrap())
            .await
            .expect("Database error."),
    );

    let announce_urls = database.get_torrent_announce_urls_from_id(torrent_id);

    let user_tracker_key = database.get_user_tracker_key(user_id);

    announce_urls.retain(|tracker| *tracker != tracker_url.to_string());
}

/*   /// It adds the tracker URL in the first position of the tracker list.
   pub fn include_url_as_main_tracker(&mut self, tracker_url: &Url) {
    // Remove any existing instances of tracker_url
    self.trackers.retain(|tracker| *tracker != tracker_url.to_string());

    // Insert tracker_url at the first position
    self.trackers.insert(0, tracker_url.to_owned().to_string());
} */
