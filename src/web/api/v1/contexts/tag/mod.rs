//! API context: `tag`.
//!
//! This API context is responsible for handling torrent tags.
//!
//! # Endpoints
//!
//! - [Get all tags](#get-all-tags)
//! - [Add a tag](#add-a-tag)
//! - [Delete a tag](#delete-a-tag)
//!
//! **NOTICE**: We don't support multiple languages yet, so the tag is always
//! in English.
//!
//! # Get all tags
//!
//! `GET /v1/tag`
//!
//! Returns all torrent tags.
//!
//! **Example request**
//!
//! ```bash
//! curl "http://127.0.0.1:3000/v1/tags"
//! ```
//!
//! **Example response** `200`
//!
//! ```json
//! {
//!   "data": [
//!     {
//!       "tag_id": 1,
//!       "name": "anime"
//!     },
//!     {
//!       "tag_id": 2,
//!       "name": "manga"
//!     }
//!   ]
//! }
//! ```
//! **Resource**
//!
//! Refer to the [`Tag`](crate::databases::database::Tag)
//! struct for more information about the response attributes.
//!
//! # Add a tag
//!
//! `POST /v1/tag`
//!
//! It adds a new tag.
//!
//! **POST params**
//!
//! Name | Type | Description | Required | Example
//! ---|---|---|---|---
//! `name` | `String` | The tag name | Yes | `new tag`
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI" \
//!   --request POST \
//!   --data '{"name":"new tag"}' \
//!   http://127.0.0.1:3000/v1/tag
//! ```
//!
//! **Example response** `200`
//!
//! ```json
//! {
//!   "data": "new tag"
//! }
//! ```
//!
//! **Resource**
//!
//! Refer to [`OkResponse`](crate::models::response::OkResponse<T>) for more
//! information about the response attributes. The response contains only the
//! name of the newly created tag.
//!
//! # Delete a tag
//!
//! `DELETE /v1/tag`
//!
//! It deletes a tag.
//!
//! **POST params**
//!
//! Name | Type | Description | Required | Example
//! ---|---|---|---|---
//! `tag_id` | `i64` | The internal tag ID | Yes | `1`
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI" \
//!   --request DELETE \
//!   --data '{"tag_id":1}' \
//!   http://127.0.0.1:3000/v1/tag
//! ```
//!
//! **Example response** `200`
//!
//! ```json
//! {
//!   "data": 1
//! }
//! ```
//!
//! **Resource**
//!
//! Refer to [`OkResponse`](crate::models::response::OkResponse<T>) for more
//! information about the response attributes. The response contains only the
//! name of the deleted tag.
pub mod forms;
pub mod handlers;
pub mod responses;
pub mod routes;
