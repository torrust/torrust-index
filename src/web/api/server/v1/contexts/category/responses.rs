//! API responses for the the [`category`](crate::web::api::server::v1::contexts::category) API
//! context.
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::databases::database::Category as DatabaseCategory;
use crate::web::api::server::v1::responses::OkResponseData;

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    /// Deprecated. Use `id`.
    pub category_id: i64, // todo: remove when the Index GUI uses the new `id` field.
    pub name: String,
    pub num_torrents: i64,
}

/// Response after successfully creating a new category.
pub fn added_category(category_name: &str) -> Json<OkResponseData<String>> {
    Json(OkResponseData {
        data: category_name.to_string(),
    })
}

/// Response after successfully deleting a new category.
pub fn deleted_category(category_name: &str) -> Json<OkResponseData<String>> {
    Json(OkResponseData {
        data: category_name.to_string(),
    })
}

impl From<DatabaseCategory> for Category {
    fn from(db_category: DatabaseCategory) -> Self {
        Category {
            id: db_category.category_id,
            category_id: db_category.category_id,
            name: db_category.name,
            num_torrents: db_category.num_torrents,
        }
    }
}
