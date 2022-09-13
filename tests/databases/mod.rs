use std::future::Future;
use torrust_index_backend::databases::database::{connect_database, Database};

mod mysql;
mod tests;
mod sqlite;

// used to run tests with a clean database
async fn run_test<'a, T, F>(db_fn: T, db: &'a Box<dyn Database>)
    where
        T: FnOnce(&'a Box<dyn Database>) -> F + 'a,
        F: Future<Output = ()>
{
    // cleanup database before testing
    assert!(db.delete_all_database_rows().await.is_ok());

    // run test using clean database
    db_fn(db).await;
}

// runs all tests
pub async fn run_tests(db_path: &str) {
    let db_res = connect_database(db_path).await;

    assert!(db_res.is_ok());

    let db = db_res.unwrap();

    run_test(tests::it_can_add_a_user, &db).await;
    run_test(tests::it_can_add_a_torrent_category, &db).await;
    run_test(tests::it_can_add_a_torrent_and_tracker_stats_to_that_torrent, &db).await;
}

