use crate::common::asserts::assert_json_ok;
use crate::common::contexts::category::responses::AddedCategoryResponse;
use crate::common::responses::TextResponse;

pub fn assert_added_category_response(response: &TextResponse, category_name: &str) {
    let added_category_response: AddedCategoryResponse = serde_json::from_str(&response.body)
        .unwrap_or_else(|_| panic!("response {:#?} should be a AddedCategoryResponse", response.body));

    assert_eq!(added_category_response.data, category_name);

    assert_json_ok(response);
}
