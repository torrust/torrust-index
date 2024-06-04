//! API contract for `user` context.

/*

This test suite is not complete. It's just a starting point to show how to
write E2E tests. Anyway, the goal is not to fully cover all the app features
with E2E tests. The goal is to cover the most important features and to
demonstrate how to write E2E tests. Some important pending tests could be:

todo:

- It should allow renewing a token one week before it expires.
- It should allow verifying user registration via email.

The first one requires to mock the time. Consider extracting the mod
<https://github.com/torrust/torrust-tracker/tree/develop/src/shared/clock> into
an independent crate.

The second one requires:
- To call the mailcatcher API to get the verification URL.
- To enable email verification in the configuration.
- To fix current tests to verify the email for newly created users.
- To find out which email is the one that contains the verification URL for a
given test. That maybe done using the email recipient if that's possible with
the mailcatcher API.

*/

mod registration {

    use torrust_index::web::api;

    use crate::common::client::Client;
    use crate::common::contexts::user::asserts::assert_added_user_response;
    use crate::common::contexts::user::fixtures::random_user_registration_form;
    use crate::e2e::environment::TestEnv;

    #[tokio::test]
    async fn it_should_allow_a_guest_user_to_register() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let form = random_user_registration_form();

        let response = client.register_user(form).await;

        assert_added_user_response(&response);
    }
}

mod authentication {

    use torrust_index::web::api;

    use crate::common::client::Client;
    use crate::common::contexts::user::asserts::{
        assert_successful_login_response, assert_token_renewal_response, assert_token_verified_response,
    };
    use crate::common::contexts::user::fixtures::{DEFAULT_PASSWORD, VALID_PASSWORD};
    use crate::common::contexts::user::forms::{
        ChangePasswordForm, LoginForm, TokenRenewalForm, TokenVerificationForm, Username,
    };
    use crate::e2e::environment::TestEnv;
    use crate::e2e::web::api::v1::contexts::user::steps::{new_logged_in_user, new_registered_user};

    #[tokio::test]
    async fn it_should_allow_a_registered_user_to_login() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let registered_user = new_registered_user(&env).await;

        let response = client
            .login_user(LoginForm {
                login: registered_user.username.clone(),
                password: registered_user.password.clone(),
            })
            .await;

        assert_successful_login_response(&response, &registered_user.username);
    }

    #[tokio::test]
    async fn it_should_allow_logged_in_users_to_change_their_passwords() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let logged_in_user = new_logged_in_user(&env).await;

        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_user.token);

        let new_password = VALID_PASSWORD.to_string();

        let response = client
            .change_password(
                Username::new(logged_in_user.username.clone()),
                ChangePasswordForm {
                    current_password: DEFAULT_PASSWORD.to_string(),
                    password: new_password.clone(),
                    confirm_password: new_password.clone(),
                },
            )
            .await;

        assert_eq!(response.status, 200);

        let response = client
            .login_user(LoginForm {
                login: logged_in_user.username.clone(),
                password: new_password,
            })
            .await;

        assert_successful_login_response(&response, &logged_in_user.username);
    }

    #[tokio::test]
    async fn it_should_allow_a_logged_in_user_to_verify_an_authentication_token() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let logged_in_user = new_logged_in_user(&env).await;

        let response = client
            .verify_token(TokenVerificationForm {
                token: logged_in_user.token.clone(),
            })
            .await;

        assert_token_verified_response(&response);
    }

    #[tokio::test]
    async fn it_should_not_allow_a_logged_in_user_to_renew_an_authentication_token_which_is_still_valid_for_more_than_one_week() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let logged_in_user = new_logged_in_user(&env).await;

        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_user.token);

        let response = client
            .renew_token(TokenRenewalForm {
                token: logged_in_user.token.clone(),
            })
            .await;

        assert_token_renewal_response(&response, &logged_in_user);
    }
}

mod banned_user_list {

    use torrust_index::web::api;

    use crate::common::client::Client;
    use crate::common::contexts::user::asserts::assert_banned_user_response;
    use crate::common::contexts::user::forms::Username;
    use crate::e2e::environment::TestEnv;
    use crate::e2e::web::api::v1::contexts::user::steps::{new_logged_in_admin, new_logged_in_user, new_registered_user};

    #[tokio::test]
    async fn it_should_allow_an_admin_to_ban_a_user() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let logged_in_admin = new_logged_in_admin(&env).await;

        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

        let registered_user = new_registered_user(&env).await;

        let response = client.ban_user(Username::new(registered_user.username.clone())).await;

        assert_banned_user_response(&response, &registered_user);
    }

    #[tokio::test]
    async fn it_should_not_allow_a_non_admin_to_ban_a_user() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let logged_non_admin = new_logged_in_user(&env).await;

        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_non_admin.token);

        let registered_user = new_registered_user(&env).await;

        let response = client.ban_user(Username::new(registered_user.username.clone())).await;

        assert_eq!(response.status, 403);
    }

    #[tokio::test]
    async fn it_should_not_allow_a_guest_to_ban_a_user() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let registered_user = new_registered_user(&env).await;

        let response = client.ban_user(Username::new(registered_user.username.clone())).await;

        assert_eq!(response.status, 401);
    }
}
