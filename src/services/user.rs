//! User services.
use std::sync::Arc;

use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use async_trait::async_trait;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
#[cfg(test)]
use mockall::automock;
use pbkdf2::password_hash::rand_core::OsRng;
use tracing::{debug, info};

use super::authentication::DbUserAuthenticationRepository;
use super::authorization::{self, ACTION};
use crate::config::{Configuration, EmailOnSignup, PasswordConstraints};
use crate::databases::database::{Database, Error};
use crate::errors::ServiceError;
use crate::mailer;
use crate::mailer::VerifyClaims;
use crate::models::user::{UserCompact, UserId, UserProfile, Username};
use crate::services::authentication::verify_password;
use crate::utils::validation::validate_email_address;
use crate::web::api::server::v1::contexts::user::forms::{ChangePasswordForm, RegistrationForm};

/// Since user email could be optional, we need a way to represent "no email"
/// in the database. This function returns the string that should be used for
/// that purpose.
fn no_email() -> String {
    String::new()
}

pub struct RegistrationService {
    configuration: Arc<Configuration>,
    mailer: Arc<mailer::Service>,
    user_repository: Arc<Box<dyn Repository>>,
    user_profile_repository: Arc<DbUserProfileRepository>,
}

impl RegistrationService {
    #[must_use]
    pub fn new(
        configuration: Arc<Configuration>,
        mailer: Arc<mailer::Service>,
        user_repository: Arc<Box<dyn Repository>>,
        user_profile_repository: Arc<DbUserProfileRepository>,
    ) -> Self {
        Self {
            configuration,
            mailer,
            user_repository,
            user_profile_repository,
        }
    }

    /// It registers a new user.
    ///
    /// # Errors
    ///
    /// This function will return a:
    ///
    /// * `ServiceError::EmailMissing` if email is required, but missing.
    /// * `ServiceError::EmailInvalid` if supplied email is badly formatted.
    /// * `ServiceError::PasswordsDontMatch` if the supplied passwords do not match.
    /// * `ServiceError::PasswordTooShort` if the supplied password is too short.
    /// * `ServiceError::PasswordTooLong` if the supplied password is too long.
    /// * `ServiceError::UsernameInvalid` if the supplied username is badly formatted.
    /// * `ServiceError::FailedToSendVerificationEmail` if unable to send the required verification email.
    /// * An error if unable to successfully hash the password.
    /// * An error if unable to insert user into the database.
    ///
    /// # Panics
    ///
    /// This function will panic if the email is required, but missing.
    pub async fn register_user(&self, registration_form: &RegistrationForm, api_base_url: &str) -> Result<UserId, ServiceError> {
        info!("registering user: {}", registration_form.username);

        let Ok(username) = registration_form.username.parse::<Username>() else {
            return Err(ServiceError::UsernameInvalid);
        };

        let settings = self.configuration.settings.read().await;

        let opt_email = match settings.auth.email_on_signup {
            EmailOnSignup::Required => {
                if registration_form.email.is_none() {
                    return Err(ServiceError::EmailMissing);
                }
                registration_form.email.clone()
            }
            EmailOnSignup::Ignored => None,
            EmailOnSignup::Optional => registration_form.email.clone(),
        };

        if let Some(email) = &registration_form.email {
            if !validate_email_address(email) {
                return Err(ServiceError::EmailInvalid);
            }
        }

        let password_constraints = PasswordConstraints {
            min_password_length: settings.auth.password_constraints.min_password_length,
            max_password_length: settings.auth.password_constraints.max_password_length,
        };

        validate_password_constraints(
            &registration_form.password,
            &registration_form.confirm_password,
            &password_constraints,
        )?;

        let password_hash = hash_password(&registration_form.password)?;

        let user_id = self
            .user_repository
            .add(
                &username.to_string(),
                &opt_email.clone().unwrap_or(no_email()),
                &password_hash,
            )
            .await?;

        // If this is the first created account, give administrator rights
        if user_id == 1 {
            drop(self.user_repository.grant_admin_role(&user_id).await);
        }

        if settings.mail.email_verification_enabled {
            if let Some(email) = opt_email {
                let mail_res = self
                    .mailer
                    .send_verification_mail(&email, &registration_form.username, user_id, api_base_url)
                    .await;

                if mail_res.is_err() {
                    drop(self.user_repository.delete(&user_id).await);
                    return Err(ServiceError::FailedToSendVerificationEmail);
                }
            }
        }

        Ok(user_id)
    }

    /// It verifies the email address of a user via the token sent to the
    /// user's email.
    ///
    /// # Errors
    ///
    /// This function will return a `ServiceError::DatabaseError` if unable to
    /// update the user's email verification status.
    pub async fn verify_email(&self, token: &str) -> Result<bool, ServiceError> {
        let settings = self.configuration.settings.read().await;

        let token_data = match decode::<VerifyClaims>(
            token,
            &DecodingKey::from_secret(settings.auth.secret_key.as_bytes()),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(token_data) => {
                if !token_data.claims.iss.eq("email-verification") {
                    return Ok(false);
                }

                token_data.claims
            }
            Err(_) => return Ok(false),
        };

        drop(settings);

        let user_id = token_data.sub;

        if self.user_profile_repository.verify_email(&user_id).await.is_err() {
            return Err(ServiceError::DatabaseError);
        };

        Ok(true)
    }
}

pub struct ProfileService {
    configuration: Arc<Configuration>,
    user_authentication_repository: Arc<DbUserAuthenticationRepository>,
}

impl ProfileService {
    #[must_use]
    pub fn new(configuration: Arc<Configuration>, user_repository: Arc<DbUserAuthenticationRepository>) -> Self {
        Self {
            configuration,
            user_authentication_repository: user_repository,
        }
    }

    /// It registers a new user.
    ///
    /// # Errors
    ///
    /// This function will return a:
    ///
    /// * `ServiceError::InvalidPassword` if the current password supplied is invalid.
    /// * `ServiceError::PasswordsDontMatch` if the supplied passwords do not match.
    /// * `ServiceError::PasswordTooShort` if the supplied password is too short.
    /// * `ServiceError::PasswordTooLong` if the supplied password is too long.
    /// * An error if unable to successfully hash the password.
    /// * An error if unable to change the password in the database.
    pub async fn change_password(&self, user_id: UserId, change_password_form: &ChangePasswordForm) -> Result<(), ServiceError> {
        info!("changing user password for user ID: {user_id}");

        let settings = self.configuration.settings.read().await;

        let user_authentication = self
            .user_authentication_repository
            .get_user_authentication_from_id(&user_id)
            .await?;

        verify_password(change_password_form.current_password.as_bytes(), &user_authentication)?;

        let password_constraints = PasswordConstraints {
            min_password_length: settings.auth.password_constraints.min_password_length,
            max_password_length: settings.auth.password_constraints.max_password_length,
        };

        validate_password_constraints(
            &change_password_form.password,
            &change_password_form.confirm_password,
            &password_constraints,
        )?;

        let password_hash = hash_password(&change_password_form.password)?;

        self.user_authentication_repository
            .change_password(user_id, &password_hash)
            .await?;

        Ok(())
    }
}

pub struct BanService {
    user_profile_repository: Arc<DbUserProfileRepository>,
    banned_user_list: Arc<DbBannedUserList>,
    authorization_service: Arc<authorization::Service>,
}

impl BanService {
    #[must_use]
    pub fn new(
        user_profile_repository: Arc<DbUserProfileRepository>,
        banned_user_list: Arc<DbBannedUserList>,
        authorization_service: Arc<authorization::Service>,
    ) -> Self {
        Self {
            user_profile_repository,
            banned_user_list,
            authorization_service,
        }
    }

    /// Ban a user from the Index.
    ///
    /// # Errors
    ///
    /// This function will return a:
    ///
    /// * `ServiceError::InternalServerError` if unable get user from the request.
    /// * An error if unable to get user profile from supplied username.
    /// * An error if unable to set the ban of the user in the database.
    pub async fn ban_user(&self, username_to_be_banned: &str, user_id: &UserId) -> Result<(), ServiceError> {
        debug!("user with ID {user_id} banning username: {username_to_be_banned}");

        self.authorization_service.authorize(ACTION::BanUser, Some(*user_id)).await?;

        let user_profile = self
            .user_profile_repository
            .get_user_profile_from_username(username_to_be_banned)
            .await?;

        self.banned_user_list.add(&user_profile.user_id).await?;

        Ok(())
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Repository: Sync + Send {
    async fn get_compact(&self, user_id: &UserId) -> Result<UserCompact, ServiceError>;
    async fn grant_admin_role(&self, user_id: &UserId) -> Result<(), Error>;
    async fn delete(&self, user_id: &UserId) -> Result<(), Error>;
    async fn add(&self, username: &str, email: &str, password_hash: &str) -> Result<UserId, Error>;
}

pub struct DbUserRepository {
    database: Arc<Box<dyn Database>>,
}

impl DbUserRepository {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }
}

#[async_trait]
impl Repository for DbUserRepository {
    /// It returns the compact user.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    async fn get_compact(&self, user_id: &UserId) -> Result<UserCompact, ServiceError> {
        // todo: persistence layer should have its own errors instead of
        // returning a `ServiceError`.
        self.database
            .get_user_compact_from_id(*user_id)
            .await
            .map_err(|_| ServiceError::UserNotFound)
    }

    /// It grants the admin role to the user.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    async fn grant_admin_role(&self, user_id: &UserId) -> Result<(), Error> {
        self.database.grant_admin_role(*user_id).await
    }

    /// It deletes the user.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    async fn delete(&self, user_id: &UserId) -> Result<(), Error> {
        self.database.delete_user(*user_id).await
    }

    /// It adds a new user.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    async fn add(&self, username: &str, email: &str, password_hash: &str) -> Result<UserId, Error> {
        self.database.insert_user_and_get_id(username, email, password_hash).await
    }
}

pub struct DbUserProfileRepository {
    database: Arc<Box<dyn Database>>,
}

impl DbUserProfileRepository {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }

    /// It marks the user's email as verified.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn verify_email(&self, user_id: &UserId) -> Result<(), Error> {
        self.database.verify_email(*user_id).await
    }

    /// It get the user profile from the username.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn get_user_profile_from_username(&self, username: &str) -> Result<UserProfile, Error> {
        self.database.get_user_profile_from_username(username).await
    }
}

pub struct DbBannedUserList {
    database: Arc<Box<dyn Database>>,
}

impl DbBannedUserList {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }

    /// It add a user to the banned users list.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    ///
    /// # Panics
    ///
    /// It panics if the expiration date cannot be parsed. It should never
    /// happen as the date is hardcoded for now.
    pub async fn add(&self, user_id: &UserId) -> Result<(), Error> {
        // todo: add reason and `date_expiry` parameters to request.

        // code-review: add the user ID of the user who banned the user.

        // For the time being, we will not use a reason for banning a user.
        let reason = "no reason".to_string();

        // User will be banned until the year 9999
        let date_expiry = chrono::NaiveDateTime::parse_from_str("9999-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .expect("Could not parse date from 9999-01-01 00:00:00.");

        self.database.ban_user(*user_id, &reason, date_expiry).await
    }
}

fn validate_password_constraints(
    password: &str,
    confirm_password: &str,
    password_rules: &PasswordConstraints,
) -> Result<(), ServiceError> {
    if password != confirm_password {
        return Err(ServiceError::PasswordsDontMatch);
    }

    let password_length = password.len();

    if password_length <= password_rules.min_password_length {
        return Err(ServiceError::PasswordTooShort);
    }

    if password_length >= password_rules.max_password_length {
        return Err(ServiceError::PasswordTooLong);
    }

    Ok(())
}

fn hash_password(password: &str) -> Result<String, ServiceError> {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();

    Ok(password_hash)
}
