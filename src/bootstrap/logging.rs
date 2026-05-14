//! Setup for the application logging.
use torrust_index_cli_common::init_json_tracing;
use tracing::level_filters::LevelFilter;
use tracing::{Level, info};

use crate::config::Threshold;

pub fn setup(threshold: &Threshold) {
    let tracing_level_filter: LevelFilter = threshold.clone().into();

    setup_level_filter(tracing_level_filter);
}

pub fn setup_level_filter(filter: LevelFilter) {
    let Some(level) = level_from_filter(filter) else {
        return;
    };

    init_json_tracing(level);
    info!("Logging initialized");
}

const fn level_from_filter(filter: LevelFilter) -> Option<Level> {
    match filter {
        LevelFilter::OFF => None,
        LevelFilter::ERROR => Some(Level::ERROR),
        LevelFilter::WARN => Some(Level::WARN),
        LevelFilter::INFO => Some(Level::INFO),
        LevelFilter::DEBUG => Some(Level::DEBUG),
        LevelFilter::TRACE => Some(Level::TRACE),
    }
}
