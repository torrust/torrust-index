#![allow(clippy::missing_errors_doc)]

use std::sync::Arc;
use std::{error, fs};

use crate::models::torrent_file::Torrent;
use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v2_0_0::{SqliteDatabaseV2_0_0, TorrentRecordV2};
use crate::utils::parse_torrent::decode_torrent;

#[allow(clippy::missing_panics_doc)]
#[allow(clippy::too_many_lines)]
pub async fn transfer_torrents(
    source_database: Arc<SqliteDatabaseV1_0_0>,
    target_database: Arc<SqliteDatabaseV2_0_0>,
    upload_path: &str,
) {
    println!("Transferring torrents ...");

    // Transfer table `torrust_torrents_files`

    // Although the The table `torrust_torrents_files` existed in version v1.0.0
    // it was was not used.

    // Transfer table `torrust_torrents`

    let torrents = source_database.get_torrents().await.unwrap();

    for torrent in &torrents {
        // [v2] table torrust_torrents

        println!("[v2][torrust_torrents] adding the torrent: {:?} ...", &torrent.torrent_id);

        let uploader = source_database.get_user_by_username(&torrent.uploader).await.unwrap();

        assert!(
            uploader.username == torrent.uploader,
            "Error copying torrent with id {:?}.
                Username (`uploader`) in `torrust_torrents` table does not match `username` in `torrust_users` table",
            &torrent.torrent_id
        );

        let filepath = format!("{}/{}.torrent", upload_path, &torrent.torrent_id);

        let torrent_from_file_result = read_torrent_from_file(&filepath);

        assert!(
            torrent_from_file_result.is_ok(),
            "Error torrent file not found: {:?}",
            &filepath
        );

        let torrent_from_file = torrent_from_file_result.unwrap();

        let id = target_database
            .insert_torrent(&TorrentRecordV2::from_v1_data(torrent, &torrent_from_file.info, &uploader))
            .await
            .unwrap();

        assert!(
            id == torrent.torrent_id,
            "Error copying torrent {:?} from source DB to the target DB",
            &torrent.torrent_id
        );

        println!("[v2][torrust_torrents] torrent with id {:?} added.", &torrent.torrent_id);

        // [v2] table torrust_torrent_files

        println!("[v2][torrust_torrent_files] adding torrent files");

        if torrent_from_file.is_a_single_file_torrent() {
            // The torrent contains only one file then:
            // - "path" is NULL
            // - "md5sum" can be NULL

            println!(
                "[v2][torrust_torrent_files][single-file-torrent] adding torrent file {:?} with length {:?} ...",
                &torrent_from_file.info.name, &torrent_from_file.info.length,
            );

            let file_id = target_database
                .insert_torrent_file_for_torrent_with_one_file(
                    torrent.torrent_id,
                    // TODO: it seems med5sum can be None. Why? When?
                    &torrent_from_file.info.md5sum.clone(),
                    torrent_from_file.info.length.unwrap(),
                )
                .await;

            println!(
                "[v2][torrust_torrent_files][single-file-torrent] torrent file insert result: {:?}",
                &file_id
            );
        } else {
            // Multiple files are being shared
            let files = torrent_from_file.info.files.as_ref().unwrap();

            for file in files.iter() {
                println!(
                    "[v2][torrust_torrent_files][multiple-file-torrent] adding torrent file: {:?} ...",
                    &file
                );

                let file_id = target_database
                    .insert_torrent_file_for_torrent_with_multiple_files(torrent, file)
                    .await;

                println!(
                    "[v2][torrust_torrent_files][multiple-file-torrent] torrent file insert result: {:?}",
                    &file_id
                );
            }
        }

        // [v2] table torrust_torrent_info

        println!(
            "[v2][torrust_torrent_info] adding the torrent info for torrent id {:?} ...",
            &torrent.torrent_id
        );

        let id = target_database.insert_torrent_info(torrent).await;

        println!("[v2][torrust_torrents] torrent info insert result: {:?}.", &id);

        // [v2] table torrust_torrent_announce_urls

        println!(
            "[v2][torrust_torrent_announce_urls] adding the torrent announce url for torrent id {:?} ...",
            &torrent.torrent_id
        );

        if torrent_from_file.announce_list.is_some() {
            // BEP-0012. Multiple trackers.

            println!(
                "[v2][torrust_torrent_announce_urls][announce-list] adding the torrent announce url for torrent id {:?} ...",
                &torrent.torrent_id
            );

            // flatten the nested vec (this will however remove the)
            let announce_urls = torrent_from_file
                .announce_list
                .clone()
                .unwrap()
                .into_iter()
                .flatten()
                .collect::<Vec<String>>();

            for tracker_url in &announce_urls {
                println!(
                    "[v2][torrust_torrent_announce_urls][announce-list] adding the torrent announce url for torrent id {:?} ...",
                    &torrent.torrent_id
                );

                let announce_url_id = target_database
                    .insert_torrent_announce_url(torrent.torrent_id, tracker_url)
                    .await;

                println!(
                    "[v2][torrust_torrent_announce_urls][announce-list] torrent announce url insert result {:?} ...",
                    &announce_url_id
                );
            }
        } else if torrent_from_file.announce.is_some() {
            println!(
                "[v2][torrust_torrent_announce_urls][announce] adding the torrent announce url for torrent id {:?} ...",
                &torrent.torrent_id
            );

            let announce_url_id = target_database
                .insert_torrent_announce_url(torrent.torrent_id, &torrent_from_file.announce.unwrap())
                .await;

            println!(
                "[v2][torrust_torrent_announce_urls][announce] torrent announce url insert result {:?} ...",
                &announce_url_id
            );
        }
    }
    println!("Torrents transferred");
}

pub fn read_torrent_from_file(path: &str) -> Result<Torrent, Box<dyn error::Error>> {
    let contents = match fs::read(path) {
        Ok(contents) => contents,
        Err(e) => return Err(e.into()),
    };

    match decode_torrent(&contents) {
        Ok(torrent) => Ok(torrent),
        Err(e) => Err(e),
    }
}
