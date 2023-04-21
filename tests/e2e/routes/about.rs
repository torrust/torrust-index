use crate::e2e::asserts::{assert_response_title, assert_text_ok};
use crate::e2e::env::TestEnv;

#[tokio::test]
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_load_the_about_page_with_information_about_the_api() {
    let client = TestEnv::default().guess_client();

    let response = client.about().await;

    assert_text_ok(&response);
    assert_response_title(&response, "About");
}

#[tokio::test]
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_load_the_license_page_at_the_api_entrypoint() {
    let client = TestEnv::default().guess_client();

    let response = client.license().await;

    assert_text_ok(&response);
    assert_response_title(&response, "Licensing");
}
