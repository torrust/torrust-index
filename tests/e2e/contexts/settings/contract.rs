use crate::common::client::Client;
use crate::common::contexts::settings::responses::{AllSettingsResponse, Public, PublicSettingsResponse, SiteNameResponse};
use crate::e2e::contexts::user::steps::new_logged_in_admin;
use crate::e2e::environment::TestEnv;

#[tokio::test]
async fn it_should_allow_guests_to_get_the_public_settings() {
    let mut env = TestEnv::new();
    env.start().await;
    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    let response = client.get_public_settings().await;

    let res: PublicSettingsResponse = serde_json::from_str(&response.body).unwrap();

    assert_eq!(
        res.data,
        Public {
            website_name: env.server_settings().unwrap().website.name,
            tracker_url: env.server_settings().unwrap().tracker.url,
            tracker_mode: env.server_settings().unwrap().tracker.mode,
            email_on_signup: env.server_settings().unwrap().auth.email_on_signup,
        }
    );
    if let Some(content_type) = &response.content_type {
        assert_eq!(content_type, "application/json");
    }
    assert_eq!(response.status, 200);
}

#[tokio::test]
async fn it_should_allow_guests_to_get_the_site_name() {
    let mut env = TestEnv::new();
    env.start().await;
    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    let response = client.get_site_name().await;

    let res: SiteNameResponse = serde_json::from_str(&response.body).unwrap();

    assert_eq!(res.data, "Torrust");
    if let Some(content_type) = &response.content_type {
        assert_eq!(content_type, "application/json");
    }
    assert_eq!(response.status, 200);
}

#[tokio::test]
async fn it_should_allow_admins_to_get_all_the_settings() {
    let mut env = TestEnv::new();
    env.start().await;

    let logged_in_admin = new_logged_in_admin(&env).await;
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

    let response = client.get_settings().await;

    let res: AllSettingsResponse = serde_json::from_str(&response.body).unwrap();

    assert_eq!(res.data, env.server_settings().unwrap());
    if let Some(content_type) = &response.content_type {
        assert_eq!(content_type, "application/json");
    }
    assert_eq!(response.status, 200);
}

#[tokio::test]
async fn it_should_allow_admins_to_update_all_the_settings() {
    let mut env = TestEnv::new();

    if !env.is_isolated() {
        // This test can't be executed in a non-isolated environment because
        // it will change the settings for all the other tests.
        return;
    }

    env.start().await;

    let logged_in_admin = new_logged_in_admin(&env).await;
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

    let mut new_settings = env.server_settings().unwrap();

    new_settings.website.name = "UPDATED NAME".to_string();

    let response = client.update_settings(&new_settings).await;

    let res: AllSettingsResponse = serde_json::from_str(&response.body).unwrap();

    assert_eq!(res.data, new_settings);
    if let Some(content_type) = &response.content_type {
        assert_eq!(content_type, "application/json");
    }
    assert_eq!(response.status, 200);
}
