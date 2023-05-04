//! End-to-end tests
//!
//! These test can be executed against an out-of-process server (shared) or
//! against an in-process server (isolated).
//!
//! If you want to run the tests against an out-of-process server, you need to
//! set the environment variable `TORRUST_IDX_BACK_E2E_SHARED` to `true`.
//!
//! > **NOTICE**: The server must be running before running the tests. The
//! server url is hardcoded to `http://localhost:3000` for now. We are planning
//! to make it configurable in the future via a environment variable.
//!
//! ```text
//! TORRUST_IDX_BACK_E2E_SHARED=true cargo test
//! ```
//!
//! If you want to run the tests against an isolated server, you need to execute
//! the following command:
//!
//! ```text
//! cargo test
//! ```
//!
//! > **NOTICE**: Some tests require the real tracker to be running, so they
//! can only be run in shared mode until we implement a mock for the
//! `torrust_index_backend::tracker::TrackerService`.
//!
//! You may have errors like `Too many open files (os error 24)`. If so, you
//! need to increase the limit of open files for the current user. You can do
//! it by executing the following command (on Ubuntu):
//!
//! ```text
//! ulimit -n 4096
//! ```
//!
//! You can also make that change permanent, please refer to your OS
//! documentation. See <https://superuser.com/a/1200818/277693> for more
//! information.
pub mod config;
pub mod contexts;
pub mod environment;
