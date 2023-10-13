//! Import Tracker Statistics command.
//!
//! It imports the number of seeders and leechers for all torrent from the linked tracker.
//!
//! You can execute it with: `cargo run --bin import_tracker_statistics`
use torrust_index::console::commands::import_tracker_statistics::run_importer;

#[tokio::main]
async fn main() {
    run_importer().await;
}
