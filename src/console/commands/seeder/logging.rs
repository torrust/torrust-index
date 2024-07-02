//! Logging setup for the `seeder`.
use tracing::debug;
use tracing::level_filters::LevelFilter;

/// # Panics
///
///
pub fn setup(level: LevelFilter) {
    tracing_subscriber::fmt().with_max_level(level).init();

    debug!("Logging initialized");
}
