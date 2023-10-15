//! API context: `torrent`.
//!
//! This API context is responsible for handling all torrent related requests.
//!
//! # Original and canonical infohashes
//!
//! Uploaded torrents can contain non-standard fields in the `info` dictionary.
//!
//! For example, this is a torrent file in JSON format with a "custom" field.
//!
//! ```json
//! {
//!     "info": {
//!        "length": 602515,
//!        "name": "mandelbrot_set_01",
//!        "piece length": 32768,
//!        "pieces": "<hex>8A 88 32 BE ED 05 5F AA C4 AF 4A 90 4B 9A BF 0D EC 83 42 1C 73 39 05 B8 D6 20 2C 1B D1 8A 53 28 1F B5 D4 23 0A 23 C8 DB AC C4 E6 6B 16 12 08 C7 A4 AD 64 45 70 ED 91 0D F1 38 E7 DF 0C 1A D0 C9 23 27 7C D1 F9 D4 E5 A1 5F F5 E5 A0 E4 9E FB B1 43 F5 4B AD 0E D4 9D CB 49 F7 E6 7B BA 30 5F AF F9 88 56 FB 45 9A B4 95 92 3E 2C 7F DA A6 D3 82 E7 63 A3 BB 4B 28 F3 57 C7 CB 7D 8C 06 E3 46 AB D7 E8 8E 8A 8C 9F C7 E6 C5 C5 64 82 ED 47 BB 2A F1 B7 3F A5 3C 5B 9C AF 43 EC 2A E1 08 68 9A 49 C8 BF 1B 07 AD BE E9 2D 7E BE 9C 18 7F 4C A1 97 0E 54 3A 18 94 0E 60 8D 5C 69 0E 41 46 0D 3C 9A 37 F6 81 62 4F 95 C0 73 92 CA 9A D5 A9 89 AC 8B 85 12 53 0B FB E2 96 26 3E 26 A6 5B 70 53 48 65 F3 6C 27 0F 6B BD 1C EE EB 1A 9D 5F 77 A8 D8 AF D8 14 82 4A E0 B4 62 BC F1 A5 F5 F2 C7 60 F8 38 C8 5B 0B A9 07 DD 86 FA C0 7B F0 26 D7 D1 9A 42 C3 1F 9F B9 59 83 10 62 41 E9 06 3C 6D A1 19 75 01 57 25 9E B7 FE DF 91 04 D4 51 4B 6D 44 02 8D 31 8E 84 26 95 0F 30 31 F0 2C 16 39 BD 53 1D CF D3 5E 3E 41 A9 1E 14 3F 73 24 AC 5E 9E FC 4D C5 70 45 0F 45 8B 9B 52 E6 D0 26 47 8F 43 08 9E 2A 7C C5 92 D5 86 36 FE 48 E9 B8 86 84 92 23 49 5B EE C4 31 B2 1D 10 75 8E 4C 07 84 8F</hex>",
//!        "custom": "custom03"
//!     }
//! }
//! ```
//!
//! When you upload a torrent file with non-standards fields in the `info`
//! dictionary, the Index removes those non-standard fields. That generates a
//! new info-hash because all fields in the `info` key are used to calculate it.
//!
//! The Index stores the original info-hash. The resulting info-hash after
//! removing the non-standard fields is called "canonical" infohash. The Index
//! stores the relationship between the original info-hash and the canonical one.
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
//!   "http://127.0.0.1:3001/v1/torrent/upload"
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
//! and the [internal database ID could be removed from the response](https://github.com/torrust/torrust-index/discussions/149).
//!
//! **Resource**
//!
//! Refer to the [`TorrustIndex`](crate::models::response::NewTorrentResponse)
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
//!   "http://127.0.0.1:3001/v1/torrent/download/5452869BE36F9F3350CCEE6B4544E7E76CAAADAB"
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
//!   "http://127.0.0.1:3001/v1/torrent/5452869BE36F9F3350CCEE6B4544E7E76CAAADAB"
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
//!   "http://127.0.0.1:3001/v1/torrents"
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
//!   "http://127.0.0.1:3001/v1/torrent/5452869BE36F9F3350CCEE6B4544E7E76CAAADAB"
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
//!   "http://127.0.0.1:3001/v1/torrent/5452869BE36F9F3350CCEE6B4544E7E76CAAADAB"
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
pub mod errors;
pub mod forms;
pub mod handlers;
pub mod responses;
pub mod routes;
