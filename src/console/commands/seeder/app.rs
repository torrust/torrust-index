//! Program to upload random torrent to a live Index API.
//!
//! Run with:
//!
//! ```text
//! cargo run --bin seeder -- --api-base-url <API_BASE_URL> --number-of-torrents <NUMBER_OF_TORRENTS> --user <USER> --password <PASSWORD> --interval <INTERVAL>
//! ```
//!
//! For example:
//!
//! ```text
//! cargo run --bin seeder -- --api-base-url "localhost:3001" --number-of-torrents 1000 --user admin --password 12345678 --interval 0
//! ```
//!
//! That command would upload 1000 random torrents to the Index using the user
//! account admin with password 123456 and waiting 1 second between uploads.
use std::thread::sleep;
use std::time::Duration;

use anyhow::Context;
use clap::Parser;
use log::{debug, info, LevelFilter};
use text_colorizer::Colorize;
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
    logging::setup(LevelFilter::Info);

    let args = Args::parse();

    let api_user = login_index_api(&args.api_base_url, &args.user, &args.password).await;

    let api_client = Client::authenticated(&args.api_base_url, &api_user.token);

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
pub async fn login_index_api(api_url: &str, username: &str, password: &str) -> LoggedInUserData {
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
