//! Migration command to migrate data from v1.0.0 to v2.0.0
//! Run it with `cargo run --bin db_migrate`

use std::sync::Arc;
use torrust_index_backend::config::Configuration;
use torrust_index_backend::databases::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use torrust_index_backend::databases::sqlite_v2_0_0::SqliteDatabaseV2_0_0;

async fn current_db() -> Arc<SqliteDatabaseV1_0_0> {
    // Connect to the old v1.0.0 DB
    let cfg = match Configuration::load_from_file().await {
        Ok(config) => Arc::new(config),
        Err(error) => {
            panic!("{}", error)
        }
    };

    let settings = cfg.settings.read().await;

    Arc::new(SqliteDatabaseV1_0_0::new(&settings.database.connect_url).await)
}

async fn new_db(db_filename: String) -> Arc<SqliteDatabaseV2_0_0> {
    let dest_database_connect_url = format!("sqlite://{}?mode=rwc", db_filename);
    Arc::new(SqliteDatabaseV2_0_0::new(&dest_database_connect_url).await)
}

async fn reset_destiny_database(dest_database: Arc<SqliteDatabaseV2_0_0>) {
    println!("Truncating all tables in destiny database ...");
    dest_database
        .delete_all_database_rows()
        .await
        .expect("Can't reset destiny database.");
}

async fn transfer_categories(
    source_database: Arc<SqliteDatabaseV1_0_0>,
    dest_database: Arc<SqliteDatabaseV2_0_0>,
) {
    let source_categories = source_database.get_categories_order_by_id().await.unwrap();
    println!("[v1] categories: {:?}", &source_categories);

    let result = dest_database.reset_categories_sequence().await.unwrap();
    println!("result {:?}", result);

    for cat in &source_categories {
        println!(
            "[v2] adding category: {:?} {:?} ...",
            &cat.category_id, &cat.name
        );
        let id = dest_database
            .insert_category_and_get_id(&cat.name)
            .await
            .unwrap();

        if id != cat.category_id {
            panic!(
                "Error copying category {:?} from source DB to destiny DB",
                &cat.category_id
            );
        }

        println!("[v2] category: {:?} {:?} added.", id, &cat.name);
    }

    let dest_categories = dest_database.get_categories().await.unwrap();
    println!("[v2] categories: {:?}", &dest_categories);
}

#[actix_web::main]
async fn main() {
    // Get connections to source adn destiny databases
    let source_database = current_db().await;
    let dest_database = new_db("data_v2.db".to_string()).await;

    println!("Upgrading data from version v1.0.0 to v2.0.0 ...");

    reset_destiny_database(dest_database.clone()).await;
    transfer_categories(source_database.clone(), dest_database.clone()).await;

    // TODO: WIP. We have to transfer data from the 5 tables in V1 and the torrent files in folder `uploads`.
}
