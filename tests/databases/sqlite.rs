use crate::databases::run_tests;

const DATABASE_URL: &str = "sqlite::memory:";

#[tokio::test]
async fn run_sqlite_tests() {
    run_tests(DATABASE_URL).await;
}
