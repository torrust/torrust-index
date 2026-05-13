//! Documentation for [Torrust Tracker Index](https://github.com/torrust/torrust-index) API.
//!
//! This is the index API for [Torrust Tracker Index](https://github.com/torrust/torrust-index).
//!
//! It is written in Rust and uses the [Axum](https://github.com/tokio-rs/axum) framework. It is designed to be
//! used with by the [Torrust Tracker Index Gui](https://github.com/torrust/torrust-index-gui).
//!
//! If you are looking for information on how to use the API, please see the
//! [API v1](crate::web::api::server::v1) section of the documentation.
//!
//! # Table of contents
//!
//! - [Features](#features)
//! - [Services](#services)
//! - [Installation](#installation)
//!     - [Minimum requirements](#minimum-requirements)
//!     - [Prerequisites](#prerequisites)
//!     - [Install from sources](#install-from-sources)
//!     - [Run with docker](#run-with-docker)
//!     - [Development](#development)
//! - [Configuration](#configuration)
//! - [Usage](#usage)
//!     - [API](#api)
//!     - [Tracker Statistics Importer](#tracker-statistics-importer)
//!     - [Upgrader](#upgrader)
//! - [Contributing](#contributing)
//! - [Documentation](#documentation)
//!
//! # Features
//!
//! - Torrent categories
//! - Image proxy cache for torrent descriptions
//! - User registration and authentication
//! - DB Support for `SQLite` and `MySQL`
//!
//! # Services
//!
//! From the end-user perspective the Torrust Tracker exposes three different services.
//!
//! - A REST [API](crate::web::api::server::v1)
//!
//! From the administrator perspective, the Torrust Index exposes:
//!
//! - A console command to update torrents statistics from the associated tracker
//! - A console command to upgrade the database schema from version `1.0.0` to `2.0.0`
//!
//! # Installation
//!
//! ## Minimum requirements
//!
//! - Rust Stable `1.88` (edition 2024)
//!
//! ## Prerequisites
//!
//! In order the run the index you will need a running torrust tracker. In the
//! configuration you need to fill the `tracker` section with the following:
//!
//! ```toml
//! [tracker]
//! url = "udp://localhost:6969"
//!
//! api_url = "http://localhost:1212/"
//! token = "MyAccessToken"
//! token_valid_seconds = 7257600
//! ```
//!
//! Refer to the [`config::tracker`](crate::config::Tracker) documentation for more information.
//!
//! You can follow the tracker installation instructions [here](https://docs.rs/torrust-tracker)
//! or you can use the docker to run both the tracker and the index. Refer to the
//! [Run with docker](#run-with-docker) section for more information.
//!
//! You will also need to install this dependency:
//!
//! ```text
//! sudo apt-get install libssl-dev
//! ```
//!
//! We needed because we are using native TLS support instead of [rustls](https://github.com/rustls/rustls).
//!
//! More info: <https://github.com/torrust/torrust-index/issues/463>.
//!
//! If you are using `SQLite3` as database driver, you will need to install the
//! following dependency:
//!
//! ```text
//! sudo apt-get install libsqlite3-dev
//! ```
//!
//! > **NOTICE**: those are the commands for `Ubuntu`. If you are using a
//! > different OS, you will need to install the equivalent packages. Please
//! > refer to the documentation of your OS.
//!
//! With the default configuration you will need to create the `storage` directory:
//!
//! ```text
//! storage/
//! └── database
//!     └── data.db
//! ```
//!
//! The default configuration expects a directory `./storage/database` to be writable by the app process.
//!
//! By default the index uses `SQLite` and the database file name `data.db`.
//!
//! ## Install from sources
//!
//! ```text
//! git clone git@github.com:torrust/torrust-index.git \
//!   && cd torrust-index \
//!   && cargo build --release \
//!   && mkdir -p ./storage/database
//! ```
//!
//! Then you can run it with: `./target/release/torrust-index`
//!
//! ## Run with docker
//!
//! You can run the index with a pre-built docker image. Per ADR-T-009 §D2
//! the shipped TOMLs no longer carry default values for `tracker.token` or
//! `database.connect_url`, so both must be supplied via env-var overrides
//! (or a populated `index.toml`) at startup — a bare `docker run` without
//! them now fails with a serde `missing field` error rather than booting
//! against hidden defaults:
//!
//! ```text
//! cd /tmp \
//!   && mkdir torrust-index \
//!   && cd torrust-index \
//!   && mkdir -p ./storage/index/lib/database \
//!   && mkdir -p ./storage/index/log \
//!   && mkdir -p ./storage/index/etc \
//!   && export USER_ID=1000 \
//!   && docker run -it \
//!     --env USER_ID="$USER_ID" \
//!     --env TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="MyAccessToken" \
//!     --env TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL="sqlite:///var/lib/torrust/index/database/index.sqlite3.db?mode=rwc" \
//!     --publish 3001:3001/tcp \
//!     --volume "$(pwd)/storage/index/lib":"/var/lib/torrust/index" \
//!     --volume "$(pwd)/storage/index/log":"/var/log/torrust/index" \
//!     --volume "$(pwd)/storage/index/etc":"/etc/torrust/index" \
//!     torrust/index:develop
//! ```
//!
//! For more information see the [container guide](https://github.com/torrust/torrust-index/blob/develop/docs/containers.md)
//! and [ADR-T-009](https://github.com/torrust/torrust-index/blob/develop/adr/009-container-infrastructure-refactor.md).
//! For local-stack development the repository ships a Compose split
//! (`compose.yaml` baseline + auto-loaded `compose.override.yaml` dev
//! sandbox) wrapped by `make up-dev` / `make up-prod`.
//!
//! ## Development
//!
//! We are using the [The Rust SQL Toolkit](https://github.com/launchbadge/sqlx)
//! [(sqlx)](https://github.com/launchbadge/sqlx) for database migrations.
//!
//! You can install it with:
//!
//! ```text
//! cargo install sqlx-cli
//! ```
//!
//! To initialize the database run:
//!
//! ```text
//! echo "DATABASE_URL=sqlite://data.db?mode=rwc" > .env
//! sqlx db setup
//! ```
//!
//! The `sqlx db setup` command will create the database specified in your
//! `DATABASE_URL` and run any pending migrations.
//!
//! > **WARNING**: The `.env` file is also used by docker-compose.
//!
//! > **NOTICE**: Refer to the [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli)
//! > documentation for other commands to create new migrations or run them.
//!
//! > **NOTICE**: You can run the index with [tmux](https://github.com/tmux/tmux/wiki) with `tmux new -s torrust-index`.
//!
//! # Configuration
//!
//! The index uses a TOML configuration file. Per ADR-T-009 §D2 two fields
//! are mandatory at the schema level — `tracker.token` and
//! `database.connect_url`. They must be supplied either inline in the TOML
//! or via the corresponding `TORRUST_INDEX_CONFIG_OVERRIDE_*` env vars; a
//! missing value fails at deserialisation with a precise serde
//! `missing field` error.
//!
//! See [`share/default/config/`](https://github.com/torrust/torrust-index/tree/develop/share/default/config)
//! for example configuration files, or the [`config`] module documentation
//! for the full schema reference. The parsing surface lives in the
//! standalone `torrust-index-config` crate
//! ([`packages/index-config/`](https://github.com/torrust/torrust-index/tree/develop/packages/index-config))
//! so that helper binaries (notably `torrust-index-config-probe`) can load
//! the same `Settings` the application loads, without pulling in the
//! runtime stack.
//!
//! Key sections:
//!
//! - `[tracker]` — Tracker connection (API URL, **mandatory** `token`).
//! - `[auth]` — Password constraints. Optionally supply RSA key paths
//!   (`auth.private_key_path` / `auth.public_key_path`) for persistent
//!   JWT sessions; otherwise an ephemeral key pair is auto-generated.
//! - `[database]` — `SQLite` or `MySQL` **mandatory** `connect_url`.
//! - `[net]` — Bind address (default: `0.0.0.0:3001`). Optional
//!   `[net.tls]` block for TLS termination.
//! - `[[permissions.overrides]]` — Optional TOML-based permission
//!   overrides (see ADR-T-008).
//!
//! For more information about configuration you can visit the documentation for the [`config`] module.
//!
//! Alternatively to the config file you can use one environment variable `TORRUST_INDEX_CONFIG_TOML` to pass the configuration to the index:
//!
//! ```text
//! TORRUST_INDEX_CONFIG_TOML=$(cat config.toml)
//! cargo run
//! ```
//!
//! In the previous example you are just setting the env var with the contents of the `config.toml` file.
//!
//! The env var contains the same data as the `config.toml`. It's particularly useful in you are [running the index with docker](https://github.com/torrust/torrust-index/tree/develop/docker).
//!
//! > **NOTICE**: The `TORRUST_INDEX_CONFIG_TOML` env var has priority over the `config.toml` file.
//!
//! > **NOTICE**: You can also change the location for the configuration file with the `TORRUST_INDEX_CONFIG_TOML_PATH` env var.
//!
//! # Usage
//!
//! ## API
//!
//! Running the index with the default configuration will expose the REST API on port 3001: <http://localhost:3001>
//!
//! ## Tracker Statistics Importer
//!
//! This console command allows you to manually import the tracker statistics.
//! ADR-T-010 classifies it as a side-effect command: the target contract is
//! empty stdout and JSON diagnostics on stderr. The current implementation is a
//! legacy output gap until the command is migrated, so scripts should not parse
//! its plain-text diagnostics.
//!
//! For more information about this command you can visit the documentation for
//! the [`Import tracker statistics`](crate::console::commands::tracker_statistics_importer) module.
//!
//! ## Upgrader
//!
//! This console command allows you to manually upgrade the application from one
//! version to another.
//! ADR-T-010 classifies it as a side-effect command: the target contract is
//! empty stdout and JSON diagnostics on stderr. The current implementation is a
//! legacy output gap until the command is migrated, so scripts should branch on
//! exit status rather than parsing current plain text.
//!
//! For more information about this command you can visit the documentation for
//! the [`Upgrade app from version 1.0.0 to 2.0.0`](crate::upgrades::from_v1_0_0_to_v2_0_0::upgrader) module.
//!
//! # Contributing
//!
//! If you want to contribute to this documentation you can:
//!
//! - [Open a new discussion](https://github.com/torrust/torrust-index/discussions)
//! - [Open a new issue](https://github.com/torrust/torrust-index/issues).
//! - [Open a new pull request](https://github.com/torrust/torrust-index/pulls).
//!
//! # Documentation
//!
//! You can find this documentation on [docs.rs](https://docs.rs/torrust-index/).
//!
//! If you want to contribute to this documentation you can [open a new pull request](https://github.com/torrust/torrust-index/pulls).
//!
//! In addition to the production code documentation you can find a lot of
//! examples in the [tests](https://github.com/torrust/torrust-index/tree/develop/tests/e2e/contexts) directory.
pub mod app;
pub mod bootstrap;
pub mod cache;
pub mod common;
pub mod config;
pub mod console;
pub mod databases;
pub mod errors;
pub mod jwt;
pub mod mailer;
pub mod models;
pub mod services;
pub mod tracker;
pub mod ui;
pub mod upgrades;
pub mod utils;
pub mod web;

#[cfg(test)]
mod tests;

trait AsCSV {
    fn as_csv<T>(&self) -> Result<Option<Vec<T>>, ()>
    where
        T: std::str::FromStr;
}

impl<S> AsCSV for Option<S>
where
    S: AsRef<str>,
{
    fn as_csv<T>(&self) -> Result<Option<Vec<T>>, ()>
    where
        T: std::str::FromStr,
    {
        match self {
            Some(s) if !s.as_ref().trim().is_empty() => {
                let mut acc = vec![];
                for s in s.as_ref().split(',') {
                    let item = s.trim().parse::<T>().map_err(|_| ())?;
                    acc.push(item);
                }
                if acc.is_empty() { Ok(None) } else { Ok(Some(acc)) }
            }
            _ => Ok(None),
        }
    }
}
