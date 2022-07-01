# Torrust Index Backend

![README HEADER](./img/Torrust_Repo_BackEnd_Readme_Header-20220615.jpg)

![Open Source](https://badgen.net/badge/Open%20Source/100%25/DA2CE7)
![Cool](https://badgen.net/badge/Cool/100%25/FF7F50)

![Nautilus Sponsored](https://badgen.net/badge/Sponsor/Nautilus%20Cyberneering/red)

---

## 📢Important Updates 📢

- None at the moment [ACCESS ALL UPDATES](https://github.com/torrust/torrust-index-backend/wiki/Project-Updates)

---

## Index

- [PROJECT DESCRIPTION](#project-description)
- [PROJECT ROADMAP](#project_roadmap)
- [DOCUMENTATION](#documentation)
- [INSTALLATION](#installation)
- [CONTACT & CONTRIBUTING](#contact_and_contributing)
- [CREDITS](#credits)

## Project Description

This repository serves as the backend for the [Torrust Index](https://github.com/torrust/torrust-index) project.

### Roadmap

*Coming soon.*

## Documentation

You can read the documentation [here](https://torrust.com/torrust-index/install/#installing-the-backend).

## Installation

1. Install prerequisites:

    - [Rust/Cargo](https://www.rust-lang.org/) - Compiler toolchain & Package Manager (cargo).

2. Clone the repository:

    ```bash
    git clone https://github.com/torrust/torrust-index-backend.git
    ```

3. Open the project directory and create a file called: `.env`:

    ```bash
    cd torrust-index-backend
    echo "DATABASE_URL=sqlite://data.db?mode=rwc" > .env
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

7. Edit the newly generated `config.toml` file ([config.toml documentation](https://torrust.github.io/torrust-tracker/CONFIG.html)):

    ```bash
    nano config.toml
    ```

8. Run the backend again:

    ```bash
    ./target/torrust-index-backend
    ```

## Contact and Contributing

Feel free to contact us via:

Message `Warm Beer#3352` on Discord or email `mick@dutchbits.nl`.

or

Please make suggestions and report any **Torrust Index Back End** specific bugs you find to the issue tracker of this repository [here](https://github.com/torrust/torrust-index-backend/issues)

**Torrust Index Front End** specific issues can be submitted [here](https://github.com/torrust/torrust-index-frontend/issues).

Universal issues with the **Torrust Index** can be submitted [here](https://github.com/torrust/torrust-index/issues). Ideas and feature requests are welcome as well!

---

## Credits & Sponsors

This project was developed by [Dutch Bits](https://dutchbits.nl) for [Nautilus Cyberneering GmbH](https://nautilus-cyberneering.de/).

The project has been possible through the support and contribution of both Nautilus Cyberneering, its team and collaborators, as well as that of our great open source contributors. Thank you to you all!
