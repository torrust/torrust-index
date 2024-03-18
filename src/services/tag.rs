//! Tag service.
use std::sync::Arc;

use super::authorization::AuthorizationService;
use crate::databases::database::{Database, Error as DatabaseError, Error};
use crate::errors::ServiceError;
use crate::models::torrent_tag::{TagId, TorrentTag};
use crate::models::user::UserId;

pub struct Service {
    tag_repository: Arc<DbTagRepository>,
    authorization_service: Arc<AuthorizationService>,
}

impl Service {
    #[must_use]
    pub fn new(tag_repository: Arc<DbTagRepository>, authorization_service: Arc<AuthorizationService>) -> Service {
        Service {
            tag_repository,
            authorization_service,
        }
    }

    /// Adds a new tag.
    ///
    /// # Errors
    ///
    /// It returns an error if:
    ///
    /// * The user does not have the required permissions.
    /// * There is a database error.
    pub async fn add_tag(&self, tag_name: &str, user_id: &UserId) -> Result<TagId, ServiceError> {
        self.authorization_service.authorize_user(*user_id, true).await?;

        let trimmed_name = tag_name.trim();

        if trimmed_name.is_empty() {
            return Err(ServiceError::TagNameEmpty);
        }

        match self.tag_repository.add(trimmed_name).await {
            Ok(id) => Ok(id),
            Err(e) => match e {
                DatabaseError::TagAlreadyExists => Err(ServiceError::TagAlreadyExists),
                _ => Err(ServiceError::DatabaseError),
            },
        }
    }

    /// Deletes a tag.
    ///
    /// # Errors
    ///
    /// It returns an error if:
    ///
    /// * The user does not have the required permissions.
    /// * There is a database error.
    pub async fn delete_tag(&self, tag_id: &TagId, user_id: &UserId) -> Result<(), ServiceError> {
        self.authorization_service.authorize_user(*user_id, true).await?;

        match self.tag_repository.delete(tag_id).await {
            Ok(()) => Ok(()),
            Err(e) => match e {
                DatabaseError::TagNotFound => Err(ServiceError::TagNotFound),
                _ => Err(ServiceError::DatabaseError),
            },
        }
    }
}

pub struct DbTagRepository {
    database: Arc<Box<dyn Database>>,
}

impl DbTagRepository {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }

    /// It adds a new tag and returns the newly created tag.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn add(&self, tag_name: &str) -> Result<TagId, Error> {
        self.database.insert_tag_and_get_id(tag_name).await
    }

    /// It returns all the tags.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn get_all(&self) -> Result<Vec<TorrentTag>, Error> {
        self.database.get_tags().await
    }

    /// It removes a tag and returns it.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn delete(&self, tag_id: &TagId) -> Result<(), Error> {
        self.database.delete_tag(*tag_id).await
    }
}
