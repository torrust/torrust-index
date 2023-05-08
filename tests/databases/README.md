# Persistence Tests

Torrust requires Docker to run different database systems for testing.

Start the databases with `docker-compose` before running tests:

```s
docker-compose -f tests/databases/docker-compose.yml up
```

Run all tests using:

```s
cargo test
```

Connect to the DB using MySQL client:

```s
mysql -h127.0.0.1 -uroot -ppassword torrust-index_test
```

Right now only tests for MySQLite are executed. To run tests for MySQL too,
you have to replace this line in `tests/databases/mysql.rs`:

```rust

```rust
#[tokio::test]
#[should_panic]
async fn run_mysql_tests() {
    panic!("Todo Test Times Out!");
    #[allow(unreachable_code)]
    {
        run_tests(DATABASE_URL).await;
    }
}
```

with this:

```rust
#[tokio::test]
async fn run_mysql_tests() {
    run_tests(DATABASE_URL).await;
}
```
