use crate::common::client::Client;
use crate::common::contexts::settings::form::UpdateSettingsForm;
use crate::common::contexts::settings::responses::{AllSettingsResponse, Public, PublicSettingsResponse, SiteNameResponse};
use crate::common::contexts::settings::{Auth, Database, Mail, Net, Settings, Tracker, Website};
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
            website_name: "Torrust".to_string(),
            tracker_url: env.tracker_url(),
            tracker_mode: "Public".to_string(),
            email_on_signup: "Optional".to_string(),
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
    env.start().await;

    let logged_in_admin = new_logged_in_admin(&env).await;
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

    // todo: we can't actually change the settings because it would affect other E2E tests.
    // Location for the `config.toml` file is hardcoded. We could use a ENV variable to change it.

    let response = client
        .update_settings(UpdateSettingsForm {
            website: Website {
                name: "Torrust".to_string(),
            },
            tracker: Tracker {
                url: "udp://tracker:6969".to_string(),
                mode: "Public".to_string(),
                api_url: "http://tracker:1212".to_string(),
                token: "MyAccessToken".to_string(),
                token_valid_seconds: 7_257_600,
            },
            net: Net {
                port: 3000,
                base_url: None,
            },
            auth: Auth {
                email_on_signup: "Optional".to_string(),
                min_password_length: 6,
                max_password_length: 64,
                secret_key: "MaxVerstappenWC2021".to_string(),
            },
            database: Database {
                connect_url: "sqlite://storage/database/torrust_index_backend_e2e_testing.db?mode=rwc".to_string(),
                torrent_info_update_interval: 3600,
            },
            mail: Mail {
                email_verification_enabled: false,
                from: "example@email.com".to_string(),
                reply_to: "noreply@email.com".to_string(),
                username: String::new(),
                password: String::new(),
                server: "mailcatcher".to_string(),
                port: 1025,
            },
        })
        .await;

    let res: AllSettingsResponse = serde_json::from_str(&response.body).unwrap();

    assert_eq!(
        res.data,
        Settings {
            website: Website {
                name: "Torrust".to_string(),
            },
            tracker: Tracker {
                url: "udp://tracker:6969".to_string(),
                mode: "Public".to_string(),
                api_url: "http://tracker:1212".to_string(),
                token: "MyAccessToken".to_string(),
                token_valid_seconds: 7_257_600,
            },
            net: Net {
                port: 3000,
                base_url: None,
            },
            auth: Auth {
                email_on_signup: "Optional".to_string(),
                min_password_length: 6,
                max_password_length: 64,
                secret_key: "MaxVerstappenWC2021".to_string(),
            },
            database: Database {
                connect_url: "sqlite://storage/database/torrust_index_backend_e2e_testing.db?mode=rwc".to_string(),
                torrent_info_update_interval: 3600,
            },
            mail: Mail {
                email_verification_enabled: false,
                from: "example@email.com".to_string(),
                reply_to: "noreply@email.com".to_string(),
                username: String::new(),
                password: String::new(),
                server: "mailcatcher".to_string(),
                port: 1025,
            }
        }
    );
    if let Some(content_type) = &response.content_type {
        assert_eq!(content_type, "application/json");
    }
    assert_eq!(response.status, 200);
}
