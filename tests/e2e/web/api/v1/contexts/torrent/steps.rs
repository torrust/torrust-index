use std::str::FromStr;

use torrust_index_backend::models::info_hash::InfoHash;
use torrust_index_backend::web::api::v1::responses::ErrorResponseData;

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
        println!("Error deserializing response: {res:?}");
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
    let canonical_info_hash_hex = uploaded_torrent_response.data.info_hash.to_lowercase();

    let canonical_info_hash = InfoHash::from_str(&canonical_info_hash_hex)
        .unwrap_or_else(|_| panic!("Invalid info-hash in database: {canonical_info_hash_hex}"));

    Ok(canonical_info_hash)
}
