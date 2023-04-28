//! API contract for `root` context.
use crate::common::asserts::{assert_response_title, assert_text_ok};
use crate::e2e::environment::TestEnv;

#[tokio::test]
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_load_the_about_page_at_the_api_entrypoint() {
    let client = TestEnv::default().unauthenticated_client();

    let response = client.root().await;

    assert_text_ok(&response);
    assert_response_title(&response, "About");
}
