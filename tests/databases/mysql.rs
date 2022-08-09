use torrust_index_backend::databases::database::{Database, DatabaseDriver};
use crate::databases::database;
use crate::databases::database::run_tests;

const DATABASE_URL: &str = "mysql://root:password@localhost:3306/torrust-index";

async fn setup() -> Result<Box<dyn Database>, ()> {
    database::setup(DatabaseDriver::Mysql, DATABASE_URL).await
}

#[tokio::test]
async fn run_mysql_tests() {
    let setup = setup().await;

    assert!(setup.is_ok());

    let db = setup.unwrap();

    // cleanup database
    assert!(db.delete_all_database_rows().await.is_ok());

    run_tests(&db).await;
}


