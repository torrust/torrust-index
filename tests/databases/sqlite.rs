use torrust_index_backend::databases::database::{DatabaseDriver};
use crate::databases::{run_tests};

const DATABASE_URL: &str = "sqlite::memory:";

#[tokio::test]
async fn run_sqlite_tests() {
    run_tests(DatabaseDriver::Sqlite3, DATABASE_URL).await;
}


