//! API responses for the [`tag`](crate::web::api::v1::contexts::tag) API
//! context.
use axum::Json;

use crate::models::torrent_tag::TagId;
use crate::web::api::v1::responses::OkResponseData;

/// Response after successfully creating a new tag.
pub fn added_tag(tag_name: &str) -> Json<OkResponseData<String>> {
    Json(OkResponseData {
        data: tag_name.to_string(),
    })
}

/// Response after successfully deleting a tag.
pub fn deleted_tag(tag_id: TagId) -> Json<OkResponseData<String>> {
    Json(OkResponseData {
        data: tag_id.to_string(),
    })
}
