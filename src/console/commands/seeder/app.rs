//! Console app to upload random torrents to a live Index API.
//!
//! Run with:
//!
//! ```text
//! cargo run --bin seeder -- \
//!   --api-base-url <API_BASE_URL> \
//!   --number-of-torrents <NUMBER_OF_TORRENTS> \
//!   --user <USER> \
//!   --password <PASSWORD> \
//!   --interval <INTERVAL>
//! ```
//!
//! For example:
//!
//! ```text
//! cargo run --bin seeder -- \
//!   --api-base-url "http://localhost:3001" \
//!   --number-of-torrents 1000 \
//!   --user admin \
//!   --password 12345678 \
//!   --interval 0
//! ```
//!
//! That command would upload 1000 random torrents to the Index using the user
//! account admin with password 123456 and waiting 1 second between uploads.
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
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Context;
use clap::Parser;
use reqwest::Url;
use text_colorizer::Colorize;
use tracing::level_filters::LevelFilter;
use tracing::{debug, info};
use uuid::Uuid;

use super::api::Error;
use crate::console::commands::seeder::api::{login, upload_torrent};
use crate::console::commands::seeder::logging;
use crate::services::torrent_file::generate_random_torrent;
use crate::utils::parse_torrent;
use crate::web::api::client::v1::client::Client;
use crate::web::api::client::v1::contexts::torrent::forms::{BinaryFile, UploadTorrentMultipartForm};
use crate::web::api::client::v1::contexts::torrent::responses::UploadedTorrent;
use crate::web::api::client::v1::contexts::user::responses::LoggedInUserData;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
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
}

/// # Errors
///
/// Will not return any errors for the time being.
pub async fn run() -> anyhow::Result<()> {
    logging::setup(LevelFilter::INFO);

    let args = Args::parse();

    let api_url = Url::from_str(&args.api_base_url).context("failed to parse API base URL")?;

    let api_user = login_index_api(&api_url, &args.user, &args.password).await;

    let api_client = Client::authenticated(&api_url, &api_user.token);

    info!(target:"seeder", "Uploading { } random torrents to the Torrust Index with a { } seconds interval...", args.number_of_torrents.to_string().yellow(), args.interval.to_string().yellow());

    for i in 1..=args.number_of_torrents {
        info!(target:"seeder", "Uploading torrent #{} ...", i.to_string().yellow());

        match upload_random_torrent(&api_client).await {
            Ok(uploaded_torrent) => {
                debug!(target:"seeder", "Uploaded torrent {uploaded_torrent:?}");

                let json = serde_json::to_string(&uploaded_torrent).context("failed to serialize upload response into JSON")?;

                info!(target:"seeder", "Uploaded torrent: {}", json.yellow());
            }
            Err(err) => print!("Error uploading torrent {err:?}"),
        };

        if i != args.number_of_torrents {
            sleep(Duration::from_secs(args.interval));
        }
    }

    Ok(())
}

/// It logs in a user in the Index API.
pub async fn login_index_api(api_url: &Url, username: &str, password: &str) -> LoggedInUserData {
    let unauthenticated_client = Client::unauthenticated(api_url);

    info!(target:"seeder", "Trying to login with username: {} ...", username.yellow());

    let user: LoggedInUserData = login(&unauthenticated_client, username, password).await;

    if user.admin {
        info!(target:"seeder", "Logged as admin with account: {} ", username.yellow());
    } else {
        info!(target:"seeder", "Logged as {} ", username.yellow());
    }

    user
}

async fn upload_random_torrent(api_client: &Client) -> Result<UploadedTorrent, Error> {
    let uuid = Uuid::new_v4();

    info!(target:"seeder", "Uploading torrent with uuid: {} ...", uuid.to_string().yellow());

    let torrent_file = generate_random_torrent_file(uuid);

    let upload_form = UploadTorrentMultipartForm {
        title: format!("title-{uuid}"),
        description: format!("description-{uuid}"),
        category: "test".to_string(),
        torrent_file,
    };

    upload_torrent(api_client, upload_form).await
}

/// It returns the bencoded binary data of the torrent meta file.
fn generate_random_torrent_file(uuid: Uuid) -> BinaryFile {
    let torrent = generate_random_torrent(uuid);

    let bytes = parse_torrent::encode_torrent(&torrent).expect("msg:the torrent should be bencoded");

    BinaryFile::from_bytes(torrent.info.name, bytes)
}
