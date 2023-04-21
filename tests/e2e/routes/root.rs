use crate::e2e::asserts::{assert_ok, assert_response_title};
use crate::e2e::client::Client;
use crate::e2e::connection_info::connection_with_no_token;
use crate::e2e::http::Query;

#[tokio::test]
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_load_the_about_page_at_the_api_entrypoint() {
    let client = Client::new(connection_with_no_token("localhost:3000"));

    let response = client.get("", Query::empty()).await;

    assert_ok(&response);
    assert_response_title(&response, "About");
}
