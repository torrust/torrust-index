//! API contract for `settings` context.

mod for_guest_users {

    use torrust_index::services::settings::EmailOnSignup;
    use torrust_index::web::api;

    use crate::common::asserts::assert_json_ok_response;
    use crate::common::client::Client;
    use crate::common::contexts::settings::responses::{Public, PublicSettingsResponse, SiteNameResponse};
    use crate::e2e::environment::TestEnv;

    #[tokio::test]
    async fn it_should_not_allow_guests_to_get_all_settings() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let response = client.get_settings().await;

        assert_eq!(response.status, 401);
    }

    #[tokio::test]
    async fn it_should_allow_guests_to_get_the_public_settings() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let response = client.get_public_settings().await;

        let res: PublicSettingsResponse = serde_json::from_str(&response.body)
            .unwrap_or_else(|_| panic!("response {:#?} should be a PublicSettingsResponse", response.body));

        let email_on_signup = match &env.server_settings().unwrap().registration {
            Some(registration) => match &registration.email {
                Some(email) => {
                    if email.required {
                        EmailOnSignup::Required
                    } else {
                        EmailOnSignup::Optional
                    }
                }
                None => EmailOnSignup::NotIncluded,
            },
            None => EmailOnSignup::NotIncluded,
        };

        assert_eq!(
            res.data,
            Public {
                website_name: env.server_settings().unwrap().website.name,
                tracker_url: env.server_settings().unwrap().tracker.url,
                tracker_listed: env.server_settings().unwrap().tracker.listed,
                tracker_private: env.server_settings().unwrap().tracker.private,
                email_on_signup: email_on_signup.to_string(),
            }
        );

        assert_json_ok_response(&response);
    }

    #[tokio::test]
    async fn it_should_allow_guests_to_get_the_site_name() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let response = client.get_site_name().await;

        let res: SiteNameResponse = serde_json::from_str(&response.body).unwrap();

        assert_eq!(res.data, "Torrust");

        assert_json_ok_response(&response);
    }
}

mod for_authenticated_users {

    use torrust_index::services::settings::EmailOnSignup;
    use torrust_index::web::api;

    use crate::common::asserts::assert_json_ok_response;
    use crate::common::client::Client;
    use crate::common::contexts::settings::responses::{Public, PublicSettingsResponse, SiteNameResponse};
    use crate::e2e::environment::TestEnv;
    use crate::e2e::web::api::v1::contexts::user::steps::new_logged_in_user;

    #[tokio::test]
    async fn it_should_not_allow_authenticated_users_to_get_all_settings() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let authenticated_user = new_logged_in_user(&env).await;

        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &authenticated_user.token);

        let response = client.get_settings().await;

        assert_eq!(response.status, 403);
    }

    #[tokio::test]
    async fn it_should_allow_authenticated_users_to_get_the_public_settings() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let authenticated_user = new_logged_in_user(&env).await;

        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &authenticated_user.token);

        let response = client.get_public_settings().await;

        let res: PublicSettingsResponse = serde_json::from_str(&response.body)
            .unwrap_or_else(|_| panic!("response {:#?} should be a PublicSettingsResponse", response.body));

        let email_on_signup = match &env.server_settings().unwrap().registration {
            Some(registration) => match &registration.email {
                Some(email) => {
                    if email.required {
                        EmailOnSignup::Required
                    } else {
                        EmailOnSignup::Optional
                    }
                }
                None => EmailOnSignup::NotIncluded,
            },
            None => EmailOnSignup::NotIncluded,
        };

        assert_eq!(
            res.data,
            Public {
                website_name: env.server_settings().unwrap().website.name,
                tracker_url: env.server_settings().unwrap().tracker.url,
                tracker_listed: env.server_settings().unwrap().tracker.listed,
                tracker_private: env.server_settings().unwrap().tracker.private,
                email_on_signup: email_on_signup.to_string(),
            }
        );

        assert_json_ok_response(&response);
    }

    #[tokio::test]
    async fn it_should_allow_authenticated_users_to_get_the_site_name() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let authenticated_user = new_logged_in_user(&env).await;

        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &authenticated_user.token);

        let response = client.get_site_name().await;

        let res: SiteNameResponse = serde_json::from_str(&response.body).unwrap();

        assert_eq!(res.data, "Torrust");

        assert_json_ok_response(&response);
    }
}

mod for_admin_users {
    use torrust_index::services::settings::EmailOnSignup;
    use torrust_index::web::api;

    use crate::common::asserts::assert_json_ok_response;
    use crate::common::client::Client;
    use crate::common::contexts::settings::responses::{AllSettingsResponse, Public, PublicSettingsResponse, SiteNameResponse};
    use crate::e2e::environment::TestEnv;
    use crate::e2e::web::api::v1::contexts::user::steps::new_logged_in_admin;

    #[tokio::test]
    async fn it_should_allow_admins_to_get_all_the_settings() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let logged_in_admin = new_logged_in_admin(&env).await;
        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

        let response = client.get_settings().await;

        let res: AllSettingsResponse = serde_json::from_str(&response.body).unwrap();

        assert_eq!(res.data, env.server_settings_masking_secrets().unwrap());

        assert_json_ok_response(&response);
    }

    #[tokio::test]
    async fn it_should_allow_admins_to_get_the_public_settings() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let logged_in_admin = new_logged_in_admin(&env).await;

        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

        let response = client.get_public_settings().await;

        let res: PublicSettingsResponse = serde_json::from_str(&response.body)
            .unwrap_or_else(|_| panic!("response {:#?} should be a PublicSettingsResponse", response.body));

        let email_on_signup = match &env.server_settings().unwrap().registration {
            Some(registration) => match &registration.email {
                Some(email) => {
                    if email.required {
                        EmailOnSignup::Required
                    } else {
                        EmailOnSignup::Optional
                    }
                }
                None => EmailOnSignup::NotIncluded,
            },
            None => EmailOnSignup::NotIncluded,
        };

        assert_eq!(
            res.data,
            Public {
                website_name: env.server_settings().unwrap().website.name,
                tracker_url: env.server_settings().unwrap().tracker.url,
                tracker_listed: env.server_settings().unwrap().tracker.listed,
                tracker_private: env.server_settings().unwrap().tracker.private,
                email_on_signup: email_on_signup.to_string(),
            }
        );

        assert_json_ok_response(&response);
    }

    #[tokio::test]
    async fn it_should_allow_authenticated_users_to_get_the_site_name() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let logged_in_admin = new_logged_in_admin(&env).await;

        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

        let response = client.get_site_name().await;

        let res: SiteNameResponse = serde_json::from_str(&response.body).unwrap();

        assert_eq!(res.data, "Torrust");

        assert_json_ok_response(&response);
    }
}
