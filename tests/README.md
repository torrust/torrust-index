### Running Tests
Torrust requires Docker to run different database systems for testing. [install docker here](https://docs.docker.com/engine/).

Start the databases with `docker-compose` before running tests:

    $ docker-compose up

Run all tests using:

    $ cargo test
