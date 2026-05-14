//! Console app to upload random torrents to a live Index API.
//!
//! ADR-T-010 classifies this as a side-effect command: stdout remains empty and
//! diagnostics are JSON records on stderr.
//!
//! Run with:
//!
//! ```text
//! cargo run --quiet --bin seeder -- \
//!   --api-base-url <API_BASE_URL> \
//!   --number-of-torrents <NUMBER_OF_TORRENTS> \
//!   --user <USER> \
//!   --password <PASSWORD> \
//!   --interval <INTERVAL> \
//!   2>seeder.ndjson
//! jq . seeder.ndjson
//! ```
//!
//! For example:
//!
//! ```text
//! cargo run --quiet --bin seeder -- \
//!   --api-base-url "http://localhost:3001" \
//!   --number-of-torrents 1000 \
//!   --user admin \
//!   --password 12345678 \
//!   --interval 0 \
//!   2>seeder.ndjson
//! jq . seeder.ndjson
//! ```
//!
//! That command would upload 1000 random torrents to the Index using the user
//! account `admin` with password `12345678` and no delay between uploads.
//!
//! The random torrents generated are single-file torrents from a TXT file.
//! All generated torrents used a UUID to identify the test torrent. The torrent
//! is generated on the fly without needing to generate the contents file.
//! However, if you like it, you can generate the contents and the torrent
//! manually with the following commands:
//!
//! ```text
//! cd /tmp
//! mkdir test_torrents
//! cd test_torrents
//! uuidgen
//! echo $'1fd827fb-29dc-47bd-b116-bf96f6466e65' > file-1fd827fb-29dc-47bd-b116-bf96f6466e65.txt
//! imdl torrent create file-1fd827fb-29dc-47bd-b116-bf96f6466e65.txt
//! imdl torrent show file-1fd827fb-29dc-47bd-b116-bf96f6466e65.txt.torrent
//! ```
//!
//! That could be useful for testing purposes. For example, if you want to seed
//! the torrent with a `BitTorrent` client.
//!
//! Let's explain each line:
//!
//! First, we need to generate the UUID:
//!
//! ```text
//! uuidgen
//! 1fd827fb-29dc-47bd-b116-bf96f6466e65
//! ````
//!
//! Then, we need to create a text file and write the UUID into the file:
//!
//! ```text
//! echo $'1fd827fb-29dc-47bd-b116-bf96f6466e65' > file-1fd827fb-29dc-47bd-b116-bf96f6466e65.txt
//! ```
//!
//! Finally you can use a torrent creator like [Intermodal](https://github.com/casey/intermodal)
//!  to generate the torrent file. You can use any `BitTorrent` client or other
//! console tool.
//!
//! ```text
//! imdl torrent create file-1fd827fb-29dc-47bd-b116-bf96f6466e65.txt
//! $ imdl torrent create file-1fd827fb-29dc-47bd-b116-bf96f6466e65.txt
//! [1/3] 🧿 Searching `file-1fd827fb-29dc-47bd-b116-bf96f6466e65.txt` for files…
//! [2/3] 🧮 Hashing pieces…
//! [3/3] 💾 Writing metainfo to `file-1fd827fb-29dc-47bd-b116-bf96f6466e65.txt.torrent`…
//! ✨✨ Done! ✨✨
//! ````
//!
//! The torrent meta file contains this information:
//!
//! ```text
//! $ imdl torrent show file-1fd827fb-29dc-47bd-b116-bf96f6466e65.txt.torrent
//!          Name  file-1fd827fb-29dc-47bd-b116-bf96f6466e65.txt
//! Creation Date  2024-02-07 12:47:32 UTC
//!    Created By  imdl/0.1.13
//!     Info Hash  c8cf845e9771013b5c0e022cb1fc1feebdb24b66
//!  Torrent Size  201 bytes
//!  Content Size  37 bytes
//!       Private  no
//!    Piece Size  16 KiB
//!   Piece Count  1
//!    File Count  1
//!         Files  file-1fd827fb-29dc-47bd-b116-bf96f6466e65.txt
//!````
//!
//! The torrent generated manually contains this info:
//!
//! ```json
//! {
//!   "created by": "imdl/0.1.13",
//!   "creation date": 1707304810,
//!   "encoding": "UTF-8",
//!     "info": {
//!       "length": 37,
//!       "name": "file-1fd827fb-29dc-47bd-b116-bf96f6466e65.txt",
//!       "piece length": 16384,
//!       "pieces": "<hex>E2 11 4F 69 79 50 1E CC F6 32 91 A5 12 FA D5 6B 49 20 12 D3</hex>"
//!     }
//!  }
//! ```
//!
//! If you upload that torrent to the Index and you download it, then you
//! get this torrent information:
//!
//! ```json
//! {
//!   "announce": "udp://tracker.torrust-demo.com:6969/k24qT2KgWFh9d5e1iHSJ9kOwfK45fH4V",
//!   "announce-list": [
//!     [
//!       "udp://tracker.torrust-demo.com:6969/k24qT2KgWFh9d5e1iHSJ9kOwfK45fH4V"
//!     ]
//!   ],
//!   "info": {
//!     "length": 37,
//!     "name": "file-1fd827fb-29dc-47bd-b116-bf96f6466e65.txt",
//!     "piece length": 16384,
//!     "pieces": "<hex>E2 11 4F 69 79 50 1E CC F6 32 91 A5 12 FA D5 6B 49 20 12 D3</hex>"
//!   }
//! }
//! ```
//!
//! As you can see the `info` dictionary is exactly the same, which produces
//! the same info-hash for the torrent.
use std::time::Duration;

use clap::Parser;
use reqwest::Url;
use thiserror::Error;
use torrust_index_cli_common::BaseArgs;
use tracing::{debug, error, info};
use uuid::Uuid;

use super::api::Error as ApiError;
use crate::console::commands::seeder::api::{login, upload_torrent};
use crate::services::torrent_file::generate_random_torrent;
use crate::utils::parse_torrent;
use crate::web::api::client::v1::client::Client;
use crate::web::api::client::v1::contexts::torrent::forms::{BinaryFile, UploadTorrentMultipartForm};
use crate::web::api::client::v1::contexts::torrent::responses::UploadedTorrent;
use crate::web::api::client::v1::contexts::user::responses::LoggedInUserData;

#[derive(Parser)]
#[command(name = "seeder", version, about = "Upload random torrents to a live Index API", long_about = None)]
pub struct Args {
    #[arg(short, long)]
    api_base_url: String,

    #[arg(short, long)]
    number_of_torrents: i32,

    #[arg(short, long)]
    user: String,

    #[arg(short, long)]
    password: String,

    #[arg(short, long)]
    interval: u64,

    #[command(flatten)]
    pub base: BaseArgs,
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("failed to parse API base URL: {source}")]
    ParseApiBaseUrl { source: url::ParseError },

    #[error("failed to login to the Index API: {source}")]
    Login { source: ApiError },

    #[error("failed to upload torrent to the Index API: {source}")]
    UploadTorrent { source: ApiError },

    #[error("failed to encode generated torrent: {source}")]
    EncodeTorrent { source: serde_bencode::Error },

    #[error("failed to serialize upload response into JSON: {source}")]
    SerializeUploadResponse { source: serde_json::Error },
}

/// # Errors
///
/// Returns an error if setup fails. Individual torrent upload failures are
/// logged and the command continues with the next generated torrent.
pub async fn run(args: Args) -> Result<(), CommandError> {
    let api_url = args
        .api_base_url
        .parse::<Url>()
        .map_err(|source| CommandError::ParseApiBaseUrl { source })?;

    let api_user = login_index_api(&api_url, &args.user, &args.password).await?;

    let api_client = Client::authenticated(&api_url, &api_user.token);

    info!(target:"seeder", number_of_torrents = args.number_of_torrents, interval_seconds = args.interval, "uploading random torrents to the Index API");

    for i in 1..=args.number_of_torrents {
        info!(target:"seeder", torrent_number = i, "uploading torrent");

        match upload_random_torrent(&api_client).await {
            Ok(uploaded_torrent) => {
                debug!(target:"seeder", ?uploaded_torrent, "uploaded torrent");

                let json = serde_json::to_string(&uploaded_torrent)
                    .map_err(|source| CommandError::SerializeUploadResponse { source })?;

                info!(target:"seeder", uploaded_torrent = %json, "uploaded torrent");
            }
            Err(error) => error!(target:"seeder", torrent_number = i, %error, "failed to upload torrent"),
        }

        if i != args.number_of_torrents {
            tokio::time::sleep(Duration::from_secs(args.interval)).await;
        }
    }

    Ok(())
}

/// It logs in a user in the Index API.
///
/// # Errors
///
/// Returns an error if the login request fails or the response cannot be
/// decoded.
pub async fn login_index_api(api_url: &Url, username: &str, password: &str) -> Result<LoggedInUserData, CommandError> {
    let unauthenticated_client = Client::unauthenticated(api_url);

    info!(target:"seeder", username, "trying to login");

    let user: LoggedInUserData = login(&unauthenticated_client, username, password)
        .await
        .map_err(|source| CommandError::Login { source })?;

    if user.role == "admin" {
        info!(target:"seeder", username, role = %user.role, "logged in as admin");
    } else {
        info!(target:"seeder", username, role = %user.role, "logged in");
    }

    Ok(user)
}

async fn upload_random_torrent(api_client: &Client) -> Result<UploadedTorrent, CommandError> {
    let uuid = Uuid::new_v4();

    info!(target:"seeder", %uuid, "uploading torrent with uuid");

    let torrent_file = generate_random_torrent_file(uuid)?;

    let upload_form = UploadTorrentMultipartForm {
        title: format!("title-{uuid}"),
        description: format!("description-{uuid}"),
        category: "test".to_string(),
        torrent_file,
    };

    upload_torrent(api_client, upload_form)
        .await
        .map_err(|source| CommandError::UploadTorrent { source })
}

/// It returns the bencoded binary data of the torrent meta file.
fn generate_random_torrent_file(uuid: Uuid) -> Result<BinaryFile, CommandError> {
    let torrent = generate_random_torrent(uuid);

    let bytes = parse_torrent::encode_torrent(&torrent).map_err(|source| CommandError::EncodeTorrent { source })?;

    Ok(BinaryFile::from_bytes(torrent.info.name, bytes))
}
