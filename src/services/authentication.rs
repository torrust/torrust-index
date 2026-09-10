//! Authentication services.
//!
//! Provides login (username + password → JWT) and token renewal.
//! JWT signing and verification are delegated to [`crate::jwt::JsonWebToken`].
//!
//! ## Token revocation (ADR-T-007 Phases 4 & 7)
//!
//! On login, the service fetches the user's current
//! `token_generation` from the database and embeds it in the JWT
//! (`gen` claim). On renewal, validation is delegated to
//! [`JsonWebToken::validate_session`](crate::jwt::JsonWebToken::validate_session),
//! which verifies the JWT, checks the generation counter, and
//! rejects banned users in a single code path.
use std::sync::Arc;

use argon2::password_hash::phc::PasswordHash;
use argon2::{Argon2, PasswordVerifier};
use pbkdf2::Pbkdf2;

use super::user::DbUserProfileRepository;
use crate::config::Configuration;
use crate::databases::database::{Database, Error};
use crate::errors::AuthError;
// Re-export so that existing `use crate::services::authentication::JsonWebToken`
// paths keep compiling.
pub use crate::jwt::JsonWebToken;
use crate::models::user::{UserAuthentication, UserCompact, UserId};
use crate::services::user::Repository;
use crate::utils::clock;

pub struct Service {
    configuration: Arc<Configuration>,
    json_web_token: Arc<JsonWebToken>,
    database: Arc<Box<dyn Database>>,
    user_repository: Arc<Box<dyn Repository>>,
    user_profile_repository: Arc<DbUserProfileRepository>,
    user_authentication_repository: Arc<DbUserAuthenticationRepository>,
}

impl Service {
    pub fn new(
        configuration: Arc<Configuration>,
        json_web_token: Arc<JsonWebToken>,
        database: Arc<Box<dyn Database>>,
        user_repository: Arc<Box<dyn Repository>>,
        user_profile_repository: Arc<DbUserProfileRepository>,
        user_authentication_repository: Arc<DbUserAuthenticationRepository>,
    ) -> Self {
        Self {
            configuration,
            json_web_token,
            database,
            user_repository,
            user_profile_repository,
            user_authentication_repository,
        }
    }

    /// Authenticate user with username and password.
    /// It returns a JWT token and a compact user profile.
    ///
    /// # Errors
    ///
    /// It returns:
    ///
    /// * An `AuthError::WrongPasswordOrUsername` if unable to get user profile.
    /// * An `AuthError::InternalServerError` if unable to get user authentication data from the user id.
    /// * An `AuthError::EmailNotVerified` if the email should be, but is not verified.
    /// * An error if unable to verify the password.
    /// * An error if unable to get the user data from the database.
    pub async fn login(&self, username: &str, password: &str) -> Result<(String, UserCompact), AuthError> {
        // Get the user profile from database
        let user_profile = self
            .user_profile_repository
            .get_user_profile_from_username(username)
            .await
            .map_err(|_| AuthError::WrongPasswordOrUsername)?;

        // Should not be able to fail if user_profile succeeded
        let user_authentication = self
            .user_authentication_repository
            .get_user_authentication_from_id(&user_profile.user_id)
            .await
            .map_err(|_| AuthError::InternalServerError)?;

        verify_password(password.as_bytes(), &user_authentication).map_err(|_| AuthError::WrongPasswordOrUsername)?;

        let settings = self.configuration.settings.read().await;

        // Fail login if email verification is required and this email is not verified
        if let Some(registration) = &settings.registration
            && let Some(email) = &registration.email
            && email.verification_required
            && !user_profile.email_verified
        {
            return Err(AuthError::EmailNotVerified);
        }

        // Drop read lock on settings
        drop(settings);

        let user_compact = self
            .user_repository
            .get_compact(&user_profile.user_id)
            .await
            .map_err(|err| match err {
                Error::UserNotFound => AuthError::UserNotFound,
                err => AuthError::from(err),
            })?;

        // Fetch the current token generation for this user
        let token_generation = self.database.get_token_generation(user_compact.user_id).await?;

        // Sign JWT with compact user details as payload
        let token = self.json_web_token.sign(user_compact.clone(), token_generation).await?;

        Ok((token, user_compact))
    }

    /// Renew a supplied JWT.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * Unable to verify the supplied payload as a valid jwt.
    /// * Unable to get user data from the database.
    pub async fn renew_token(&self, token: &str) -> Result<(String, UserCompact), AuthError> {
        const ONE_WEEK_IN_SECONDS: u64 = 604_800;

        let claims = self.json_web_token.validate_session(&**self.database, token).await?;

        let user_compact = self.user_repository.get_compact(&claims.sub).await.map_err(|err| match err {
            Error::UserNotFound => AuthError::UserNotFound,
            err => AuthError::from(err),
        })?;

        // Renew token if it is valid for less than one week
        let token = match claims.exp.saturating_sub(clock::now()) {
            x if x < ONE_WEEK_IN_SECONDS => self.json_web_token.sign(user_compact.clone(), claims.token_gen).await?,
            _ => token.to_string(),
        };

        Ok((token, user_compact))
    }
}

pub struct DbUserAuthenticationRepository {
    database: Arc<Box<dyn Database>>,
}

impl DbUserAuthenticationRepository {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }

    /// Get user authentication data from user id.
    ///
    /// # Errors
    ///
    /// This function will return an error if unable to get the user
    /// authentication data from the database.
    pub async fn get_user_authentication_from_id(&self, user_id: &UserId) -> Result<UserAuthentication, Error> {
        self.database.get_user_authentication_from_id(*user_id).await
    }

    /// It changes the user's password.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn change_password(&self, user_id: UserId, password_hash: &str) -> Result<(), Error> {
        self.database.change_user_password(user_id, password_hash).await
    }

    /// Change password and increment `token_generation` atomically.
    /// See ADR-T-007 §A-2a.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn change_password_and_revoke_tokens(&self, user_id: UserId, password_hash: &str) -> Result<(), Error> {
        self.database
            .change_user_password_and_revoke_tokens(user_id, password_hash)
            .await
    }

    /// Increment the user's `token_generation` counter, invalidating all
    /// outstanding session tokens.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn increment_token_generation(&self, user_id: UserId) -> Result<(), Error> {
        self.database.increment_token_generation(user_id).await
    }
}

/// Verify if the user supplied and the database supplied passwords match
///
/// # Errors
///
/// This function will return an error if unable to parse password hash from the stored user authentication value.
/// This function will return an `AuthError::InvalidPassword` if unable to match the password with either `argon2id` or `pbkdf2-sha256`.
pub fn verify_password(password: &[u8], user_authentication: &UserAuthentication) -> Result<(), AuthError> {
    // wrap string of the hashed password into a PasswordHash struct for verification
    let parsed_hash = PasswordHash::new(&user_authentication.password_hash)?;

    match parsed_hash.algorithm.as_str() {
        "argon2id" => {
            if Argon2::default().verify_password(password, &parsed_hash).is_err() {
                return Err(AuthError::InvalidPassword);
            }

            Ok(())
        }
        "pbkdf2-sha256" => {
            if Pbkdf2::default().verify_password(password, &parsed_hash).is_err() {
                return Err(AuthError::InvalidPassword);
            }

            Ok(())
        }
        _ => Err(AuthError::InvalidPassword),
    }
}
