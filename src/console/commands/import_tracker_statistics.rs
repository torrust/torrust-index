//! It imports statistics for all torrents from the linked tracker.

use std::env;
use std::sync::Arc;

use derive_more::{Display, Error};
use text_colorizer::Colorize;

use crate::bootstrap::config::init_configuration;
use crate::bootstrap::logging;
use crate::databases::database;
use crate::tracker::service::Service;
use crate::tracker::statistics_importer::StatisticsImporter;

const NUMBER_OF_ARGUMENTS: usize = 0;

#[derive(Debug)]
pub struct Arguments {}

#[derive(Debug, Display, PartialEq, Error)]
#[allow(dead_code)]
pub enum ImportError {
    #[display(fmt = "internal server error")]
    WrongNumberOfArgumentsError,
}

fn parse_args() -> Result<Arguments, ImportError> {
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

    Ok(Arguments {})
}

fn print_usage() {
    eprintln!(
        "{} - imports torrents statistics from linked tracker.

        cargo run --bin import_tracker_statistics

        ",
        "Tracker Statistics Importer".green()
    );
}

pub async fn run_importer() {
    import(&parse_args().expect("unable to parse command arguments")).await;
}

/// Import Command Arguments
///
/// # Panics
///
/// Panics if `Configuration::load_from_file` has any error.
pub async fn import(_args: &Arguments) {
    println!("Importing statistics from linked tracker ...");

    let configuration = init_configuration().await;

    logging::setup();

    let cfg = Arc::new(configuration);

    let settings = cfg.settings.read().await;

    let tracker_url = settings.tracker.url.clone();

    eprintln!("Tracker url: {}", tracker_url.green());

    let database = Arc::new(
        database::connect(&settings.database.connect_url)
            .await
            .expect("unable to connect to db"),
    );

    let tracker_service = Arc::new(Service::new(cfg.clone(), database.clone()).await);
    let tracker_statistics_importer =
        Arc::new(StatisticsImporter::new(cfg.clone(), tracker_service.clone(), database.clone()).await);

    tracker_statistics_importer
        .import_all_torrents_statistics()
        .await
        .expect("variable `tracker_service` is unable to `update_torrents`");
}
