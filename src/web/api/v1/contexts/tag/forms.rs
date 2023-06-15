//! API forms for the the [`tag`](crate::web::api::v1::contexts::tag) API
//! context.
use serde::{Deserialize, Serialize};

use crate::models::torrent_tag::TagId;

#[derive(Serialize, Deserialize, Debug)]
pub struct AddTagForm {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteTagForm {
    pub tag_id: TagId,
}
