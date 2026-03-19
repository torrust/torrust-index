use crate::config::v2::tracker::ApiToken;

#[test]
#[should_panic(expected = "tracker API token cannot be empty")]
fn api_token_can_not_be_empty() {
    drop(ApiToken::new(""));
}
