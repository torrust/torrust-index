#![allow(clippy::missing_errors_doc)]

use std::fs;
use std::sync::Arc;

use tracing::{debug, info};

use crate::models::torrent_file::Torrent;
use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v2_0_0::{SqliteDatabaseV2_0_0, TorrentRecordV2};
use crate::upgrades::from_v1_0_0_to_v2_0_0::error::UpgradeError;
use crate::utils::parse_torrent::decode_torrent;

#[allow(clippy::too_many_lines)]
pub async fn transfer_torrents(
    source_database: Arc<SqliteDatabaseV1_0_0>,
    target_database: Arc<SqliteDatabaseV2_0_0>,
    upload_path: &str,
) -> Result<(), UpgradeError> {
    info!("transferring torrents");

    // Transfer table `torrust_torrents_files`

    // Although the The table `torrust_torrents_files` existed in version v1.0.0
    // it was was not used.

    // Transfer table `torrust_torrents`

    let torrents = source_database.get_torrents().await.map_err(|source| UpgradeError::Sqlx {
        context: "failed to read source torrents",
        source,
    })?;

    for torrent in &torrents {
        // [v2] table torrust_torrents

        info!(torrent_id = torrent.torrent_id, "adding torrent");

        let uploader = source_database
            .get_user_by_username(&torrent.uploader)
            .await
            .map_err(|source| UpgradeError::Sqlx {
                context: "failed to read torrent uploader",
                source,
            })?;

        if uploader.username != torrent.uploader {
            return Err(UpgradeError::UploaderMismatch {
                torrent_id: torrent.torrent_id,
                expected: torrent.uploader.clone(),
                actual: uploader.username,
            });
        }

        let filepath = format!("{}/{}.torrent", upload_path, torrent.torrent_id);

        let torrent_from_file = read_torrent_from_file(&filepath)?;

        let target_torrent = TorrentRecordV2::from_v1_data(torrent, &torrent_from_file.info, &uploader)?;
        let id = target_database
            .insert_torrent(&target_torrent)
            .await
            .map_err(|source| UpgradeError::Sqlx {
                context: "failed to insert torrent",
                source,
            })?;

        if id != torrent.torrent_id {
            return Err(UpgradeError::IdMismatch {
                entity: "torrent",
                expected: torrent.torrent_id,
                actual: id,
            });
        }

        info!(torrent_id = torrent.torrent_id, "torrent added");

        // [v2] table torrust_torrent_files

        info!(torrent_id = torrent.torrent_id, "adding torrent files");

        if torrent_from_file.is_a_single_file_torrent() {
            // The torrent contains only one file then:
            // - "path" is NULL
            // - "md5sum" can be NULL

            let length = torrent_from_file.info.length.ok_or(UpgradeError::MissingTorrentField {
                torrent_id: torrent.torrent_id,
                field: "length",
            })?;
            info!(torrent_id = torrent.torrent_id, name = %torrent_from_file.info.name, length, "adding single-file torrent entry");

            let file_id = target_database
                .insert_torrent_file_for_torrent_with_one_file(
                    torrent.torrent_id,
                    // TODO: it seems med5sum can be None. Why? When?
                    &torrent_from_file.info.md5sum.clone(),
                    length,
                )
                .await
                .map_err(|source| UpgradeError::Sqlx {
                    context: "failed to insert single-file torrent file",
                    source,
                })?;

            debug!(file_id, torrent_id = torrent.torrent_id, "inserted single-file torrent entry");
        } else {
            // Multiple files are being shared
            let files = torrent_from_file
                .info
                .files
                .as_ref()
                .ok_or(UpgradeError::MissingTorrentField {
                    torrent_id: torrent.torrent_id,
                    field: "files",
                })?;

            for file in files {
                debug!(torrent_id = torrent.torrent_id, ?file, "adding multi-file torrent entry");

                let file_id = target_database
                    .insert_torrent_file_for_torrent_with_multiple_files(torrent, file)
                    .await
                    .map_err(|source| UpgradeError::Sqlx {
                        context: "failed to insert multi-file torrent file",
                        source,
                    })?;

                debug!(file_id, torrent_id = torrent.torrent_id, "inserted multi-file torrent entry");
            }
        }

        // [v2] table torrust_torrent_info

        info!(torrent_id = torrent.torrent_id, "adding torrent info");

        let id = target_database
            .insert_torrent_info(torrent)
            .await
            .map_err(|source| UpgradeError::Sqlx {
                context: "failed to insert torrent info",
                source,
            })?;

        debug!(torrent_info_id = id, torrent_id = torrent.torrent_id, "inserted torrent info");

        // [v2] table torrust_torrent_announce_urls

        info!(torrent_id = torrent.torrent_id, "adding torrent announce urls");

        if let Some(announce_list) = &torrent_from_file.announce_list {
            // BEP-0012. Multiple trackers.

            for tracker_url in announce_list.iter().flatten() {
                debug!(torrent_id = torrent.torrent_id, %tracker_url, "adding announce-list URL");

                let announce_url_id = target_database
                    .insert_torrent_announce_url(torrent.torrent_id, tracker_url)
                    .await
                    .map_err(|source| UpgradeError::Sqlx {
                        context: "failed to insert announce-list URL",
                        source,
                    })?;

                debug!(announce_url_id, torrent_id = torrent.torrent_id, "inserted announce-list URL");
            }
        } else if let Some(announce) = &torrent_from_file.announce {
            debug!(torrent_id = torrent.torrent_id, tracker_url = %announce, "adding announce URL");

            let announce_url_id = target_database
                .insert_torrent_announce_url(torrent.torrent_id, announce)
                .await
                .map_err(|source| UpgradeError::Sqlx {
                    context: "failed to insert announce URL",
                    source,
                })?;

            debug!(announce_url_id, torrent_id = torrent.torrent_id, "inserted announce URL");
        }
    }

    info!("torrents transferred");

    Ok(())
}

pub fn read_torrent_from_file(path: &str) -> Result<Torrent, UpgradeError> {
    let contents = fs::read(path).map_err(|source| UpgradeError::Io {
        context: "failed to read torrent file",
        source,
    })?;

    decode_torrent(&contents).map_err(|error| UpgradeError::DecodeTorrent {
        path: path.to_string(),
        message: error.to_string(),
    })
}
