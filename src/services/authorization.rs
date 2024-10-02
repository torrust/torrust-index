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
    GetAboutPage,
    GetLicensePage,
    AddCategory,
    DeleteCategory,
    GetCategories,
    GetImageByUrl,
    GetSettings,
    GetSettingsSecret,
    GetPublicSettings,
    GetSiteName,
    AddTag,
    DeleteTag,
    GetTags,
    AddTorrent,
    GetTorrent,
    DeleteTorrent,
    GetTorrentInfo,
    GenerateTorrentInfoListing,
    GetCanonicalInfoHash,
    ChangePassword,
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
    /// - The user is not authorized to perform the action.
    pub async fn authorize(&self, action: ACTION, maybe_user_id: Option<UserId>) -> std::result::Result<(), ServiceError> {
        let role = self.get_role(maybe_user_id).await;

        let enforcer = self.casbin_enforcer.enforcer.read().await;

        let authorize = enforcer
            .enforce((&role, action))
            .map_err(|_| ServiceError::UnauthorizedAction)?;

        if authorize {
            Ok(())
        } else if role == UserRole::Guest {
            Err(ServiceError::UnauthorizedActionForGuests)
        } else {
            Err(ServiceError::UnauthorizedAction)
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
    /// Will panic if:
    ///
    /// - The enforcer can't be created.
    /// - The policies can't be loaded.
    pub async fn with_default_configuration() -> Self {
        let casbin_configuration = CasbinConfiguration::default();

        let mut enforcer = Enforcer::new(casbin_configuration.default_model().await, ())
            .await
            .expect("Error creating the enforcer");

        enforcer
            .add_policies(casbin_configuration.policy_lines())
            .await
            .expect("Error loading the policy");

        let enforcer = Arc::new(RwLock::new(enforcer));

        Self { enforcer }
    }

    /// # Panics
    ///
    /// Will panic if:
    ///
    /// - The enforcer can't be created.
    /// - The policies can't be loaded.
    pub async fn with_configuration(casbin_configuration: CasbinConfiguration) -> Self {
        let mut enforcer = Enforcer::new(casbin_configuration.default_model().await, ())
            .await
            .expect("Error creating the enforcer");

        enforcer
            .add_policies(casbin_configuration.policy_lines())
            .await
            .expect("Error loading the policy");

        let enforcer = Arc::new(RwLock::new(enforcer));

        Self { enforcer }
    }
}

#[allow(dead_code)]
pub struct CasbinConfiguration {
    model: String,
    policy: String,
}

impl CasbinConfiguration {
    #[must_use]
    pub fn new(model: &str, policy: &str) -> Self {
        Self {
            model: model.to_owned(),
            policy: policy.to_owned(),
        }
    }

    /// # Panics
    ///
    /// It panics if the model cannot be loaded.
    async fn default_model(&self) -> DefaultModel {
        DefaultModel::from_str(&self.model).await.expect("Error loading the model")
    }

    /// Converts the policy from a string type to a vector.
    fn policy_lines(&self) -> Vec<Vec<String>> {
        self.policy
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.split(',').map(|s| s.trim().to_owned()).collect::<Vec<String>>())
            .collect()
    }
}

impl Default for CasbinConfiguration {
    fn default() -> Self {
        Self {
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
                admin, GetAboutPage
                admin, GetLicensePage
                admin, AddCategory
                admin, DeleteCategory
                admin, GetCategories
                admin, GetImageByUrl
                admin, GetSettings
                admin, GetSettingsSecret
                admin, GetPublicSettings
                admin, GetSiteName
                admin, AddTag
                admin, DeleteTag
                admin, GetTags
                admin, AddTorrent
                admin, GetTorrent
                admin, DeleteTorrent
                admin, GetTorrentInfo
                admin, GenerateTorrentInfoListing
                admin, GetCanonicalInfoHash
                admin, ChangePassword
                admin, BanUser
                registered, GetAboutPage
                registered, GetLicensePage
                registered, GetCategories
                registered, GetImageByUrl
                registered, GetPublicSettings
                registered, GetSiteName
                registered, GetTags
                registered, AddTorrent
                registered, GetTorrent
                registered, GetTorrentInfo
                registered, GenerateTorrentInfoListing
                registered, GetCanonicalInfoHash
                registered, ChangePassword
                guest, GetAboutPage
                guest, GetLicensePage
                guest, GetCategories
                guest, GetPublicSettings
                guest, GetSiteName
                guest, GetTags
                guest, GetTorrent
                guest, GetTorrentInfo
                guest, GenerateTorrentInfoListing
                guest, GetCanonicalInfoHash
                ",
            ),
        }
    }
}
