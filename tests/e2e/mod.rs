//! End-to-end tests.
//!
//! Execute E2E tests with:
//!
//! ```
//! cargo test --features e2e-tests
//! ```
//!
//! or the Cargo alias
//!
//! ```
//! cargo e2e
//! ```
//!
//! > **NOTICE**: E2E tests are not executed by default, because they require
//! a running instance of the API.
//!
//! See the docker documentation for more information on how to run the API.
mod asserts;
mod client;
mod connection_info;
pub mod env;
mod http;
mod response;
mod routes;
