//! Authentication services.
use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use pbkdf2::Pbkdf2;

use super::user::DbUserProfileRepository;
use crate::config::Configuration;
use crate::databases::database::{Database, Error};
use crate::errors::AuthError;
use crate::models::user::{UserAuthentication, UserClaims, UserCompact, UserId};
use crate::services::user::Repository;
use crate::utils::clock;

pub struct Service {
    configuration: Arc<Configuration>,
    json_web_token: Arc<JsonWebToken>,
    user_repository: Arc<Box<dyn Repository>>,
    user_profile_repository: Arc<DbUserProfileRepository>,
    user_authentication_repository: Arc<DbUserAuthenticationRepository>,
}

impl Service {
    pub fn new(
        configuration: Arc<Configuration>,
        json_web_token: Arc<JsonWebToken>,
        user_repository: Arc<Box<dyn Repository>>,
        user_profile_repository: Arc<DbUserProfileRepository>,
        user_authentication_repository: Arc<DbUserAuthenticationRepository>,
    ) -> Self {
        Self {
            configuration,
            json_web_token,
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
        if let Some(registration) = &settings.registration {
            if let Some(email) = &registration.email {
                if email.verification_required && !user_profile.email_verified {
                    return Err(AuthError::EmailNotVerified);
                }
            }
        }

        // Drop read lock on settings
        drop(settings);

        let user_compact = self
            .user_repository
            .get_compact(&user_profile.user_id)
            .await
            .map_err(|_| AuthError::UserNotFound)?;

        // Sign JWT with compact user details as payload
        let token = self.json_web_token.sign(user_compact.clone()).await;

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

        // Verify if token is valid
        let claims = self.json_web_token.verify(token).await?;

        let user_compact = self
            .user_repository
            .get_compact(&claims.user.user_id)
            .await
            .map_err(|_| AuthError::UserNotFound)?;

        // Renew token if it is valid for less than one week
        let token = match claims.exp - clock::now() {
            x if x < ONE_WEEK_IN_SECONDS => self.json_web_token.sign(user_compact.clone()).await,
            _ => token.to_string(),
        };

        Ok((token, user_compact))
    }
}

pub struct JsonWebToken {
    cfg: Arc<Configuration>,
}

impl JsonWebToken {
    pub const fn new(cfg: Arc<Configuration>) -> Self {
        Self { cfg }
    }

    /// Create Json Web Token.
    ///
    /// # Panics
    ///
    /// This function will panic if the default encoding algorithm does not ç
    /// match the encoding key.
    pub async fn sign(&self, user: UserCompact) -> String {
        let key = self.cfg.settings.read().await.auth.user_claim_token_pepper.clone();

        // Create JWT that expires in two weeks
        let key = key.as_bytes();

        // todo: create config option for setting the token validity in seconds.
        let exp_date = clock::now() + 1_209_600; // two weeks from now

        let claims = UserClaims { user, exp: exp_date };

        encode(&Header::default(), &claims, &EncodingKey::from_secret(key)).expect("argument `Header` should match `EncodingKey`")
    }

    /// Verify Json Web Token.
    ///
    /// # Errors
    ///
    /// This function will return an error if the JWT is not good or expired.
    pub async fn verify(&self, token: &str) -> Result<UserClaims, AuthError> {
        let settings = self.cfg.settings.read().await;

        match decode::<UserClaims>(
            token,
            &DecodingKey::from_secret(settings.auth.user_claim_token_pepper.as_bytes()),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(token_data) => {
                if token_data.claims.exp < clock::now() {
                    return Err(AuthError::TokenExpired);
                }
                Ok(token_data.claims)
            }
            Err(_) => Err(AuthError::TokenInvalid),
        }
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
            if Pbkdf2.verify_password(password, &parsed_hash).is_err() {
                return Err(AuthError::InvalidPassword);
            }

            Ok(())
        }
        _ => Err(AuthError::InvalidPassword),
    }
}
