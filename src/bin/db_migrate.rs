//! Migration command to migrate data from v1.0.0 to v2.0.0
//! Run it with `cargo run --bin db_migrate`

use std::sync::Arc;
use torrust_index_backend::config::Configuration;
use torrust_index_backend::databases::database::{
    connect_database, connect_database_without_running_migrations,
};

#[actix_web::main]
async fn main() {
    let dest_database_connect_url = "sqlite://data_v2.db?mode=rwc";

    let cfg = match Configuration::load_from_file().await {
        Ok(config) => Arc::new(config),
        Err(error) => {
            panic!("{}", error)
        }
    };

    let settings = cfg.settings.read().await;

    // Connect to the current v1.0.0 DB
    let source_database = Arc::new(
        connect_database_without_running_migrations(&settings.database.connect_url)
            .await
            .expect("Can't connect to source DB."),
    );

    // Connect to the new v2.0.0 DB (running migrations)
    let dest_database = Arc::new(
        connect_database(&dest_database_connect_url)
            .await
            .expect("Can't connect to dest DB."),
    );

    println!("Upgrading database from v1.0.0 to v2.0.0 ...");

    // It's just a test for the source connection.
    // Print categories in current DB
    let categories = source_database.get_categories().await;
    println!("[v1] categories: {:?}", &categories);

    // It's just a test for the dest connection.
    // Print categories in new DB
    let categories = dest_database.get_categories().await;
    println!("[v2] categories: {:?}", &categories);

    // Transfer categories

    /* TODO:
    - Transfer categories: remove categories from seeding, reset sequence for IDs, copy categories in the right order to keep the same ids.
    - ...
    */
}
