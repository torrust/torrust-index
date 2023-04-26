//! End-to-end tests.
//!
//! Execute E2E tests with:
//!
//! ```
//! cargo test --features e2e-tests
//! cargo test --features e2e-tests -- --nocapture
//! ```
//!
//! or the Cargo alias:
//!
//! ```
//! cargo e2e
//! ```
//!
//! > **NOTICE**: E2E tests are not executed by default, because they require
//! a running instance of the API.
//!
//! You can also run only one test with:
//!
//! ```
//! cargo test --features e2e-tests TEST_NAME -- --nocapture
//! cargo test --features e2e-tests it_should_register_a_new_user -- --nocapture
//! ```
//!
//! > **NOTICE**: E2E tests always use the same databases
//! `storage/database/torrust_index_backend_e2e_testing.db` and
//! `./storage/database/torrust_tracker_e2e_testing.db`. If you want to use a
//! clean database, delete the files before running the tests.
//!
//! See the [docker documentation](https://github.com/torrust/torrust-index-backend/tree/develop/docker) for more information on how to run the API.
mod asserts;
mod client;
mod connection_info;
mod contexts;
mod environment;
mod http;
mod response;
