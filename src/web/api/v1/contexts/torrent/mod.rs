//! API context: `torrent`.
//!
//! This API context is responsible for handling all torrent related requests.
//!
//! # Endpoints
//!
//! - [Upload new torrent](#upload-new-torrent)
//! - [Download a torrent](#download-a-torrent)
//! - [Get torrent info](#get-torrent-info)
//! - [List torrent infos](#list-torrent-infos)
//! - [Update torrent info](#update-torrent-info)
//! - [Delete a torrent](#delete-a-torrent)
//!
//! # Upload new torrent
//!
//! `POST /v1/torrent/upload`
//!
//! It uploads a new torrent to the index.
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: multipart/form-data" \
//!   --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI" \
//!   --request POST \
//!   --form "title=MandelbrotSet" \
//!   --form "description=MandelbrotSet image" \
//!   --form "category=software" \
//!   --form "torrent=@docs/media/mandelbrot_2048x2048_infohash_v1.png.torrent;type=application/x-bittorrent" \
//!   "http://127.0.0.1:3000/v1/torrent/upload"
//! ```
//!
//! **Example response** `200`
//!
//! ```json
//! {
//!   "data": {
//!     "torrent_id": 2,
//!     "info_hash": "5452869BE36F9F3350CCEE6B4544E7E76CAAADAB"
//!   }
//! }
//! ```
//!
//! **NOTICE**: Info-hashes will be lowercase hex-encoded strings in the future
//! and the [internal database ID could be removed from the response](https://github.com/torrust/torrust-index-backend/discussions/149).
//!
//! **Resource**
//!
//! Refer to the [`TorrustBackend`](crate::models::response::NewTorrentResponse)
//! struct for more information about the response attributes.
//!
//! # Download a torrent
//!
//! `GET /v1/torrent/download/{info_hash}`
//!
//! It downloads a new torrent file from the the index.
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/x-bittorrent" \
//!   --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI" \
//!   --output mandelbrot_2048x2048_infohash_v1.png.torrent \
//!   "http://127.0.0.1:3000/v1/torrent/download/5452869BE36F9F3350CCEE6B4544E7E76CAAADAB"
//! ```
//!
//! **Example response** `200`
//!
//! The response is a torrent file `mandelbrot_2048x2048_infohash_v1.png.torrent`.
//!
//! ```text
//! $ imdl torrent show mandelbrot_2048x2048_infohash_v1.png.torrent
//!          Name  mandelbrot_2048x2048.png
//!     Info Hash  1a326de411f96bc15622c62358130f0824f561e1
//!  Torrent Size  492 bytes
//!  Content Size  168.17 KiB
//!       Private  no
//!       Tracker  udp://localhost:6969/eklijkg8901K2Ol6O6CttT1xlUzO4bFD
//! Announce List  Tier 1: udp://localhost:6969/eklijkg8901K2Ol6O6CttT1xlUzO4bFD
//!                Tier 2: udp://localhost:6969
//!    Piece Size  16 KiB
//!   Piece Count  11
//!    File Count  1
//!         Files  mandelbrot_2048x2048.png
//! ```
//!
//! # Get torrent info
//!
//! `GET /v1/torrents/{info_hash}`
//!
//! It returns the torrent info.
//!
//! **Path parameters**
//!
//! Name | Type | Description | Required | Example
//! ---|---|---|---|---
//! `info_hash` | `InfoHash` | The info-hash | Yes | `5452869BE36F9F3350CCEE6B4544E7E76CAAADAB`
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI" \
//!   --request GET \
//!   "http://127.0.0.1:3000/v1/torrent/5452869BE36F9F3350CCEE6B4544E7E76CAAADAB"
//! ```
//!
//! **Example response** `200`
//!
//! ```json
//! {
//!   "data": {
//!     "torrent_id": 2,
//!     "uploader": "indexadmin",
//!     "info_hash": "5452869BE36F9F3350CCEE6B4544E7E76CAAADAB",
//!     "title": "MandelbrotSet",
//!     "description": "MandelbrotSet image",
//!     "category": {
//!       "category_id": 5,
//!       "name": "software",
//!       "num_torrents": 1
//!     },
//!     "upload_date": "2023-05-25 11:33:02",
//!     "file_size": 172204,
//!     "seeders": 0,
//!     "leechers": 0,
//!     "files": [
//!       {
//!         "path": [
//!           "mandelbrot_2048x2048.png"
//!         ],
//!         "length": 172204,
//!         "md5sum": null
//!       }
//!     ],
//!     "trackers": [
//!       "udp://localhost:6969/eklijkg8901K2Ol6O6CttT1xlUzO4bFD",
//!       "udp://localhost:6969"
//!     ],
//!     "magnet_link": "magnet:?xt=urn:btih:5452869BE36F9F3350CCEE6B4544E7E76CAAADAB&dn=MandelbrotSet&tr=udp%3A%2F%2Flocalhost%3A6969%2Feklijkg8901K2Ol6O6CttT1xlUzO4bFD&tr=udp%3A%2F%2Flocalhost%3A6969"
//!   }
//! }
//! ```
//!
//! **Resource**
//!
//! Refer to the [`TorrentResponse`](crate::models::response::TorrentResponse)
//! struct for more information about the response attributes.
//!
//! # List torrent infos
//!
//! `GET /v1/torrents`
//!
//! It returns the torrent info for multiple torrents
//!
//! **Get parameters**
//!
//! Name | Type | Description | Required | Example
//! ---|---|---|---|---
//! `search` | `Option<String>` | A text to search | No | `MandelbrotSet`
//! `categories` | `Option<String>` | A coma-separated category list | No | `music,other,movie,software`
//!
//! **Pagination GET parameters**
//!
//! Name | Type | Description | Required | Example
//! ---|---|---|---|---
//! `page_size` | `Option<u8>` | Number of torrents per page | No | `10`
//! `page` | `Option<u32>` | Page offset, starting at `0` | No | `music,other,movie,software`
//!
//! Pagination default values can be configured in the server configuration file.
//!
//! ```toml
//! [api]
//! default_torrent_page_size = 10
//! max_torrent_page_size = 30
//! ```
//!
//! **Sorting GET parameters**
//!
//! Name | Type | Description | Required | Example
//! ---|---|---|---|---
//! `sort` | `Option<Sorting>` | [Sorting](crate::databases::database::Sorting) options | No | `size_DESC`
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI" \
//!   --request GET \
//!   "http://127.0.0.1:3000/v1/torrents"
//! ```
//!
//! **Example response** `200`
//!
//! ```json
//! {
//!   "data": {
//!     "total": 1,
//!     "results": [
//!       {
//!         "torrent_id": 2,
//!         "uploader": "indexadmin",
//!         "info_hash": "5452869BE36F9F3350CCEE6B4544E7E76CAAADAB",
//!         "title": "MandelbrotSet",
//!         "description": "MandelbrotSet image",
//!         "category_id": 5,
//!         "date_uploaded": "2023-05-25 11:33:02",
//!         "file_size": 172204,
//!         "seeders": 0,
//!         "leechers": 0
//!       }
//!     ]
//!   }
//! }
//! ```
//!
//! **Resource**
//!
//! Refer to the [`TorrentsResponse`](crate::models::response::TorrentsResponse)
//! struct for more information about the response attributes.
//!
//! # Update torrent info
//!
//! `POST /v1/torrents/{info_hash}`
//!
//! It updates the torrent info.
//!
//! **Path parameters**
//!
//! Name | Type | Description | Required | Example
//! ---|---|---|---|---
//! `info_hash` | `InfoHash` | The info-hash | Yes | `5452869BE36F9F3350CCEE6B4544E7E76CAAADAB`
//!
//! **Post parameters**
//!
//! Name | Type | Description | Required | Example
//! ---|---|---|---|---
//! `title` | `Option<String>` | The torrent title | No | `MandelbrotSet`
//! `description` | `Option<String>` | The torrent description  | No | `MandelbrotSet image`
//! `category` | `Option<CategoryId>` | The torrent category ID  | No | `1`
//! `tags` | `Option<Vec<TagId>>` | The tag Id list  | No | `[1,2,3]`
//!
//!
//! Refer to the [`UpdateTorrentInfoForm`](crate::web::api::v1::contexts::torrent::forms::UpdateTorrentInfoForm)
//! struct for more information about the request attributes.
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI" \
//!   --request PUT \
//!   --data '{"title":"MandelbrotSet", "description":"MandelbrotSet image"}' \
//!   "http://127.0.0.1:3000/v1/torrent/5452869BE36F9F3350CCEE6B4544E7E76CAAADAB"
//! ```
//!
//! **Example response** `200`
//!
//! ```json
//! {
//!   "data": {
//!     "torrent_id": 2,
//!     "uploader": "indexadmin",
//!     "info_hash": "5452869BE36F9F3350CCEE6B4544E7E76CAAADAB",
//!     "title": "MandelbrotSet",
//!     "description": "MandelbrotSet image",
//!     "category": {
//!       "category_id": 5,
//!       "name": "software",
//!       "num_torrents": 1
//!     },
//!     "upload_date": "2023-05-25 11:33:02",
//!     "file_size": 172204,
//!     "seeders": 0,
//!     "leechers": 0,
//!     "files": [],
//!     "trackers": [],
//!     "magnet_link": ""
//!   }
//! }
//! ```
//!
//! **NOTICE**: the response is not the same as the `GET /v1/torrents/{info_hash}`.
//! It does not contain the `files`, `trackers` and `magnet_link` attributes.
//!
//! **Resource**
//!
//! Refer to the [`TorrentResponse`](crate::models::response::TorrentResponse)
//! struct for more information about the response attributes.
//!
//! # Delete a torrent
//!
//! `DELETE /v1/torrents/{info_hash}`
//!
//! It deletes a torrent.
//!
//! **Path parameters**
//!
//! Name | Type | Description | Required | Example
//! ---|---|---|---|---
//! `info_hash` | `InfoHash` | The info-hash | Yes | `5452869BE36F9F3350CCEE6B4544E7E76CAAADAB`
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI" \
//!   --request DELETE \
//!   "http://127.0.0.1:3000/v1/torrent/5452869BE36F9F3350CCEE6B4544E7E76CAAADAB"
//! ```
//!
//! **Example response** `200`
//!
//! ```json
//! {
//!   "data": {
//!     "torrent_id": 2,
//!     "info_hash": "5452869BE36F9F3350CCEE6B4544E7E76CAAADAB",
//!   }
//! }
//! ```
//!
//! **Resource**
//!
//! Refer to the [`DeletedTorrentResponse`](crate::models::response::DeletedTorrentResponse)
//! struct for more information about the response attributes.
pub mod forms;
pub mod handlers;
pub mod responses;
pub mod routes;
