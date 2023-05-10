use std::future::Future;

use torrust_index_backend::databases::database;
use torrust_index_backend::databases::database::Database;

mod mysql;
mod sqlite;
mod tests;

// used to run tests with a clean database
async fn run_test<'a, T, F, DB: Database + ?Sized>(db_fn: T, db: &'a DB)
where
    T: FnOnce(&'a DB) -> F + 'a,
    F: Future<Output = ()>,
{
    // cleanup database before testing
    assert!(db.delete_all_database_rows().await.is_ok());

    // run test using clean database
    db_fn(db).await;
}

// runs all tests
pub async fn run_tests(db_path: &str) {
    let db_res = database::connect(db_path).await;

    assert!(db_res.is_ok());

    let db_boxed = db_res.unwrap();

    let db: &dyn Database = db_boxed.as_ref();

    run_test(tests::it_can_add_a_user, db).await;
    run_test(tests::it_can_add_a_torrent_category, db).await;
    run_test(tests::it_can_add_a_torrent_and_tracker_stats_to_that_torrent, db).await;
}
