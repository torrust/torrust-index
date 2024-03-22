//! Authorization service.
use std::sync::Arc;

use super::user::Repository;
use crate::errors::ServiceError;
use crate::models::user::{UserCompact, UserId};

pub enum ACTION {
    AddCategory,
    DeleteCategory,
}

pub struct Service {
    user_repository: Arc<Box<dyn Repository>>,
}

impl Service {
    #[must_use]
    pub fn new(user_repository: Arc<Box<dyn Repository>>) -> Self {
        Self { user_repository }
    }

    /// # Errors
    ///
    /// Will return an error if:
    ///
    /// - There is not any user with the provided `UserId` (when the user id is some).
    /// - The user is not authorized to perform the action.
    pub async fn authorize(&self, action: ACTION, maybe_user_id: Option<UserId>) -> Result<(), ServiceError> {
        match action {
            ACTION::AddCategory | ACTION::DeleteCategory => match maybe_user_id {
                Some(user_id) => {
                    let user = self.get_user(user_id).await?;

                    if !user.administrator {
                        return Err(ServiceError::Unauthorized);
                    }

                    Ok(())
                }
                None => Err(ServiceError::Unauthorized),
            },
        }
    }

    async fn get_user(&self, user_id: UserId) -> Result<UserCompact, ServiceError> {
        self.user_repository.get_compact(&user_id).await
    }
}

#[cfg(test)]
mod tests {}
