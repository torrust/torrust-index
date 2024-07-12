//! Authorization service.
use std::fmt;
use std::sync::Arc;

use casbin::{CoreApi, DefaultModel, Enforcer, MgmtApi};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::user::Repository;
use crate::errors::ServiceError;
use crate::models::user::{UserCompact, UserId};

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
enum UserRole {
    Admin,
    Registered,
    Guest,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let role_str = match self {
            UserRole::Admin => "admin",
            UserRole::Registered => "registered",
            UserRole::Guest => "guest",
        };
        write!(f, "{role_str}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub enum ACTION {
    GetCategories,
    AddCategory,
    DeleteCategory,
    GetSettings,
    GetSettingsSecret,
    GetTags,
    AddTag,
    DeleteTag,
    DeleteTorrent,
    BanUser,
    GetAboutPage,
    GetLicensePage,
    GetImageByUrl,
    GetPublicSettings,
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
    /// - The user is not authorized to perform the action.

    pub async fn authorize(&self, action: ACTION, opt_user_id: Option<UserId>) -> std::result::Result<(), ServiceError> {
        let role = self.get_role(opt_user_id).await;

        let enforcer = self.casbin_enforcer.enforcer.read().await;

        let authorize = enforcer.enforce((role, action)).map_err(|_| ServiceError::Unauthorized)?;

        if authorize {
            Ok(())
        } else {
            Err(ServiceError::Unauthorized)
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

    /// It returns the role of the user.
    /// If the user found in the request does not exist in the database or there is no user id, a guest role is returned
    async fn get_role(&self, maybe_user_id: Option<UserId>) -> UserRole {
        match maybe_user_id {
            Some(user_id) => {
                // Checks if the user found in the request exists in the database
                let user_guard = self.get_user(user_id).await;

                match user_guard {
                    Ok(user_data) => {
                        if user_data.administrator {
                            UserRole::Admin
                        } else {
                            UserRole::Registered
                        }
                    }
                    Err(_) => UserRole::Guest,
                }
            }
            None => UserRole::Guest,
        }
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

        let model = DefaultModel::from_str(&casbin_configuration.model)
            .await
            .expect("Error loading the model");

        // Converts the policy from a string type to a vector
        let policy = casbin_configuration
            .policy
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.split(',').map(|s| s.trim().to_owned()).collect::<Vec<String>>())
            .collect();

        let mut enforcer = Enforcer::new(model, ()).await.expect("Error creating the enforcer");

        enforcer.add_policies(policy).await.expect("Error loading the policy");

        let enforcer = Arc::new(RwLock::new(enforcer));

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
                r = role, action
                
                [policy_definition]
                p = role, action
                
                [policy_effect]
                e = some(where (p.eft == allow))
                
                [matchers]
                m = r.role == p.role && r.action == p.action
            ",
            ),
            policy: String::from(
                "
                admin, AddCategory
                admin, DeleteCategory
                admin, GetPublicSettings
                admin, GetSettingsSecret
                admin, AddTag
                admin, DeleteTag
                admin, DeleteTorrent
                admin, BanUser
                admin, GetImageByUrl
                registered, GetImageByUrl
                registered, GetPublicSettings
                guest, GetCategories
                guest, GetTags
                guest, GetAboutPage
                guest, GetLicensePage
                guest, GetPublicSettings
                ",
            ),
        }
    }
}
