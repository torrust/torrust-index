use crate::common::contexts::torrent::fixtures::{random_torrent, TestTorrent, TorrentIndexInfo, TorrentListedInIndex};
use crate::common::contexts::torrent::forms::UploadTorrentMultipartForm;
use crate::common::contexts::torrent::responses::UploadedTorrentResponse;
use crate::common::contexts::user::responses::LoggedInUserData;
use crate::environments::shared::TestEnv;

/// Add a new random torrent to the index
pub async fn upload_random_torrent_to_index(uploader: &LoggedInUserData) -> (TestTorrent, TorrentListedInIndex) {
    let random_torrent = random_torrent();
    let indexed_torrent = upload_torrent(uploader, &random_torrent.index_info).await;
    (random_torrent, indexed_torrent)
}

/// Upload a torrent to the index
pub async fn upload_torrent(uploader: &LoggedInUserData, torrent: &TorrentIndexInfo) -> TorrentListedInIndex {
    let client = TestEnv::running().await.authenticated_client(&uploader.token);

    let form: UploadTorrentMultipartForm = torrent.clone().into();

    let response = client.upload_torrent(form.into()).await;

    let res: UploadedTorrentResponse = serde_json::from_str(&response.body).unwrap();

    TorrentListedInIndex::from(torrent.clone(), res.data.torrent_id)
}
