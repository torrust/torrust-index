//! It imports statistics for all torrents from the linked tracker.
//!
//! It imports the number of seeders and leechers for all torrents from the
//! associated tracker.
//!
//! You can execute it with: `cargo run --bin import_tracker_statistics`.
//!
//! After running it you will see the following output:
//!
//! ```text
//! Importing statistics from linked tracker ...
//! Loading configuration from config file `./config.toml`
//! Tracker url: udp://localhost:6969
//! ```
//!
//! Statistics are also imported:
//!
//! - Periodically by the importer job. The importer job is executed every hour
//!   by default. See [`TrackerStatisticsImporter`](crate::config::TrackerStatisticsImporter)
//!   for more details.
//! - When a new torrent is added.
//! - When the API returns data about a torrent statistics are collected from
//!   the tracker in real time.
use std::env;
use std::sync::Arc;

use derive_more::{Display, Error};
use text_colorizer::Colorize;

use crate::bootstrap::config::initialize_configuration;
use crate::bootstrap::logging;
use crate::databases::database;
use crate::tracker::service::Service;
use crate::tracker::statistics_importer::StatisticsImporter;

const NUMBER_OF_ARGUMENTS: usize = 0;

#[derive(Debug, Display, PartialEq, Error)]
#[allow(dead_code)]
pub enum ImportError {
    #[display(fmt = "internal server error")]
    WrongNumberOfArgumentsError,
}

fn parse_args() -> Result<(), ImportError> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() != NUMBER_OF_ARGUMENTS {
        eprintln!(
            "{} wrong number of arguments: expected {}, got {}",
            "Error".red().bold(),
            NUMBER_OF_ARGUMENTS,
            args.len()
        );
        print_usage();
        return Err(ImportError::WrongNumberOfArgumentsError);
    }

    Ok(())
}

fn print_usage() {
    eprintln!(
        "{} - imports torrents statistics from linked tracker.

        cargo run --bin import_tracker_statistics

        ",
        "Tracker Statistics Importer".green()
    );
}

/// Import Tracker Statistics Command
///
/// # Panics
///
/// Panics if arguments cannot be parsed.
pub async fn run() {
    parse_args().expect("unable to parse command arguments");
    import().await;
}

/// Import Command Arguments
///
/// # Panics
///
/// Panics if it can't connect to the database.
pub async fn import() {
    println!("Importing statistics from linked tracker ...");

    let configuration = initialize_configuration();

    let threshold = configuration.settings.read().await.logging.threshold.clone();

    logging::setup(&threshold);

    let cfg = Arc::new(configuration);

    let settings = cfg.settings.read().await;

    let tracker_url = settings.tracker.url.clone();

    eprintln!("Tracker url: {}", tracker_url.to_string().green());

    let database = Arc::new(
        database::connect(settings.database.connect_url.as_ref())
            .await
            .expect("unable to connect to db"),
    );

    let tracker_service = Arc::new(Service::new(cfg.clone(), database.clone()).await);
    let tracker_statistics_importer =
        Arc::new(StatisticsImporter::new(cfg.clone(), tracker_service.clone(), database.clone()).await);

    tracker_statistics_importer
        .import_all_torrents_statistics()
        .await
        .expect("should import all torrents statistics");
}
