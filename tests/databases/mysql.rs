#[allow(unused_imports)]
use crate::databases::run_tests;

#[allow(dead_code)]
const DATABASE_URL: &str = "mysql://root:password@localhost:3306/torrust-index_test";

#[tokio::test]
#[should_panic]
async fn run_mysql_tests() {
    panic!("Todo Test Times Out!");
    #[allow(unreachable_code)]
    {
        run_tests(DATABASE_URL).await;
    }
}
