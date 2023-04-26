use serde::{Deserialize, Serialize};

use crate::e2e::contexts::user::fixtures::logged_in_admin;
use crate::e2e::environment::TestEnv;

// Request data

pub type UpdateSettingsForm = Settings;

// Response data

#[derive(Deserialize)]
pub struct AllSettingsResponse {
    pub data: Settings,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Settings {
    pub website: Website,
    pub tracker: Tracker,
    pub net: Net,
    pub auth: Auth,
    pub database: Database,
    pub mail: Mail,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Website {
    pub name: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Tracker {
    pub url: String,
    pub mode: String,
    pub api_url: String,
    pub token: String,
    pub token_valid_seconds: u64,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Net {
    pub port: u64,
    pub base_url: Option<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Auth {
    pub email_on_signup: String,
    pub min_password_length: u64,
    pub max_password_length: u64,
    pub secret_key: String,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Database {
    pub connect_url: String,
    pub torrent_info_update_interval: u64,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Mail {
    pub email_verification_enabled: bool,
    pub from: String,
    pub reply_to: String,
    pub username: String,
    pub password: String,
    pub server: String,
    pub port: u64,
}

#[derive(Deserialize)]
pub struct PublicSettingsResponse {
    pub data: Public,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Public {
    pub website_name: String,
    pub tracker_url: String,
    pub tracker_mode: String,
    pub email_on_signup: String,
}

#[derive(Deserialize)]
pub struct SiteNameResponse {
    pub data: String,
}

#[tokio::test]
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_allow_guests_to_get_the_public_settings() {
    let client = TestEnv::default().unauthenticated_client();

    let response = client.get_public_settings().await;

    let res: PublicSettingsResponse = serde_json::from_str(&response.body).unwrap();

    assert_eq!(
        res.data,
        Public {
            website_name: "Torrust".to_string(),
            tracker_url: "udp://tracker:6969".to_string(),
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
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_allow_guests_to_get_the_site_name() {
    let client = TestEnv::default().unauthenticated_client();

    let response = client.get_site_name().await;

    let res: SiteNameResponse = serde_json::from_str(&response.body).unwrap();

    assert_eq!(res.data, "Torrust");
    if let Some(content_type) = &response.content_type {
        assert_eq!(content_type, "application/json");
    }
    assert_eq!(response.status, 200);
}

#[tokio::test]
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_allow_admins_to_get_all_the_settings() {
    let logged_in_admin = logged_in_admin().await;
    let client = TestEnv::default().authenticated_client(&logged_in_admin.token);

    let response = client.get_settings().await;

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

#[tokio::test]
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_allow_admins_to_update_all_the_settings() {
    let logged_in_admin = logged_in_admin().await;
    let client = TestEnv::default().authenticated_client(&logged_in_admin.token);

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
