//! API context: `category`.
//!
//! This API context is responsible for handling torrent categories.
//!
//! # Endpoints
//!
//! - [Get all categories](#get-all-categories)
//! - [Add a category](#add-a-category)
//! - [Delete a category](#delete-a-category)
//!
//! **NOTICE**: We don't support multiple languages yet, so the category name
//! is always in English.
//!
//! # Get all categories
//!
//! `GET /v1/category`
//!
//! Returns all torrent categories.
//!
//! **Example request**
//!
//! ```bash
//! curl "http://127.0.0.1:3000/v1/category"
//! ```
//!
//! **Example response** `200`
//!
//! ```json
//! {
//!   "data": [
//!     {
//!       "category_id": 3,
//!       "name": "games",
//!       "num_torrents": 0
//!     },
//!     {
//!       "category_id": 1,
//!       "name": "movies",
//!       "num_torrents": 0
//!     },
//!     {
//!       "category_id": 4,
//!       "name": "music",
//!       "num_torrents": 0
//!     },
//!     {
//!       "category_id": 5,
//!       "name": "software",
//!       "num_torrents": 0
//!     },
//!     {
//!       "category_id": 2,
//!       "name": "tv shows",
//!       "num_torrents": 0
//!     }
//!   ]
//! }
//! ```
//! **Resource**
//!
//! Refer to the [`Category`](crate::databases::database::Category)
//! struct for more information about the response attributes.
//!
//! # Add a category
//!
//! `POST /v1/category`
//!
//! It adds a new category.
//!
//! **POST params**
//!
//! Name | Type | Description | Required | Example
//! ---|---|---|---|---
//! `name` | `String` | The name of the category | Yes | `new category`
//! `icon` | `Option<String>` | Icon representing the category | No |
//!
//! **Notice**: the `icon` field is not implemented yet.
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI" \
//!   --request POST \
//!   --data '{"name":"new category","icon":null}' \
//!   http://127.0.0.1:3000/v1/category
//! ```
//!
//! **Example response** `200`
//!
//! ```json
//! {
//!   "data": "new category"
//! }
//! ```
//!
//! **Resource**
//!
//! Refer to [`OkResponse`](crate::models::response::OkResponse<T>) for more
//! information about the response attributes. The response contains only the
//! name of the newly created category.
//!
//! # Delete a category
//!
//! `DELETE /v1/category`
//!
//! It deletes a category.
//!
//! **POST params**
//!
//! Name | Type | Description | Required | Example
//! ---|---|---|---|---
//! `name` | `String` | The name of the category | Yes | `new category`
//! `icon` | `Option<String>` | Icon representing the category | No |
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI" \
//!   --request DELETE \
//!   --data '{"name":"new category","icon":null}' \
//!   http://127.0.0.1:3000/v1/category
//! ```
//!
//! **Example response** `200`
//!
//! ```json
//! {
//!   "data": "new category"
//! }
//! ```
//!
//! **Resource**
//!
//! Refer to [`OkResponse`](crate::models::response::OkResponse<T>) for more
//! information about the response attributes. The response contains only the
//! name of the deleted category.
pub mod handlers;
pub mod routes;
