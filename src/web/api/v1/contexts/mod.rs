//! The API is organized in the following contexts:
//!
//! Context | Description | Version
//! ---|---|---
//! `About` | Metadata about the API | [`v1`](crate::web::api::v1::contexts::about)
//! `Category` | Torrent categories | [`v1`](crate::web::api::v1::contexts::category)
//! `Proxy` | Image proxy cache | [`v1`](crate::web::api::v1::contexts::proxy)
//! `Settings` | Index settings | [`v1`](crate::web::api::v1::contexts::settings)
//! `Tag` | Torrent tags | [`v1`](crate::web::api::v1::contexts::tag)
//! `Torrent` | Indexed torrents | [`v1`](crate::web::api::v1::contexts::torrent)
//! `User` | Users | [`v1`](crate::web::api::v1::contexts::user)
//!
pub mod about;
pub mod category;
pub mod proxy;
pub mod settings;
pub mod tag;
pub mod torrent;
pub mod user;
