/*


 let user_compact = self.user_repository.get_compact(&user_profile.user_id).await?;


*/
use std::sync::Arc;

use crate::databases::database::{Database, Error};
use crate::models::user::{UserAuthorization, UserId};
use crate::services::user::DbUserRepository;
pub struct Service {
    user_repository: Arc<DbUserRepository>,
}

impl Service {
    pub fn new(user_repository: Arc<DbUserRepository>) -> Self {
        Self { user_repository }
    }

    // Check user exists in database
    /* pub async fn user_exists_in_database(&self, user_id: &UserId) ->  {
        let user_authorization = self.
    } */

    // Check if the user has a role with enough privilages

    //Delete token from localStorage if user does not exist - FRONTEND
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
