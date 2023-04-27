use actix_web::{web, HttpRequest, HttpResponse, Responder};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use log::{debug, info};
use pbkdf2::password_hash::rand_core::OsRng;
use pbkdf2::Pbkdf2;
use serde::{Deserialize, Serialize};

use crate::common::WebAppData;
use crate::config::EmailOnSignup;
use crate::errors::{ServiceError, ServiceResult};
use crate::mailer::VerifyClaims;
use crate::models::response::{OkResponse, TokenResponse};
use crate::models::user::UserAuthentication;
use crate::utils::clock;
use crate::utils::regex::validate_email_address;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .service(web::resource("/register").route(web::post().to(register)))
            .service(web::resource("/login").route(web::post().to(login)))
            // code-review: should not this be a POST method? We add the user to the blacklist. We do not delete the user.
            .service(web::resource("/ban/{user}").route(web::delete().to(ban)))
            .service(web::resource("/token/verify").route(web::post().to(verify_token)))
            .service(web::resource("/token/renew").route(web::post().to(renew_token)))
            .service(web::resource("/email/verify/{token}").route(web::get().to(verify_email))),
    );
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Register {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub confirm_password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Login {
    pub login: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Token {
    pub token: String,
}

/// Register a User in the Index
///
/// # Errors
///
/// This function will return a `ServiceError::EmailMissing` if email is required, but missing.
/// This function will return a `ServiceError::EmailInvalid` if supplied email is badly formatted.
/// This function will return a `ServiceError::PasswordsDontMatch` if the supplied passwords do not match.
/// This function will return a `ServiceError::PasswordTooShort` if the supplied password is too short.
/// This function will return a `ServiceError::PasswordTooLong` if the supplied password is too long.
/// This function will return a `ServiceError::UsernameInvalid` if the supplied username is badly formatted.
/// This function will return an error if unable to successfully hash the password.
/// This function will return an error if unable to insert user into the database.
/// This function will return a `ServiceError::FailedToSendVerificationEmail` if unable to send the required verification email.
pub async fn register(req: HttpRequest, mut payload: web::Json<Register>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    info!("registering user: {}", payload.username);

    let settings = app_data.cfg.settings.read().await;

    match settings.auth.email_on_signup {
        EmailOnSignup::Required => {
            if payload.email.is_none() {
                return Err(ServiceError::EmailMissing);
            }
        }
        EmailOnSignup::None => payload.email = None,
        EmailOnSignup::Optional => {}
    }

    if let Some(email) = &payload.email {
        // check if email address is valid
        if !validate_email_address(email) {
            return Err(ServiceError::EmailInvalid);
        }
    }

    if payload.password != payload.confirm_password {
        return Err(ServiceError::PasswordsDontMatch);
    }

    let password_length = payload.password.len();

    if password_length <= settings.auth.min_password_length {
        return Err(ServiceError::PasswordTooShort);
    }

    if password_length >= settings.auth.max_password_length {
        return Err(ServiceError::PasswordTooLong);
    }

    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2.hash_password(payload.password.as_bytes(), &salt)?.to_string();

    if payload.username.contains('@') {
        return Err(ServiceError::UsernameInvalid);
    }

    let email = payload.email.as_ref().unwrap_or(&String::new()).to_string();

    let user_id = app_data
        .database
        .insert_user_and_get_id(&payload.username, &email, &password_hash)
        .await?;

    // if this is the first created account, give administrator rights
    if user_id == 1 {
        let _ = app_data.database.grant_admin_role(user_id).await;
    }

    let conn_info = req.connection_info().clone();

    if settings.mail.email_verification_enabled && payload.email.is_some() {
        let mail_res = app_data
            .mailer
            .send_verification_mail(
                payload.email.as_ref().expect("variable `email` is checked above"),
                &payload.username,
                user_id,
                format!("{}://{}", conn_info.scheme(), conn_info.host()).as_str(),
            )
            .await;

        if mail_res.is_err() {
            let _ = app_data.database.delete_user(user_id).await;
            return Err(ServiceError::FailedToSendVerificationEmail);
        }
    }

    Ok(HttpResponse::Ok())
}

/// Login user to Index
///
/// # Errors
///
/// This function will return a `ServiceError::WrongPasswordOrUsername` if unable to get user profile.
/// This function will return a `ServiceError::InternalServerError` if unable to get user authentication data from the user id.
/// This function will return an error if unable to verify the password.
/// This function will return a `ServiceError::EmailNotVerified` if the email should be, but is not verified.
/// This function will return an error if unable to get the user data from the database.
pub async fn login(payload: web::Json<Login>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    // get the user profile from database
    let user_profile = app_data
        .database
        .get_user_profile_from_username(&payload.login)
        .await
        .map_err(|_| ServiceError::WrongPasswordOrUsername)?;

    // should not be able to fail if user_profile succeeded
    let user_authentication = app_data
        .database
        .get_user_authentication_from_id(user_profile.user_id)
        .await
        .map_err(|_| ServiceError::InternalServerError)?;

    verify_password(payload.password.as_bytes(), &user_authentication)?;

    let settings = app_data.cfg.settings.read().await;

    // fail login if email verification is required and this email is not verified
    if settings.mail.email_verification_enabled && !user_profile.email_verified {
        return Err(ServiceError::EmailNotVerified);
    }

    // drop read lock on settings
    drop(settings);

    let user_compact = app_data.database.get_user_compact_from_id(user_profile.user_id).await?;

    // sign jwt with compact user details as payload
    let token = app_data.auth.sign_jwt(user_compact.clone()).await;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: TokenResponse {
            token,
            username: user_compact.username,
            admin: user_compact.administrator,
        },
    }))
}

/// Verify if the user supplied and the database supplied passwords match
///
/// # Errors
///
/// This function will return an error if unable to parse password hash from the stored user authentication value.
/// This function will return a `ServiceError::WrongPasswordOrUsername` if unable to match the password with either `argon2id` or `pbkdf2-sha256`.
pub fn verify_password(password: &[u8], user_authentication: &UserAuthentication) -> Result<(), ServiceError> {
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

/// Verify a supplied JWT.
///
/// # Errors
///
/// This function will return an error if unable to verify the supplied payload as a valid jwt.
pub async fn verify_token(payload: web::Json<Token>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    // verify if token is valid
    let _claims = app_data.auth.verify_jwt(&payload.token).await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: "Token is valid.".to_string(),
    }))
}

/// Renew a supplied JWT.
///
/// # Errors
///
/// This function will return an error if unable to verify the supplied payload as a valid jwt.
/// This function will return an error if unable to get user data from the database.
pub async fn renew_token(payload: web::Json<Token>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    const ONE_WEEK_IN_SECONDS: u64 = 604_800;

    // verify if token is valid
    let claims = app_data.auth.verify_jwt(&payload.token).await?;

    let user_compact = app_data.database.get_user_compact_from_id(claims.user.user_id).await?;

    // renew token if it is valid for less than one week
    let token = match claims.exp - clock::now() {
        x if x < ONE_WEEK_IN_SECONDS => app_data.auth.sign_jwt(user_compact.clone()).await,
        _ => payload.token.clone(),
    };

    Ok(HttpResponse::Ok().json(OkResponse {
        data: TokenResponse {
            token,
            username: user_compact.username,
            admin: user_compact.administrator,
        },
    }))
}

pub async fn verify_email(req: HttpRequest, app_data: WebAppData) -> String {
    let settings = app_data.cfg.settings.read().await;
    let token = match req.match_info().get("token").ok_or(ServiceError::InternalServerError) {
        Ok(token) => token,
        Err(err) => return err.to_string(),
    };

    let token_data = match decode::<VerifyClaims>(
        token,
        &DecodingKey::from_secret(settings.auth.secret_key.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(token_data) => {
            if !token_data.claims.iss.eq("email-verification") {
                return ServiceError::TokenInvalid.to_string();
            }

            token_data.claims
        }
        Err(_) => return ServiceError::TokenInvalid.to_string(),
    };

    drop(settings);

    if app_data.database.verify_email(token_data.sub).await.is_err() {
        return ServiceError::InternalServerError.to_string();
    };

    String::from("Email verified, you can close this page.")
}

/// Ban a user from the Index
///
/// TODO: add reason and `date_expiry` parameters to request
///
/// # Errors
///
/// This function will return a `ServiceError::InternalServerError` if unable get user from the request.
/// This function will return an error if unable to get user profile from supplied username.
/// This function will return an error if unable to ser the ban of the user in the database.
pub async fn ban(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    debug!("banning user");

    let user = app_data.auth.get_user_compact_from_request(&req).await?;

    // check if user is administrator
    if !user.administrator {
        return Err(ServiceError::Unauthorized);
    }

    let to_be_banned_username = req.match_info().get("user").ok_or(ServiceError::InternalServerError)?;

    debug!("user to be banned: {}", to_be_banned_username);

    let user_profile = app_data
        .database
        .get_user_profile_from_username(to_be_banned_username)
        .await?;

    let reason = "no reason".to_string();

    // user will be banned until the year 9999
    let date_expiry = chrono::NaiveDateTime::parse_from_str("9999-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
        .expect("Could not parse date from 9999-01-01 00:00:00.");

    app_data.database.ban_user(user_profile.user_id, &reason, date_expiry).await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: format!("Banned user: {}", to_be_banned_username),
    }))
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
