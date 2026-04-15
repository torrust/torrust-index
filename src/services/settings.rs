//! Settings service.
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use url::Url;

use super::authorization::{self, Action};
use crate::config::{self, Configuration, Settings};
use crate::errors::AuthError;
use crate::models::user::UserId;

pub struct Service {
    configuration: Arc<Configuration>,
    authorization_service: Arc<authorization::Service>,
}

impl Service {
    #[must_use]
    pub const fn new(configuration: Arc<Configuration>, authorization_service: Arc<authorization::Service>) -> Self {
        Self {
            configuration,
            authorization_service,
        }
    }

    /// It gets all the settings.
    ///
    /// # Errors
    ///
    /// It returns an error if the user does not have the required permissions.
    pub async fn get_all(&self, maybe_user_id: Option<UserId>) -> Result<Settings, AuthError> {
        self.authorization_service
            .authorize(Action::GetSettings, maybe_user_id)
            .await?;

        let torrust_index_configuration = self.configuration.get_all().await;

        Ok(torrust_index_configuration)
    }

    /// It gets all the settings making the secrets with asterisks.
    ///
    /// # Errors
    ///
    /// It returns an error if the user does not have the required permissions.
    pub async fn get_all_masking_secrets(&self, maybe_user_id: Option<UserId>) -> Result<Settings, AuthError> {
        self.authorization_service
            .authorize(Action::GetSettingsSecret, maybe_user_id)
            .await?;

        let mut torrust_index_configuration = self.configuration.get_all().await;

        torrust_index_configuration.remove_secrets();

        Ok(torrust_index_configuration)
    }

    /// It gets only the public settings.
    ///
    /// # Errors
    ///
    /// It returns an error if the user does not have the required permissions.
    pub async fn get_public(&self, maybe_user_id: Option<UserId>) -> Result<ConfigurationPublic, AuthError> {
        self.authorization_service
            .authorize(Action::GetPublicSettings, maybe_user_id)
            .await?;

        let settings_lock = self.configuration.get_all().await;
        Ok(extract_public_settings(&settings_lock))
    }

    /// It gets the site name from the settings.
    ///
    /// # Errors
    ///
    /// It returns an error if the user does not have the required permissions.
    pub async fn get_site_name(&self, maybe_user_id: Option<UserId>) -> Result<String, AuthError> {
        self.authorization_service
            .authorize(Action::GetSiteName, maybe_user_id)
            .await?;

        Ok(self.configuration.get_site_name().await)
    }
}

pub(crate) fn extract_public_settings(settings: &Settings) -> ConfigurationPublic {
    let email_on_signup = settings
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

    ConfigurationPublic {
        website_name: settings.website.name.clone(),
        tracker_url: settings.tracker.url.clone(),
        tracker_listed: settings.tracker.listed,
        tracker_private: settings.tracker.private,
        email_on_signup,
        website: settings.website.clone().into(),
    }
}

/// The public index configuration.
/// There is an endpoint to get this configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigurationPublic {
    pub(crate) website_name: String,
    pub(crate) tracker_url: Url,
    pub(crate) tracker_listed: bool,
    pub(crate) tracker_private: bool,
    pub(crate) email_on_signup: EmailOnSignup,
    pub(crate) website: Website,
}

/// Whether the email is required on signup or not.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum EmailOnSignup {
    /// The email is required on signup.
    Required,
    /// The email is optional on signup.
    #[default]
    Optional,
    /// The email is not allowed on signup. It will only be ignored if provided.
    NotIncluded,
}

impl fmt::Display for EmailOnSignup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_str = match self {
            Self::Required => "required",
            Self::Optional => "optional",
            Self::NotIncluded => "ignored",
        };
        write!(f, "{display_str}")
    }
}

impl FromStr for EmailOnSignup {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "required" => Ok(Self::Required),
            "optional" => Ok(Self::Optional),
            "none" => Ok(Self::NotIncluded),
            _ => Err(format!(
                "Unknown config 'email_on_signup' option (required, optional, none): {s}"
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Website {
    pub name: String,
    pub demo: Option<Demo>,
    pub terms: Terms,
}

impl From<config::Website> for Website {
    fn from(website: config::Website) -> Self {
        Self {
            name: website.name,
            demo: website.demo.map(std::convert::Into::into),
            terms: website.terms.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Demo {
    pub warning: String,
}

impl From<config::Demo> for Demo {
    fn from(demo: config::Demo) -> Self {
        Self { warning: demo.warning }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Terms {
    pub page: TermsPage,
    pub upload: TermsUpload,
}

impl From<config::Terms> for Terms {
    fn from(terms: config::Terms) -> Self {
        Self {
            page: terms.page.into(),
            upload: terms.upload.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TermsPage {
    pub title: String,
    pub content: Markdown,
}

impl From<config::TermsPage> for TermsPage {
    fn from(terms_page: config::TermsPage) -> Self {
        Self {
            title: terms_page.title,
            content: terms_page.content.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TermsUpload {
    pub content_upload_agreement: Markdown,
}

impl From<config::TermsUpload> for TermsUpload {
    fn from(terms_upload: config::TermsUpload) -> Self {
        Self {
            content_upload_agreement: terms_upload.content_upload_agreement.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Markdown(pub String);

impl Markdown {
    fn new(content: &str) -> Self {
        Self(content.to_owned())
    }
}

impl From<config::Markdown> for Markdown {
    fn from(markdown: config::Markdown) -> Self {
        Self::new(&markdown.source())
    }
}
