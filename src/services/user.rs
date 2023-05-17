//! User repository.
use std::sync::Arc;

use crate::databases::database::Database;
use crate::errors::ServiceError;
use crate::models::user::{UserCompact, UserId};

pub struct DbUserRepository {
    database: Arc<Box<dyn Database>>,
}

impl DbUserRepository {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }

    /// It returns the compact user.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn get_compact_user(&self, user_id: &UserId) -> Result<UserCompact, ServiceError> {
        // todo: persistence layer should have its own errors instead of
        // returning a `ServiceError`.
        self.database
            .get_user_compact_from_id(*user_id)
            .await
            .map_err(|_| ServiceError::UserNotFound)
    }
}
