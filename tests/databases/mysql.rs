use crate::databases::{run_tests};

const DATABASE_URL: &str = "mysql://root:password@localhost:3306/torrust-index_test";

#[tokio::test]
async fn run_mysql_tests() {
    run_tests(DATABASE_URL).await;
}


