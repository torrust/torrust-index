# Torrust Index Backend

This repository serves as the backend for the [Torrust Index](https://github.com/torrust/torrust-index) project, that implements the [Torrust Index Application Interface](https://github.com/torrust/torrust-index-api-lib).

We also provide the [Torrust Index Frontend](https://github.com/torrust/torrust-index-frontend) project, that is our reference web-application that consumes the API provided here.

## Documentation

You can read the Torrust Index documentation [here](https://torrust.com/torrust-index/install/#installing-the-backend).

## Installation

1. Setup [Rust / Cargo](https://www.rust-lang.org/) in your Environment.

2. Clone this repo.

3. Set the database connection URI in the projects `/.env` file:

    ```bash
    cd torrust-index-backend
    echo "DATABASE_URL=sqlite://data.db?mode=rwc" >> .env
    ```

4. Install sqlx-cli and build the sqlite database:

    ```bash
    cargo install sqlx-cli
    sqlx db setup
    ```

5. Build the binaries:

    ```bash
    cargo build --release
    ```

6. Run the backend once to generate the `config.toml` file:

    ```bash
    ./target/release/torrust-index-backend
    ```

7. Review and edit the default `/config.toml` file.

> _Please view the [configuration documentation](https://torrust.github.io/torrust-tracker/CONFIG.html)._

8. Run the backend again:

    ```bash
    ./target/torrust-index-backend
    ```

## Contact and Contributing

Please consider the [Torrust Contribution Guide](https://github.com/torrust/.github/blob/main/info/contributing.md).

Please report issues:

* Torrust Index Backend specifically: [here](https://github.com/torrust/torrust-index-backend/issues).
* Torrust Index in general: [here](https://github.com/torrust/torrust-index/issues).

---

## Credits & Sponsors

This project was developed by [Dutch Bits](https://dutchbits.nl) for [Nautilus Cyberneering GmbH](https://nautilus-cyberneering.de/).

The project has been possible through the support and contribution of both Nautilus Cyberneering, its team and collaborators, as well as that of our great open source contributors. Thank you to you all!
