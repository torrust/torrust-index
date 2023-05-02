use crate::common::asserts::{assert_response_title, assert_text_ok};
use crate::environments::isolated::TestEnv;

#[tokio::test]
async fn it_should_load_the_about_page_with_information_about_the_api() {
    let client = TestEnv::running().await.unauthenticated_client();

    let response = client.about().await;

    assert_text_ok(&response);
    assert_response_title(&response, "About");
}

#[tokio::test]
async fn it_should_load_the_license_page_at_the_api_entrypoint() {
    let client = TestEnv::running().await.unauthenticated_client();

    let response = client.license().await;

    assert_text_ok(&response);
    assert_response_title(&response, "Licensing");
}
