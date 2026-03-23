use crate::config::Configuration;
use crate::services::settings::{ConfigurationPublic, EmailOnSignup, extract_public_settings};

#[tokio::test]
async fn configuration_should_return_only_public_settings() {
    let configuration = Configuration::default();
    let all_settings = configuration.get_all().await;

    let email_on_signup = all_settings
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
        extract_public_settings(&all_settings),
        ConfigurationPublic {
            website_name: all_settings.website.name.clone(),
            tracker_url: all_settings.tracker.url,
            tracker_listed: all_settings.tracker.listed,
            tracker_private: all_settings.tracker.private,
            email_on_signup,
            website: all_settings.website.into(),
        }
    );
}
