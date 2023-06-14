//! The torrust Index Backend API version `v1`.
//!
//! The API is organized in contexts.
//!
//! Refer to the [`contexts`](crate::web::api::v1::contexts) module for more
//! information.
pub mod auth;
pub mod contexts;
pub mod extractors;
pub mod responses;
pub mod routes;
