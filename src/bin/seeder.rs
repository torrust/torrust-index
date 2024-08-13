//! Program to upload random torrents to a live Index API.
use torrust_index::console::commands::seeder::app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    app::run().await
}
