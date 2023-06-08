//! API contract for `about` context.
use torrust_index_backend::web::api;

use crate::common::asserts::{assert_response_title, assert_text_ok};
use crate::common::client::Client;
use crate::e2e::environment::TestEnv;

#[tokio::test]
async fn it_should_load_the_about_page_with_information_about_the_api() {
    let mut env = TestEnv::new();
    env.start(api::Implementation::ActixWeb).await;
    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    let response = client.about().await;

    assert_text_ok(&response);
    assert_response_title(&response, "About");
}

#[tokio::test]
async fn it_should_load_the_license_page_at_the_api_entrypoint() {
    let mut env = TestEnv::new();
    env.start(api::Implementation::ActixWeb).await;
    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    let response = client.license().await;

    assert_text_ok(&response);
    assert_response_title(&response, "Licensing");
}

mod with_axum_implementation {
    use torrust_index_backend::web::api;

    use crate::common::asserts::{assert_response_title, assert_text_ok};
    use crate::common::client::Client;
    use crate::e2e::environment::TestEnv;

    #[tokio::test]
    async fn it_should_load_the_about_page_with_information_about_the_api() {
        let mut env = TestEnv::new();
        env.start(api::Implementation::Axum).await;
        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let response = client.about().await;

        assert_text_ok(&response);
        assert_response_title(&response, "About");
    }
}
