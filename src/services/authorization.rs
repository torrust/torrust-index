use std::sync::Arc;

use crate::databases::database::{Database, Error};
use crate::errors::ServiceError;
use crate::models::user::{UserAuthorization, UserId};

pub struct AuthorizeService {
    user_authorization_repository: Arc<DbUserAuthorizationRepository>,
}

impl AuthorizeService {
    #[must_use]
    pub fn new(user_authorization_repository: Arc<DbUserAuthorizationRepository>) -> Self {
        Self {
            user_authorization_repository,
        }
    }

    /// Checks if the user has the right privileges to perform the requested action.
    ///
    /// # Errors
    ///
    /// This function will return an error if unable to get the user
    /// authorization data from the database or if the user
    /// does not have the right privileges to perform the action.
    pub async fn authorize_user(&self, user_id: UserId, admin_required: bool) -> Result<(), ServiceError> {
        // Checks if the user exists in the database
        let authorization_info = self
            .user_authorization_repository
            .get_user_authorization_from_id(&user_id)
            .await?;

        //If admin privilages are required, it checks if the user is an admin
        if admin_required {
            Self::authorize_admin_user(&authorization_info)
        } else {
            Ok(())
        }
    }

    fn authorize_admin_user(user_authorization_info: &UserAuthorization) -> Result<(), ServiceError> {
        if user_authorization_info.administrator {
            Ok(())
        } else {
            Err(ServiceError::Unauthorized)
        }
    }
}

pub struct DbUserAuthorizationRepository {
    database: Arc<Box<dyn Database>>,
}

impl DbUserAuthorizationRepository {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }

    /// Get user authorization data from user id.
    ///
    /// # Errors
    ///
    /// This function will return an error if unable to get the user
    /// authorization data from the database.
    pub async fn get_user_authorization_from_id(&self, user_id: &UserId) -> Result<UserAuthorization, Error> {
        self.database.get_user_authorization_from_id(*user_id).await
    }
}
