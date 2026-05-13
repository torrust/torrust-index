//! Program to upload random torrents to a live Index API.
//!
//! ADR-T-010 classifies this as a side-effect command: the target contract is
//! empty stdout and JSON diagnostics on stderr. The current implementation is a
//! legacy output gap until migration.
use torrust_index::console::commands::seeder::app;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    app::run().await
}
