//! API contract for `about` context.

use torrust_index::web::api;

use crate::common::asserts::{assert_response_title, assert_text_ok};
use crate::common::client::Client;
use crate::e2e::environment::TestEnv;

#[tokio::test]
async fn it_should_load_the_about_page_with_information_about_the_api() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;
    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    let response = client.about().await;

    assert_text_ok(&response);
    assert_response_title(&response, "About");
}

#[tokio::test]
async fn it_should_load_the_license_page_at_the_api_entrypoint() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;
    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    let response = client.license().await;

    assert_text_ok(&response);
    assert_response_title(&response, "Licensing");
}

mod for_guest_users {
    use torrust_index::web::api;

    use crate::common::asserts::assert_text_ok;
    use crate::common::client::Client;
    use crate::e2e::environment::TestEnv;

    #[tokio::test]
    async fn it_should_allow_guest_users_to_see_the_about_page() {
        let mut env = TestEnv::new();

        env.start(api::Version::V1).await;

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let response = client.about().await;
        assert_text_ok(&response);
    }

    #[tokio::test]
    async fn it_should_allow_guest_users_to_see_the_license_page() {
        let mut env = TestEnv::new();

        env.start(api::Version::V1).await;

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let response = client.license().await;
        assert_text_ok(&response);
    }
}

mod for_authenticated_users {
    use torrust_index::web::api;

    use crate::common::asserts::assert_text_ok;
    use crate::common::client::Client;
    use crate::e2e::environment::TestEnv;
    use crate::e2e::web::api::v1::contexts::user::steps::new_logged_in_user;

    #[tokio::test]
    async fn it_should_allow_authenticated_users_to_see_the_about_page() {
        let mut env = TestEnv::new();

        env.start(api::Version::V1).await;

        let authenticated_user = new_logged_in_user(&env).await;

        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &authenticated_user.token);

        let response = client.about().await;
        assert_text_ok(&response);
    }

    #[tokio::test]
    async fn it_should_allow_authenticated_users_to_see_the_license_page() {
        let mut env = TestEnv::new();

        env.start(api::Version::V1).await;

        let authenticated_user = new_logged_in_user(&env).await;

        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &authenticated_user.token);

        let response = client.license().await;
        assert_text_ok(&response);
    }
}

mod for_admin_users {
    use torrust_index::web::api;

    use crate::common::asserts::assert_text_ok;
    use crate::common::client::Client;
    use crate::e2e::environment::TestEnv;
    use crate::e2e::web::api::v1::contexts::user::steps::new_logged_in_admin;

    #[tokio::test]
    async fn it_should_allow_admin_users_to_see_the_about_page() {
        let mut env = TestEnv::new();

        env.start(api::Version::V1).await;

        let logged_in_admin = new_logged_in_admin(&env).await;

        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

        let response = client.about().await;
        assert_text_ok(&response);
    }

    #[tokio::test]
    async fn it_should_allow_admin_users_to_see_the_license_page() {
        let mut env = TestEnv::new();

        env.start(api::Version::V1).await;

        let logged_in_admin = new_logged_in_admin(&env).await;

        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

        let response = client.license().await;
        assert_text_ok(&response);
    }
}
