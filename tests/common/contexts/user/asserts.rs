use super::forms::RegistrationForm;
use crate::common::asserts::assert_json_ok;
use crate::common::contexts::user::responses::{AddedUserResponse, SuccessfulLoginResponse, TokenVerifiedResponse};
use crate::common::responses::TextResponse;

pub fn assert_added_user_response(response: &TextResponse) {
    let _added_user_response: AddedUserResponse = serde_json::from_str(&response.body)
        .unwrap_or_else(|_| panic!("response {:#?} should be a AddedUserResponse", response.body));
    assert_json_ok(response);
}

pub fn assert_successful_login_response(response: &TextResponse, registered_user: &RegistrationForm) {
    let successful_login_response: SuccessfulLoginResponse = serde_json::from_str(&response.body)
        .unwrap_or_else(|_| panic!("response {:#?} should be a SuccessfulLoginResponse", response.body));

    let logged_in_user = successful_login_response.data;

    assert_eq!(logged_in_user.username, registered_user.username);

    assert_json_ok(response);
}

pub fn assert_token_verified_response(response: &TextResponse) {
    let token_verified_response: TokenVerifiedResponse = serde_json::from_str(&response.body)
        .unwrap_or_else(|_| panic!("response {:#?} should be a TokenVerifiedResponse", response.body));

    assert_eq!(token_verified_response.data, "Token is valid.");

    assert_json_ok(response);
}
