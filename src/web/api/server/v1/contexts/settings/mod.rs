//! API context: `settings`.
//!
//! This API context is responsible for handling the application settings.
//!
//! # Endpoints
//!
//! - [Get all settings](#get-all-settings)
//! - [Update all settings](#update-all-settings)
//! - [Get site name](#get-site-name)
//! - [Get public settings](#get-public-settings)
//!
//! # Get all settings
//!
//! `GET /v1/settings`
//!
//! Returns all settings.
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI" \
//!   --request GET \
//!   "http://127.0.0.1:3001/v1/settings"
//! ```
//!
//! **Example response** `200`
//!
//! ```json
//! {
//!   "data": {
//!     "version": "2",
//!     "logging": {
//!       "threshold": "info"
//!     },
//!     "website": {
//!       "name": "Torrust"
//!     },
//!     "tracker": {
//!       "api_url": "http://localhost:1212/",
//!       "listed": false,
//!       "private": false,
//!       "token": "***",
//!       "token_valid_seconds": 7257600,
//!       "url": "udp://localhost:6969"
//!     },
//!     "net": {
//!       "base_url": null,
//!       "bind_address": "0.0.0.0:3001",
//!       "tsl": null
//!     },
//!     "auth": {
//!       "secret_key": "***",
//!       "password_constraints": {
//!         "max_password_length": 64,
//!         "min_password_length": 6
//!       }
//!     },
//!     "database": {
//!       "connect_url": "sqlite://data.db?mode=rwc"
//!     },
//!     "mail": {
//!       "email_verification_enabled": false,
//!       "from": "example@email.com",
//!       "reply_to": "noreply@email.com",
//!       "smtp": {
//!         "port": 25,
//!         "server": "",
//!         "credentials": {
//!           "password": "***",
//!           "username": ""
//!         }
//!       }
//!     },
//!     "image_cache": {
//!       "capacity": 128000000,
//!       "entry_size_limit": 4000000,
//!       "max_request_timeout_ms": 1000,
//!       "user_quota_bytes": 64000000,
//!       "user_quota_period_seconds": 3600
//!     },
//!     "api": {
//!       "default_torrent_page_size": 10,
//!       "max_torrent_page_size": 30
//!     },
//!     "registration": {
//!       "email": {
//!         "required": false,
//!         "verified": false
//!       }
//!     },
//!     "tracker_statistics_importer": {
//!       "port": 3002,
//!       "torrent_info_update_interval": 3600
//!     }
//!   }
//! }
//! ```
//! **Resource**
//!
//! Refer to the [`TorrustIndex`](crate::config::Settings)
//! struct for more information about the response attributes.
//!
//! # Update all settings
//!
//! **NOTICE**: This endpoint to update the settings does not work when you use
//! environment variables to configure the application. You need to use a
//! configuration file instead. Because settings are persisted in that file.
//! Refer to the issue [#144](https://github.com/torrust/torrust-index/issues/144)
//! for more information.
//!
//! `POST /v1/settings`
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI" \
//!   --request POST \
//!   --data '{"website":{"name":"Torrust"},"tracker":{"url":"udp://localhost:6969","mode":"public","api_url":"http://localhost:1212/","token":"MyAccessToken","token_valid_seconds":7257600},"net":{"port":3001,"base_url":null},"auth":{"min_password_length":6,"max_password_length":64,"secret_key":"MaxVerstappenWC2021"},"database":{"connect_url":"sqlite://./storage/database/data.db?mode=rwc"},"mail":{"from":"example@email.com","reply_to":"noreply@email.com","username":"","password":"","server":"","port":25},"image_cache":{"max_request_timeout_ms":1000,"capacity":128000000,"entry_size_limit":4000000,"user_quota_period_seconds":3600,"user_quota_bytes":64000000},"api":{"default_torrent_page_size":10,"max_torrent_page_size":30},"tracker_statistics_importer":{"torrent_info_update_interval":3600}}' \
//!   "http://127.0.0.1:3001/v1/settings"
//! ```
//!
//! The response contains the settings that were updated.
//!
//! **Resource**
//!
//! Refer to the [`TorrustIndex`](crate::config::Settings)
//! struct for more information about the response attributes.
//!
//! # Get site name
//!
//! `GET /v1/settings/name`
//!
//! It returns the name of the site.
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --request GET \
//!   "http://127.0.0.1:3001/v1/settings/name"
//! ```
//!
//! **Example response** `200`
//!
//! ```json
//! {
//!   "data":"Torrust"
//! }
//! ```
//!
//! # Get public settings
//!
//! `GET /v1/settings/public`
//!
//! It returns all the public settings.
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --request GET \
//!   "http://127.0.0.1:3001/v1/settings/public"
//! ```
//!
//! **Example response** `200`
//!
//! ```json
//! {
//!   "data": {
//!     "website_name": "Torrust",
//!     "tracker_url": "udp://localhost:6969",
//!     "tracker_mode": "public",
//!     "email_on_signup": "optional"
//!   }
//! }
//! ```
//!
//! **Resource**
//!
//! Refer to the [`ConfigurationPublic`](crate::config::ConfigurationPublic)
//! struct for more information about the response attributes.
pub mod handlers;
pub mod routes;
