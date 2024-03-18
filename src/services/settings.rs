//! Settings service.
use std::sync::Arc;

use super::authorization::AuthorizationService;
use crate::config::{Configuration, ConfigurationPublic, TorrustIndex};
use crate::errors::ServiceError;
use crate::models::user::UserId;

pub struct Service {
    configuration: Arc<Configuration>,
    authorization_service: Arc<AuthorizationService>,
}

impl Service {
    #[must_use]
    pub fn new(configuration: Arc<Configuration>, authorization_service: Arc<AuthorizationService>) -> Service {
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
    pub async fn get_all(&self, user_id: &UserId) -> Result<TorrustIndex, ServiceError> {
        self.authorization_service.authorize_user(*user_id, true).await?;

        let torrust_index_configuration = self.configuration.get_all().await;

        Ok(torrust_index_configuration)
    }

    /// It gets all the settings making the secrets with asterisks.
    ///
    /// # Errors
    ///
    /// It returns an error if the user does not have the required permissions.
    pub async fn get_all_masking_secrets(&self, user_id: &UserId) -> Result<TorrustIndex, ServiceError> {
        self.authorization_service.authorize_user(*user_id, true).await?;

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
