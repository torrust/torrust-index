//! Settings service.
use std::sync::Arc;

use super::authorization::{self, ACTION};
use crate::config::{Configuration, ConfigurationPublic, Settings};
use crate::errors::ServiceError;
use crate::models::user::UserId;

pub struct Service {
    configuration: Arc<Configuration>,
    authorization_service: Arc<authorization::Service>,
}

impl Service {
    #[must_use]
    pub fn new(configuration: Arc<Configuration>, authorization_service: Arc<authorization::Service>) -> Service {
        Service {
            configuration,
            authorization_service,
        }
    }

    /// It gets all the settings.
    ///
    /// # Errors
    ///
    /// It returns an error if the user does not have the required permissions.
    pub async fn get_all(&self, user_id: &UserId) -> Result<Settings, ServiceError> {
        self.authorization_service
            .authorize(ACTION::GetSettings, Some(*user_id))
            .await?;

        let torrust_index_configuration = self.configuration.get_all().await;

        Ok(torrust_index_configuration)
    }

    /// It gets all the settings making the secrets with asterisks.
    ///
    /// # Errors
    ///
    /// It returns an error if the user does not have the required permissions.
    pub async fn get_all_masking_secrets(&self, user_id: &UserId) -> Result<Settings, ServiceError> {
        self.authorization_service
            .authorize(ACTION::GetSettingsSecret, Some(*user_id))
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
    pub async fn get_public(&self) -> ConfigurationPublic {
        self.configuration.get_public().await
    }

    /// It gets the site name from the settings.
    ///
    /// # Errors
    ///
    /// It returns an error if the user does not have the required permissions.
    pub async fn get_site_name(&self) -> String {
        self.configuration.get_site_name().await
    }
}
