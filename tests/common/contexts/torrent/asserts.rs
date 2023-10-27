use super::responses::TorrentDetails;

type Check = (&'static str, bool);

/// Assert that the torrent details match the expected ones.
///
/// It ignores some fields that are not relevant for the E2E tests
/// or hard to assert due to the concurrent nature of the tests.
pub fn assert_expected_torrent_details(torrent: &TorrentDetails, expected_torrent: &TorrentDetails) {
    let mut discrepancies = Vec::new();

    let checks: Vec<Check> = vec![
        ("torrent_id", torrent.torrent_id == expected_torrent.torrent_id),
        ("uploader", torrent.uploader == expected_torrent.uploader),
        ("info_hash", torrent.info_hash == expected_torrent.info_hash),
        ("title", torrent.title == expected_torrent.title),
        ("description", torrent.description == expected_torrent.description),
        ("category.category_id", torrent.category.id == expected_torrent.category.id),
        ("category.name", torrent.category.name == expected_torrent.category.name),
        ("file_size", torrent.file_size == expected_torrent.file_size),
        ("seeders", torrent.seeders == expected_torrent.seeders),
        ("leechers", torrent.leechers == expected_torrent.leechers),
        ("files", torrent.files == expected_torrent.files),
        ("trackers", torrent.trackers == expected_torrent.trackers),
        ("magnet_link", torrent.magnet_link == expected_torrent.magnet_link),
        ("tags", torrent.tags == expected_torrent.tags),
        ("name", torrent.name == expected_torrent.name),
    ];

    for (field_name, equals) in &checks {
        if !equals {
            discrepancies.push((*field_name).to_string());
        }
    }

    let error_message = format!("left:\n{torrent:#?}\nright:\n{expected_torrent:#?}\ndiscrepancies: {discrepancies:#?}");

    assert!(discrepancies.is_empty(), "{}", error_message);
}
