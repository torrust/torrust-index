//! Authorization service.
use std::sync::Arc;

use casbin::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::user::Repository;
use crate::errors::ServiceError;
use crate::models::user::{UserCompact, UserId};

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum ACTION {
    AddCategory,
    DeleteCategory,
    GetSettings,
    GetSettingsSecret,
    AddTag,
    DeleteTag,
    DeleteTorrent,
    BanUser,
}

pub struct Service {
    user_repository: Arc<Box<dyn Repository>>,
    casbin_enforcer: Arc<CasbinEnforcer>,
}

impl Service {
    #[must_use]
    pub fn new(user_repository: Arc<Box<dyn Repository>>, casbin_enforcer: Arc<CasbinEnforcer>) -> Self {
        Self {
            user_repository,
            casbin_enforcer,
        }
    }

    ///Allows or denies an user to perform an action based on the user's privileges
    ///
    /// # Errors
    ///
    /// Will return an error if:
    /// - There is no user id found in the request
    /// - The user id is not found in the database
    /// - The user is not authorized to perform the action.

    pub async fn authorize(&self, action: ACTION, maybe_user_id: Option<UserId>) -> std::result::Result<(), ServiceError> {
        match maybe_user_id {
            Some(user_id) => {
                // Checks if the user found in the request exists in the database
                let user_guard = self.get_user(user_id).await?;

                //Converts the bool administrator value to a string so the enforcer can handle the request and match against the policy file
                let role = if user_guard.administrator { "admin" } else { "guest" };

                let enforcer = self.casbin_enforcer.enforcer.read().await;

                let authorize = enforcer.enforce((role, action)).map_err(|_| ServiceError::Unauthorized)?;

                if authorize {
                    Ok(())
                } else {
                    Err(ServiceError::Unauthorized)
                }
            }
            None => Err(ServiceError::Unauthorized),
        }
    }

    /// It returns the compact user.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    async fn get_user(&self, user_id: UserId) -> std::result::Result<UserCompact, ServiceError> {
        self.user_repository.get_compact(&user_id).await
    }
}

pub struct CasbinEnforcer {
    enforcer: Arc<RwLock<Enforcer>>,
}

impl CasbinEnforcer {
    /// # Panics
    ///
    /// It panics if the policy and/or model file cannot be loaded
    pub async fn new() -> Self {
        let casbin_configuration = CasbinConfiguration::new();

        let model = DefaultModel::from_str(&casbin_configuration.model).await.unwrap();

        let policy = casbin_configuration.policy;

        let mut enforcer = Enforcer::new(model, ()).await.unwrap();

        enforcer.add_policies(policy).await.unwrap();

        let enforcer = Arc::new(RwLock::new(enforcer));
        //casbin_enforcer.enable_log(true);
        Self { enforcer }
    }
}
#[allow(dead_code)]
struct CasbinConfiguration {
    model: String,
    policy: Vec<Vec<std::string::String>>,
}

impl CasbinConfiguration {
    pub fn new() -> Self {
        CasbinConfiguration {
            model: String::from(
                "
                [request_definition]
                r = role, action
                
                [policy_definition]
                p = role, action
                
                [policy_effect]
                e = some(where (p.eft == allow))
                
                [matchers]
                m = r.role == p.role && r.action == p.action
            ",
            ),
            policy: vec![
                vec!["admin".to_owned(), "AddCategory".to_owned()],
                vec!["admin".to_owned(), "DeleteCategory".to_owned()],
                vec!["admin".to_owned(), "GetSettings".to_owned()],
                vec!["admin".to_owned(), "GetSettingsSecret".to_owned()],
                vec!["admin".to_owned(), "AddTag".to_owned()],
                vec!["admin".to_owned(), "DeleteTag".to_owned()],
                vec!["admin".to_owned(), "DeleteTorrent".to_owned()],
                vec!["admin".to_owned(), "BanUser".to_owned()],
            ],
        }
    }
}
