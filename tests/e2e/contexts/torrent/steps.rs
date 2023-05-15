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
