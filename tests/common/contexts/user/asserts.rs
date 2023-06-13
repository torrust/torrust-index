use crate::common::asserts::assert_json_ok;
use crate::common::contexts::user::responses::AddedUserResponse;
use crate::common::responses::TextResponse;

pub fn assert_added_user_response(response: &TextResponse) {
    let _added_user_response: AddedUserResponse = serde_json::from_str(&response.body)
        .unwrap_or_else(|_| panic!("response {:#?} should be a AddedUserResponse", response.body));
    assert_json_ok(response);
}
