use torrust_index_backend::databases::database::{DatabaseDriver};
use crate::databases::{run_tests};

const DATABASE_URL: &str = "mysql://root:password@localhost:3306/torrust-index_test";

#[tokio::test]
async fn run_mysql_tests() {
    run_tests(DatabaseDriver::Mysql, DATABASE_URL).await;
}


