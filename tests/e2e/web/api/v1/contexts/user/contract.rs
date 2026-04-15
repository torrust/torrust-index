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
    async fn it_should_fail_changing_the_password_if_the_user_does_not_provide_the_current_password() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let logged_in_user = new_logged_in_user(&env).await;

        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_user.token);

        let response = client
            .change_password(
                Username::new(logged_in_user.username.clone()),
                ChangePasswordForm {
                    current_password: "INVALID PASSWORD".to_string(),
                    password: VALID_PASSWORD.to_string(),
                    confirm_password: VALID_PASSWORD.to_string(),
                },
            )
            .await;

        assert_eq!(response.status, 403);
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
    use crate::e2e::web::api::v1::contexts::user::steps::{new_logged_in_admin, new_registered_user};

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
}

mod authorization {
    mod for_guest_users {
        use torrust_index::web::api;

        use crate::common::client::Client;
        use crate::common::contexts::user::fixtures::{DEFAULT_PASSWORD, VALID_PASSWORD, random_user_registration_form};
        use crate::common::contexts::user::forms::{ChangePasswordForm, Username};
        use crate::e2e::environment::TestEnv;
        use crate::e2e::web::api::v1::contexts::user::steps::{new_logged_in_user, new_registered_user};

        #[tokio::test]
        async fn it_should_allow_a_guest_user_to_register() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

            let form = random_user_registration_form();

            let response = client.register_user(form).await;

            assert_eq!(response.status, 200);
        }

        #[tokio::test]
        async fn it_should_not_allow_guest_users_to_change_passwords() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            let logged_in_user = new_logged_in_user(&env).await;

            let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

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

            assert_eq!(response.status, 401);
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

    mod for_registered_users {
        use torrust_index::web::api;

        use crate::common::client::Client;
        use crate::common::contexts::user::fixtures::{DEFAULT_PASSWORD, VALID_PASSWORD};
        use crate::common::contexts::user::forms::{ChangePasswordForm, RegistrationForm, Username};
        use crate::e2e::environment::TestEnv;
        use crate::e2e::web::api::v1::contexts::user::steps::{new_logged_in_user, new_registered_user};

        #[tokio::test]
        async fn it_should_not_allow_a_registered_user_to_register() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            let logged_in_user = new_logged_in_user(&env).await;

            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_user.token);

            let response = client
                .register_user(RegistrationForm {
                    username: logged_in_user.username,
                    email: Some("test@email.com".to_string()),
                    password: VALID_PASSWORD.to_string(),
                    confirm_password: VALID_PASSWORD.to_string(),
                })
                .await;

            assert_eq!(response.status, 400);
        }

        #[tokio::test]
        async fn it_should_allow_registered_users_to_change_their_passwords() {
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
        }
        #[tokio::test]
        async fn it_should_not_allow_a_registered_user_to_ban_a_user() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            let logged_in_user = new_logged_in_user(&env).await;

            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_user.token);

            let registered_user = new_registered_user(&env).await;

            let response = client.ban_user(Username::new(registered_user.username.clone())).await;

            assert_eq!(response.status, 403);
        }
    }
    mod for_admin_users {
        use torrust_index::web::api;

        use crate::common::client::Client;
        use crate::common::contexts::user::fixtures::{DEFAULT_PASSWORD, VALID_PASSWORD};
        use crate::common::contexts::user::forms::{ChangePasswordForm, RegistrationForm, Username};
        use crate::e2e::environment::TestEnv;
        use crate::e2e::web::api::v1::contexts::user::steps::{new_logged_in_admin, new_registered_user};

        #[tokio::test]
        async fn it_should_not_allow_an_admin_user_to_register() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            let logged_in_admin = new_logged_in_admin(&env).await;

            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

            let response = client
                .register_user(RegistrationForm {
                    username: logged_in_admin.username,
                    email: Some("test@email.com".to_string()),
                    password: VALID_PASSWORD.to_string(),
                    confirm_password: VALID_PASSWORD.to_string(),
                })
                .await;

            assert_eq!(response.status, 400);
        }

        #[tokio::test]
        async fn it_should_allow_admin_users_to_change_their_passwords() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            let logged_in_admin = new_logged_in_admin(&env).await;

            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

            let new_password = VALID_PASSWORD.to_string();

            let response = client
                .change_password(
                    Username::new(logged_in_admin.username.clone()),
                    ChangePasswordForm {
                        current_password: DEFAULT_PASSWORD.to_string(),
                        password: new_password.clone(),
                        confirm_password: new_password.clone(),
                    },
                )
                .await;

            assert_eq!(response.status, 200);
        }

        #[tokio::test]
        async fn it_should_allow_an_admin_to_ban_a_user() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            let logged_in_admin = new_logged_in_admin(&env).await;

            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

            let registered_user = new_registered_user(&env).await;

            let response = client.ban_user(Username::new(registered_user.username.clone())).await;

            assert_eq!(response.status, 200);
        }
    }
}

mod permissions_discovery {

    use torrust_index::services::authorization::Action;
    use torrust_index::web::api;

    use crate::common::client::Client;
    use crate::common::contexts::user::responses::MyPermissionsResponse;
    use crate::e2e::environment::TestEnv;
    use crate::e2e::web::api::v1::contexts::user::steps::{new_logged_in_admin, new_logged_in_user};

    #[tokio::test]
    async fn it_should_return_all_actions_for_an_admin() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let logged_in_admin = new_logged_in_admin(&env).await;

        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

        let response = client.get_my_permissions().await;

        assert_eq!(response.status, 200);

        let permissions: MyPermissionsResponse = serde_json::from_str(&response.body)
            .unwrap_or_else(|_| panic!("response should be MyPermissionsResponse: {}", response.body));

        assert_eq!(permissions.data.role, "admin");

        // Admin is granted every action.
        let mut expected: Vec<String> = Action::ALL.iter().map(|a| format!("{a}")).collect();
        let mut actual = permissions.data.actions;
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn it_should_return_registered_actions_for_a_registered_user() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let logged_in_user = new_logged_in_user(&env).await;

        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_user.token);

        let response = client.get_my_permissions().await;

        assert_eq!(response.status, 200);

        let permissions: MyPermissionsResponse = serde_json::from_str(&response.body)
            .unwrap_or_else(|_| panic!("response should be MyPermissionsResponse: {}", response.body));

        assert_eq!(permissions.data.role, "registered");

        let mut actual = permissions.data.actions;
        actual.sort();

        let mut expected = vec![
            "GetAboutPage",
            "GetLicensePage",
            "GetCategories",
            "GetImageByUrl",
            "GetPublicSettings",
            "GetSiteName",
            "GetTags",
            "AddTorrent",
            "GetTorrent",
            "GetTorrentInfo",
            "GenerateTorrentInfoListing",
            "ChangePassword",
            "UpdateTorrent",
            "GetMyPermissions",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
        expected.sort();

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn it_should_return_guest_actions_when_no_token_is_provided() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let response = client.get_my_permissions().await;

        assert_eq!(response.status, 200);

        let permissions: MyPermissionsResponse = serde_json::from_str(&response.body)
            .unwrap_or_else(|_| panic!("response should be MyPermissionsResponse: {}", response.body));

        assert_eq!(permissions.data.role, "guest");

        let mut actual = permissions.data.actions;
        actual.sort();

        let mut expected = vec![
            "GetAboutPage",
            "GetLicensePage",
            "GetCategories",
            "GetPublicSettings",
            "GetSiteName",
            "GetTags",
            "GetTorrent",
            "GetTorrentInfo",
            "GenerateTorrentInfoListing",
            "GetMyPermissions",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
        expected.sort();

        assert_eq!(actual, expected);
    }
}

mod permission_overrides {

    use torrust_index::services::authorization::{Action, Effect, PermissionOverride, Role};
    use torrust_index::web::api;

    use crate::common::client::Client;
    use crate::common::contexts::torrent::fixtures::random_torrent;
    use crate::common::contexts::torrent::forms::UploadTorrentMultipartForm;
    use crate::common::contexts::user::responses::MyPermissionsResponse;
    use crate::e2e::environment::TestEnv;
    use crate::e2e::web::api::v1::contexts::user::steps::new_logged_in_user;

    #[tokio::test]
    async fn it_should_grant_a_normally_denied_action_via_toml_override() {
        let mut env = TestEnv::new();
        env.start_with(api::Version::V1, |cfg| {
            cfg.permissions.overrides = vec![PermissionOverride {
                role: Role::Registered,
                action: Action::DeleteTorrent,
                effect: Effect::Allow,
            }];
        })
        .await;

        let logged_in_user = new_logged_in_user(&env).await;
        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_user.token);

        let response = client.get_my_permissions().await;

        assert_eq!(response.status, 200);

        let permissions: MyPermissionsResponse = serde_json::from_str(&response.body)
            .unwrap_or_else(|_| panic!("response should be MyPermissionsResponse: {}", response.body));

        assert_eq!(permissions.data.role, "registered");
        assert!(
            permissions.data.actions.contains(&"DeleteTorrent".to_string()),
            "expected DeleteTorrent in actions after TOML override grant, got: {:?}",
            permissions.data.actions
        );
    }

    #[tokio::test]
    async fn it_should_deny_a_normally_allowed_action_via_toml_override() {
        let mut env = TestEnv::new();
        env.start_with(api::Version::V1, |cfg| {
            cfg.permissions.overrides = vec![PermissionOverride {
                role: Role::Registered,
                action: Action::AddTorrent,
                effect: Effect::Deny,
            }];
        })
        .await;

        let logged_in_user = new_logged_in_user(&env).await;
        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_user.token);

        // Verify the action is absent from /me/permissions
        let response = client.get_my_permissions().await;

        let permissions: MyPermissionsResponse = serde_json::from_str(&response.body)
            .unwrap_or_else(|_| panic!("response should be MyPermissionsResponse: {}", response.body));

        assert!(
            !permissions.data.actions.contains(&"AddTorrent".to_string()),
            "expected AddTorrent to be absent after TOML override deny, got: {:?}",
            permissions.data.actions
        );

        // Attempt to upload — the extractor should reject with 403
        // before the handler parses the multipart body.
        let test_torrent = random_torrent();
        let form: UploadTorrentMultipartForm = test_torrent.index_info.into();
        let response = client
            .upload_torrent(form.try_into().expect("multipart form should be valid"))
            .await;

        assert_eq!(response.status, 403);
    }
}
