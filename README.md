# Torrust Index Backend

This repository serves as the backend for the [Torrust Index](https://github.com/torrust/torrust) project.

## Documentation
You can read the documentation [here](https://torrust.github.io/torrust-documentation/torrust-web-backend/about/).

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
./target/torrust-index-backend
```

7. Edit the newly generated `config.toml` file ([config.toml documentation](https://torrust.github.io/torrust-tracker/CONFIG.html)):
```bash
nano config.toml
```

8. Run the backend again:
```bash
./target/torrust-index-backend
```

## Contributing
Please report any Torrust Index backend specific bugs you find to the issue tracker of this repository. Torrust Index frontend specific issues can be submitted [here](https://github.com/torrust/torrust-index-frontend). Universal issues with the Torrust Index can be submitted [here](https://github.com/torrust/torrust). Ideas and feature requests are welcome as well!
