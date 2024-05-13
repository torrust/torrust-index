// API authorization

/* use std::sync::Arc;

use crate::errors::ServiceError;
use crate::models::user::UserId;
use crate::services::authorization::Service;

pub struct Authorization {
    authorization_service: Arc<Service>,
}

impl Authorization {
    #[must_use]
    pub fn new(authorization_service: Arc<Service>) -> Self {
        Self { authorization_service }
    }

    // pub async authorize_guest_user
    // pub async authorize_registered_user
    pub async fn authorize_admin_user(user_id: &UserId) -> Result<UserId, ServiceError::Unauthorized> {
        match Authorization::new(Arc::<Service>)
            .user_authorization_repository
            .get_user_authorization_from_id()
        {
            Ok(user_authorization) => {}
            Err(_) => {}
        }
    }
}
 */
