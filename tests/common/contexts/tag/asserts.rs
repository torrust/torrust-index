use torrust_index_backend::models::torrent_tag::TagId;

use crate::common::asserts::assert_json_ok_response;
use crate::common::contexts::tag::responses::{AddedTagResponse, DeletedTagResponse};
use crate::common::responses::TextResponse;

pub fn assert_added_tag_response(response: &TextResponse, tag_name: &str) {
    let added_tag_response: AddedTagResponse = serde_json::from_str(&response.body)
        .unwrap_or_else(|_| panic!("response {:#?} should be a AddedTagResponse", response.body));

    assert_eq!(added_tag_response.data, tag_name);

    assert_json_ok_response(response);
}

pub fn assert_deleted_tag_response(response: &TextResponse, tag_id: TagId) {
    let deleted_tag_response: DeletedTagResponse = serde_json::from_str(&response.body)
        .unwrap_or_else(|_| panic!("response {:#?} should be a DeletedTagResponse", response.body));

    assert_eq!(deleted_tag_response.data, tag_id);

    assert_json_ok_response(response);
}
