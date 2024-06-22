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
                // Checks if the user found in the requests exists in the database
                let user_guard = self.get_user(user_id).await?;

                //Converts the bool administrator value to a string so the enforcer can handle the request and match against the policy file
                let role = match user_guard.administrator {
                    true => "admin",
                    false => "guest",
                };

                // The role of the user that wants to access a resource.
                let sub = role;

                // The operation that the user wants to perform
                let act = action;

                let enforcer = self.casbin_enforcer.enforcer.read().await;

                let authorize = enforcer.enforce((sub, act)).map_err(|_| ServiceError::Unauthorized)?;

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
    /// It panics if the policy and/or model file cannot be loaded or are missing
    ///
    /// todo: review error handling and panics when creating the model, policy and enforcer
    pub async fn new() -> Self {
        let casbin_configuration = CasbinConfiguration::new();

        let model = DefaultModel::from_str(&casbin_configuration.model).await.unwrap();

        let enforcer = Enforcer::new(model, "casbin/policy.csv").await.unwrap();

        let enforcer = Arc::new(RwLock::new(enforcer));
        //casbin_enforcer.enable_log(true);
        Self { enforcer }
    }
}
#[allow(dead_code)]
struct CasbinConfiguration {
    model: String,
    policy: String,
}

impl CasbinConfiguration {
    pub fn new() -> Self {
        CasbinConfiguration {
            model: String::from(
                "
                [request_definition]
                r = sub, act
                
                [policy_definition]
                p = sub, act
                
                [policy_effect]
                e = some(where (p.eft == allow))
                
                [matchers]
                m = r.sub == p.sub && r.act == p.act
            ",
            ),
            policy: String::from(
                "
                # Admin user policies
                p, admin, AddCategory
                p, admin, DeleteCategory
                p, admin, GetSettings
                p, admin, GetSettingsSecret
                p, admin, AddTag
                p, admin, DeleteTag
                p, admin, DeleteTorrent
                p, admin, BanUser),
                
                ",
            ),
        }
    }
}
