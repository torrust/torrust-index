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
#[allow(unused_imports)]
#[cfg(test)]
mod test {
    use std::str::FromStr;
    use std::sync::Arc;

    use mockall::predicate;

    use crate::databases::database;
    use crate::errors::ServiceError;
    use crate::models::user::{User, UserCompact};
    use crate::services::authorization::{Service, ACTION};
    use crate::services::user::{MockRepository, Repository};
    use crate::web::api::client::v1::random::string;

    #[tokio::test]
    async fn a_guest_user_should_not_be_able_to_add_categories() {
        let test_user_id = 1;

        let mut mock_repository = MockRepository::new();
        mock_repository
            .expect_get_compact()
            .with(predicate::eq(test_user_id))
            .times(1)
            .returning(|_| Err(ServiceError::UserNotFound));

        let service = Service::new(Arc::new(Box::new(mock_repository)));
        assert_eq!(
            service.authorize(ACTION::AddCategory, Some(test_user_id)).await,
            Err(ServiceError::UserNotFound)
        );
    }

    #[tokio::test]
    async fn a_registered_user_should_not_be_able_to_add_categories() {
        let test_user_id = 2;

        let mut mock_repository = MockRepository::new();
        mock_repository
            .expect_get_compact()
            .with(predicate::eq(test_user_id))
            .times(1)
            .returning(move |_| {
                Ok(UserCompact {
                    user_id: test_user_id,
                    username: "non_admin_user".to_string(),
                    administrator: false,
                })
            });

        let service = Service::new(Arc::new(Box::new(mock_repository)));
        assert_eq!(
            service.authorize(ACTION::AddCategory, Some(test_user_id)).await,
            Err(ServiceError::Unauthorized)
        );
    }

    #[tokio::test]
    async fn an_admin_user_should_be_able_to_add_categories() {
        let test_user_id = 3;

        let mut mock_repository = MockRepository::new();
        mock_repository
            .expect_get_compact()
            .with(predicate::eq(test_user_id))
            .times(1)
            .returning(move |_| {
                Ok(UserCompact {
                    user_id: test_user_id,
                    username: "admin_user".to_string(),
                    administrator: true,
                })
            });

        let service = Service::new(Arc::new(Box::new(mock_repository)));
        assert_eq!(service.authorize(ACTION::AddCategory, Some(test_user_id)).await, Ok(()));
    }

    #[tokio::test]
    async fn a_guest_user_should_not_be_able_to_delete_categories() {
        let test_user_id = 4;

        let mut mock_repository = MockRepository::new();
        mock_repository
            .expect_get_compact()
            .with(predicate::eq(test_user_id))
            .times(1)
            .returning(|_| Err(ServiceError::UserNotFound));

        let service = Service::new(Arc::new(Box::new(mock_repository)));
        assert_eq!(
            service.authorize(ACTION::DeleteCategory, Some(test_user_id)).await,
            Err(ServiceError::UserNotFound)
        );
    }

    #[tokio::test]
    async fn a_registered_user_should_not_be_able_to_delete_categories() {
        let test_user_id = 5;

        let mut mock_repository = MockRepository::new();
        mock_repository
            .expect_get_compact()
            .with(predicate::eq(test_user_id))
            .times(1)
            .returning(move |_| {
                Ok(UserCompact {
                    user_id: test_user_id,
                    username: "non_admin_user".to_string(),
                    administrator: false,
                })
            });

        let service = Service::new(Arc::new(Box::new(mock_repository)));
        assert_eq!(
            service.authorize(ACTION::DeleteCategory, Some(test_user_id)).await,
            Err(ServiceError::Unauthorized)
        );
    }

    #[tokio::test]
    async fn an_admin_user_should_be_able_to_delete_categories() {
        let test_user_id = 6;

        let mut mock_repository = MockRepository::new();
        mock_repository
            .expect_get_compact()
            .with(predicate::eq(test_user_id))
            .times(1)
            .returning(move |_| {
                Ok(UserCompact {
                    user_id: test_user_id,
                    username: "admin_user".to_string(),
                    administrator: true,
                })
            });

        let service = Service::new(Arc::new(Box::new(mock_repository)));
        assert_eq!(service.authorize(ACTION::DeleteCategory, Some(test_user_id)).await, Ok(()));
    }
}
