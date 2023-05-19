use actix_web::{web, HttpRequest, HttpResponse, Responder};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use log::debug;
use pbkdf2::Pbkdf2;
use serde::{Deserialize, Serialize};

use crate::common::WebAppData;
use crate::errors::{ServiceError, ServiceResult};
use crate::models::response::{OkResponse, TokenResponse};
use crate::models::user::UserAuthentication;
use crate::routes::API_VERSION;
use crate::utils::clock;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(&format!("/{API_VERSION}/user"))
            // Registration
            .service(web::resource("/register").route(web::post().to(registration_handler)))
            // code-review: should this be part of the REST API?
            // - This endpoint should only verify the email.
            // - There should be an independent service (web app) serving the email verification page.
            //   The wep app can user this endpoint to verify the email and render the page accordingly.
            .service(web::resource("/email/verify/{token}").route(web::get().to(email_verification_handler)))
            // Authentication
            .service(web::resource("/login").route(web::post().to(login)))
            .service(web::resource("/token/verify").route(web::post().to(verify_token)))
            .service(web::resource("/token/renew").route(web::post().to(renew_token)))
            // User Access Ban
            // code-review: should not this be a POST method? We add the user to the blacklist. We do not delete the user.
            .service(web::resource("/ban/{user}").route(web::delete().to(ban))),
    );
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegistrationForm {
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
/// This function will return an error if the user could not be registered.
pub async fn registration_handler(
    req: HttpRequest,
    registration_form: web::Json<RegistrationForm>,
    app_data: WebAppData,
) -> ServiceResult<impl Responder> {
    let conn_info = req.connection_info().clone();
    // todo: we should add this in the configuration. It does not work is the
    // server is behind a reverse proxy.
    let api_base_url = format!("{}://{}", conn_info.scheme(), conn_info.host());

    let _user_id = app_data
        .registration_service
        .register_user(&registration_form, &api_base_url)
        .await?;

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

pub async fn email_verification_handler(req: HttpRequest, app_data: WebAppData) -> String {
    // Get token from URL path
    let token = match req.match_info().get("token").ok_or(ServiceError::InternalServerError) {
        Ok(token) => token,
        Err(err) => return err.to_string(),
    };

    match app_data.registration_service.verify_email(token).await {
        Ok(_) => String::from("Email verified, you can close this page."),
        Err(error) => error.to_string(),
    }
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
        data: format!("Banned user: {to_be_banned_username}"),
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
