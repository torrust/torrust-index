use actix_web::{web, Responder, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{DecodingKey, decode, Validation, Algorithm};
use pbkdf2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use pbkdf2::Pbkdf2;

use crate::errors::{ServiceResult, ServiceError};
use crate::common::WebAppData;
use crate::config::EmailOnSignup;
use crate::models::response::OkResponse;
use crate::models::response::TokenResponse;
use crate::mailer::VerifyClaims;
use crate::utils::random::random_string;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .service(web::resource("/register")
                .route(web::post().to(register)))
            .service(web::resource("/login")
                .route(web::post().to(login)))
            .service(web::resource("/ban/{user}")
                .route(web::delete().to(ban_user)))
            .service(web::resource("/verify/{token}")
                .route(web::get().to(verify_user)))
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

pub async fn register(req: HttpRequest, mut payload: web::Json<Register>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let settings = app_data.cfg.settings.read().await;

    match settings.auth.email_on_signup {
        EmailOnSignup::Required => {
            if payload.email.is_none() { return Err(ServiceError::EmailMissing) }
        }
        EmailOnSignup::None => {
            payload.email = None
        }
        _ => {}
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
    let password_hash = Pbkdf2.hash_password(payload.password.as_bytes(), &salt)?.to_string();

    if payload.username.contains('@') {
        return Err(ServiceError::UsernameInvalid)
    }

    // can't drop not null constraint on sqlite, so we fill the email with unique junk :)
    let email = payload.email.as_ref().unwrap_or(&format!("EMPTY_EMAIL_{}", random_string(16))).to_string();

    let user_id = app_data.database.insert_user_and_get_id(&payload.username, &email, &password_hash).await?;

    let conn_info = req.connection_info();

    if settings.mail.email_verification_enabled && payload.email.is_some() {
        let mail_res = app_data.mailer.send_verification_mail(
            &payload.email.as_ref().unwrap(),
            &payload.username,
            user_id,
            format!("{}://{}", conn_info.scheme(), conn_info.host()).as_str()
        )
            .await;

        if mail_res.is_err() {
            let _ = app_data.database.delete_user(user_id).await;
            return Err(ServiceError::FailedToSendVerificationEmail)
        }
    }

    Ok(HttpResponse::Ok())
}

async fn grant_admin_role(app_data: &WebAppData, user_id: i64) {
    // count accounts
    let user_count = app_data.database.count_users().await;

    // make admin if first account
    if let Ok(1) = user_count {
        let _ = app_data.database.grant_admin_role(user_id).await;
    }
}

pub async fn login(payload: web::Json<Login>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let settings = app_data.cfg.settings.read().await;

    let res = app_data.database.get_user_from_username(&payload.login).await;

    match res {
        Some(user) => {
            if settings.mail.email_verification_enabled && !user.email_verified {
                return Err(ServiceError::EmailNotVerified)
            }

            drop(settings);

            let parsed_hash = PasswordHash::new(&user.password)?;

            if !Pbkdf2.verify_password(payload.password.as_bytes(), &parsed_hash).is_ok() {
                return Err(ServiceError::WrongPasswordOrUsername);
            }

            let username = user.username.clone();
            let token = app_data.auth.sign_jwt(user.clone()).await;


            Ok(HttpResponse::Ok().json(OkResponse {
                data: TokenResponse {
                    token,
                    username,
                    admin: user.administrator
                }
            }))
        }
        None => Err(ServiceError::WrongPasswordOrUsername)
    }
}

pub async fn verify_user(req: HttpRequest, app_data: WebAppData) -> String {
    let settings = app_data.cfg.settings.read().await;
    let token = req.match_info().get("token").unwrap();

    let token_data = match decode::<VerifyClaims>(
        token,
        &DecodingKey::from_secret(settings.auth.secret_key.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(token_data) => {
            if !token_data.claims.iss.eq("email-verification") {
                return ServiceError::TokenInvalid.to_string()
            }

            token_data.claims
        },
        Err(_) => return ServiceError::TokenInvalid.to_string()
    };

    drop(settings);

    if app_data.database.verify_email(token_data.sub).await.is_err() {
        return ServiceError::InternalServerError.to_string()
    };

    String::from("Email verified, you can close this page.")
}

pub async fn ban_user(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user = app_data.auth.get_user_from_request(&req).await?;

    // check if user is administrator
    if !user.administrator { return Err(ServiceError::Unauthorized) }

    let to_be_banned_username = req.match_info().get("user").unwrap();

    let _ = app_data.database.ban_user(&to_be_banned_username).await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: format!("Banned user: {}", to_be_banned_username)
    }))
}

pub async fn me(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user = match app_data.auth.get_user_from_request(&req).await {
        Ok(user) => Ok(user),
        Err(e) => Err(e)
    }?;

    let username = user.username.clone();
    let token = app_data.auth.sign_jwt(user.clone()).await;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: TokenResponse {
            token,
            username,
            admin: user.administrator
        }
    }))
}
