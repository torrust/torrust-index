//! Program to upload random torrents to a live Index API.
use torrust_index::console::commands::seeder::app;

fn main() -> anyhow::Result<()> {
    app::run()
}
