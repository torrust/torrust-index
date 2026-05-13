//! Import Tracker Statistics command.
//!
//! It imports the number of seeders and leechers for all torrents from the linked tracker.
//!
//! You can execute it with: `cargo run --bin import_tracker_statistics`.
//!
//! ADR-T-010 classifies this as a side-effect command: the target contract is
//! empty stdout and JSON diagnostics on stderr. The current implementation is a
//! legacy output gap until migration.
use torrust_index::console::commands::tracker_statistics_importer::app::run;

#[tokio::main]
async fn main() {
    run().await;
}
