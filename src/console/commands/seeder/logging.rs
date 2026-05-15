//! Logging setup for the `seeder`.
use tracing::level_filters::LevelFilter;

use crate::bootstrap::logging;

/// # Panics
///
///
pub fn setup(level: LevelFilter) {
    logging::setup_level_filter(level);
}
