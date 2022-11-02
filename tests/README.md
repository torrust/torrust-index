# Running Tests

Torrust requires Docker to run different database systems for testing. [Install docker here](https://docs.docker.com/engine/).

Start the databases with `docker-compose` before running tests:

```s
docker-compose -f tests/docker-compose.yml up
```

Run all tests using:

```s
cargo test
```

Connect to the DB using MySQL client:

```s
mysql -h127.0.0.1 -uroot -ppassword torrust-index_test
```
