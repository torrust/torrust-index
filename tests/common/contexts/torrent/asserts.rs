use super::responses::TorrentDetails;

/// Assert that the torrent details match the expected ones.
/// It ignores some fields that are not relevant for the E2E tests
/// or hard to assert due to the concurrent nature of the tests.
pub fn assert_expected_torrent_details(torrent: &TorrentDetails, expected_torrent: &TorrentDetails) {
    assert_eq!(
        torrent.torrent_id, expected_torrent.torrent_id,
        "torrent `file_size` mismatch"
    );
    assert_eq!(torrent.uploader, expected_torrent.uploader, "torrent `uploader` mismatch");
    assert_eq!(torrent.info_hash, expected_torrent.info_hash, "torrent `info_hash` mismatch");
    assert_eq!(torrent.title, expected_torrent.title, "torrent `title` mismatch");
    assert_eq!(
        torrent.description, expected_torrent.description,
        "torrent `description` mismatch"
    );
    assert_eq!(
        torrent.category.category_id, expected_torrent.category.category_id,
        "torrent `category.category_id` mismatch"
    );
    assert_eq!(
        torrent.category.name, expected_torrent.category.name,
        "torrent `category.name` mismatch"
    );
    // assert_eq!(torrent.category.num_torrents, expected_torrent.category.num_torrents, "torrent `category.num_torrents` mismatch"); // Ignored
    // assert_eq!(torrent.upload_date, expected_torrent.upload_date, "torrent `upload_date` mismatch"); // Ignored, can't mock time easily for now.
    assert_eq!(torrent.file_size, expected_torrent.file_size, "torrent `file_size` mismatch");
    assert_eq!(torrent.seeders, expected_torrent.seeders, "torrent `seeders` mismatch");
    assert_eq!(torrent.leechers, expected_torrent.leechers, "torrent `leechers` mismatch");
    assert_eq!(torrent.files, expected_torrent.files, "torrent `files` mismatch");
    assert_eq!(torrent.trackers, expected_torrent.trackers, "torrent `trackers` mismatch");
    assert_eq!(
        torrent.magnet_link, expected_torrent.magnet_link,
        "torrent `magnet_link` mismatch"
    );
}
