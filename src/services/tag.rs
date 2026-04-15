//! Tag service.
use std::sync::Arc;

use crate::databases::database::{Database, Error as DatabaseError, Error};
use crate::errors::CategoryTagError;
use crate::models::torrent_tag::{TagId, TorrentTag};

pub struct Service {
    tag_repository: Arc<DbTagRepository>,
}

impl Service {
    #[must_use]
    pub const fn new(tag_repository: Arc<DbTagRepository>) -> Self {
        Self {
            tag_repository,
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
    pub async fn add_tag(&self, tag_name: &str) -> Result<TagId, CategoryTagError> {
        let trimmed_name = tag_name.trim();

        if trimmed_name.is_empty() {
            return Err(CategoryTagError::TagNameEmpty);
        }

        match self.tag_repository.add(trimmed_name).await {
            Ok(id) => Ok(id),
            Err(e) => match e {
                DatabaseError::TagAlreadyExists => Err(CategoryTagError::TagAlreadyExists),
                _ => Err(CategoryTagError::DatabaseError),
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
    pub async fn delete_tag(&self, tag_id: &TagId) -> Result<(), CategoryTagError> {
        match self.tag_repository.delete(tag_id).await {
            Ok(()) => Ok(()),
            Err(e) => match e {
                DatabaseError::TagNotFound => Err(CategoryTagError::TagNotFound),
                _ => Err(CategoryTagError::DatabaseError),
            },
        }
    }

    /// Returns all the tags from the database
    ///
    /// # Errors
    ///
    /// It returns an error if:
    ///
    /// * The user does not have the required permissions.
    /// * There is a database error retrieving the tags.
    pub async fn get_tags(&self) -> Result<Vec<TorrentTag>, CategoryTagError> {
        self.tag_repository
            .get_all()
            .await
            .map_err(|_| CategoryTagError::DatabaseError)
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
