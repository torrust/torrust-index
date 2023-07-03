//! Authentication services.
use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use pbkdf2::Pbkdf2;

use super::user::{DbUserProfileRepository, DbUserRepository};
use crate::config::Configuration;
use crate::databases::database::{Database, Error};
use crate::errors::ServiceError;
use crate::models::user::{UserAuthentication, UserClaims, UserCompact, UserId};
use crate::utils::clock;

pub struct Service {
    configuration: Arc<Configuration>,
    json_web_token: Arc<JsonWebToken>,
    user_repository: Arc<DbUserRepository>,
    user_profile_repository: Arc<DbUserProfileRepository>,
    user_authentication_repository: Arc<DbUserAuthenticationRepository>,
}

impl Service {
    pub fn new(
        configuration: Arc<Configuration>,
        json_web_token: Arc<JsonWebToken>,
        user_repository: Arc<DbUserRepository>,
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
    /// * A `ServiceError::WrongPasswordOrUsername` if unable to get user profile.
    /// * A `ServiceError::InternalServerError` if unable to get user authentication data from the user id.
    /// * A `ServiceError::EmailNotVerified` if the email should be, but is not verified.
    /// * An error if unable to verify the password.
    /// * An error if unable to get the user data from the database.
    pub async fn login(&self, username: &str, password: &str) -> Result<(String, UserCompact), ServiceError> {
        // Get the user profile from database
        let user_profile = self
            .user_profile_repository
            .get_user_profile_from_username(username)
            .await
            .map_err(|_| ServiceError::WrongPasswordOrUsername)?;

        // Should not be able to fail if user_profile succeeded
        let user_authentication = self
            .user_authentication_repository
            .get_user_authentication_from_id(&user_profile.user_id)
            .await
            .map_err(|_| ServiceError::InternalServerError)?;

        verify_password(password.as_bytes(), &user_authentication)?;

        let settings = self.configuration.settings.read().await;

        // Fail login if email verification is required and this email is not verified
        if settings.mail.email_verification_enabled && !user_profile.email_verified {
            return Err(ServiceError::EmailNotVerified);
        }

        // Drop read lock on settings
        drop(settings);

        let user_compact = self.user_repository.get_compact(&user_profile.user_id).await?;

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
    pub async fn renew_token(&self, token: &str) -> Result<(String, UserCompact), ServiceError> {
        const ONE_WEEK_IN_SECONDS: u64 = 604_800;

        // Verify if token is valid
        let claims = self.json_web_token.verify(token).await?;

        let user_compact = self.user_repository.get_compact(&claims.user.user_id).await?;

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
    pub fn new(cfg: Arc<Configuration>) -> Self {
        Self { cfg }
    }

    /// Create Json Web Token.
    ///
    /// # Panics
    ///
    /// This function will panic if the default encoding algorithm does not ç
    /// match the encoding key.
    pub async fn sign(&self, user: UserCompact) -> String {
        let settings = self.cfg.settings.read().await;

        // Create JWT that expires in two weeks
        let key = settings.auth.secret_key.as_bytes();

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
    pub async fn verify(&self, token: &str) -> Result<UserClaims, ServiceError> {
        let settings = self.cfg.settings.read().await;

        match decode::<UserClaims>(
            token,
            &DecodingKey::from_secret(settings.auth.secret_key.as_bytes()),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(token_data) => {
                if token_data.claims.exp < clock::now() {
                    return Err(ServiceError::TokenExpired);
                }
                Ok(token_data.claims)
            }
            Err(_) => Err(ServiceError::TokenInvalid),
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
}

/// Verify if the user supplied and the database supplied passwords match
///
/// # Errors
///
/// This function will return an error if unable to parse password hash from the stored user authentication value.
/// This function will return a `ServiceError::WrongPasswordOrUsername` if unable to match the password with either `argon2id` or `pbkdf2-sha256`.
fn verify_password(password: &[u8], user_authentication: &UserAuthentication) -> Result<(), ServiceError> {
    // wrap string of the hashed password into a PasswordHash struct for verification
    let parsed_hash = PasswordHash::new(&user_authentication.password_hash)?;

    match parsed_hash.algorithm.as_str() {
        "argon2id" => {
            if Argon2::default().verify_password(password, &parsed_hash).is_err() {
                return Err(ServiceError::WrongPasswordOrUsername);
            }

            Ok(())
        }
        "pbkdf2-sha256" => {
            if Pbkdf2.verify_password(password, &parsed_hash).is_err() {
                return Err(ServiceError::WrongPasswordOrUsername);
            }

            Ok(())
        }
        _ => Err(ServiceError::WrongPasswordOrUsername),
    }
}

#[cfg(test)]
mod tests {
    use super::verify_password;
    use crate::models::user::UserAuthentication;

    #[test]
    fn password_hashed_with_pbkdf2_sha256_should_be_verified() {
        let password = "12345678".as_bytes();
        let password_hash =
            "$pbkdf2-sha256$i=10000,l=32$pZIh8nilm+cg6fk5Ubf2zQ$AngLuZ+sGUragqm4bIae/W+ior0TWxYFFaTx8CulqtY".to_string();
        let user_authentication = UserAuthentication {
            user_id: 1i64,
            password_hash,
        };

        assert!(verify_password(password, &user_authentication).is_ok());
        assert!(verify_password("incorrect password".as_bytes(), &user_authentication).is_err());
    }

    #[test]
    fn password_hashed_with_argon2_should_be_verified() {
        let password = "87654321".as_bytes();
        let password_hash =
            "$argon2id$v=19$m=4096,t=3,p=1$ycK5lJ4xmFBnaJ51M1j1eA$kU3UlNiSc3JDbl48TCj7JBDKmrT92DOUAgo4Yq0+nMw".to_string();
        let user_authentication = UserAuthentication {
            user_id: 1i64,
            password_hash,
        };

        assert!(verify_password(password, &user_authentication).is_ok());
        assert!(verify_password("incorrect password".as_bytes(), &user_authentication).is_err());
    }
}
