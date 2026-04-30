//! API contract for `settings` context.

use torrust_index::services::settings::EmailOnSignup;
use torrust_index::web::api;

use crate::common::asserts::assert_json_ok_response;
use crate::common::client::Client;
use crate::common::contexts::settings::responses::{AllSettingsResponse, Public, PublicSettingsResponse, SiteNameResponse};
use crate::e2e::environment::TestEnv;
use crate::e2e::web::api::v1::contexts::user::steps::new_logged_in_admin;

#[tokio::test]
async fn it_should_allow_guests_to_get_the_public_settings() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    let response = client.get_public_settings().await;

    let res: PublicSettingsResponse = serde_json::from_str(&response.body)
        .unwrap_or_else(|_| panic!("response {:#?} should be a PublicSettingsResponse", response.body));

    let email_on_signup =
        env.server_settings()
            .unwrap()
            .registration
            .as_ref()
            .map_or(EmailOnSignup::NotIncluded, |registration| {
                registration.email.as_ref().map_or(EmailOnSignup::NotIncluded, |email| {
                    if email.required {
                        EmailOnSignup::Required
                    } else {
                        EmailOnSignup::Optional
                    }
                })
            });

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

#[tokio::test]
async fn it_should_allow_admins_to_get_all_the_settings() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let logged_in_admin = new_logged_in_admin(&env).await;
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

    let response = client.get_settings().await;

    let mut actual: AllSettingsResponse = serde_json::from_str(&response.body).unwrap();
    let mut expected = env.server_settings_masking_secrets().unwrap();

    // Normalise environment-specific fields that legitimately differ
    // between the host-side loader (which builds `expected` from the
    // shipped TOML plus the host's env overrides) and the container's
    // effective configuration (which the entry script may augment —
    // e.g. defaulting `auth.{private,public}_key_path` to
    // `/etc/torrust/index/auth/{private,public}.pem` when neither
    // PEM nor path is supplied; ADR-T-009 §7). The DB connect URL
    // similarly differs because the container path
    // (`/var/lib/torrust/index/database/...`) is the bind-mount
    // target of the host path the test runner uses
    // (`./storage/index/lib/database/...`).
    actual.data.auth.private_key_path = None;
    actual.data.auth.public_key_path = None;
    actual.data.database.connect_url.clear();
    expected.auth.private_key_path = None;
    expected.auth.public_key_path = None;
    expected.database.connect_url.clear();

    assert_eq!(actual.data, expected);

    assert_json_ok_response(&response);
}
