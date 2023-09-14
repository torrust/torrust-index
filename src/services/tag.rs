//! Tag service.
use std::sync::Arc;

use super::user::DbUserRepository;
use crate::databases::database::{Database, Error as DatabaseError, Error};
use crate::errors::ServiceError;
use crate::models::torrent_tag::{TagId, TorrentTag};
use crate::models::user::UserId;

pub struct Service {
    tag_repository: Arc<DbTagRepository>,
    user_repository: Arc<DbUserRepository>,
}

impl Service {
    #[must_use]
    pub fn new(tag_repository: Arc<DbTagRepository>, user_repository: Arc<DbUserRepository>) -> Service {
        Service {
            tag_repository,
            user_repository,
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
    pub async fn add_tag(&self, tag_name: &str, user_id: &UserId) -> Result<(), ServiceError> {
        let user = self.user_repository.get_compact(user_id).await?;

        // Check if user is administrator
        // todo: extract authorization service
        if !user.administrator {
            return Err(ServiceError::Unauthorized);
        }

        let trimmed_name = tag_name.trim();

        if trimmed_name.is_empty() {
            return Err(ServiceError::TagNameEmpty);
        }

        match self.tag_repository.add(trimmed_name).await {
            Ok(()) => Ok(()),
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
        let user = self.user_repository.get_compact(user_id).await?;

        // Check if user is administrator
        // todo: extract authorization service
        if !user.administrator {
            return Err(ServiceError::Unauthorized);
        }

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
    pub async fn add(&self, tag_name: &str) -> Result<(), Error> {
        self.database.add_tag(tag_name).await
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
