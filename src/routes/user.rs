use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::common::WebAppData;
use crate::errors::{ServiceError, ServiceResult};
use crate::models::response::{OkResponse, TokenResponse};
use crate::routes::API_VERSION;
use crate::web::api::v1::contexts::user::forms::RegistrationForm;

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
            .service(web::resource("/login").route(web::post().to(login_handler)))
            .service(web::resource("/token/verify").route(web::post().to(verify_token_handler)))
            .service(web::resource("/token/renew").route(web::post().to(renew_token_handler)))
            // User Access Ban
            // code-review: should not this be a POST method? We add the user to the blacklist. We do not delete the user.
            .service(web::resource("/ban/{user}").route(web::delete().to(ban_handler))),
    );
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
    // todo: check if `base_url` option was define in settings `net->base_url`.
    // It should have priority over request he
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
/// This function will return an error if the user could not be logged in.
pub async fn login_handler(payload: web::Json<Login>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let (token, user_compact) = app_data
        .authentication_service
        .login(&payload.login, &payload.password)
        .await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: TokenResponse {
            token,
            username: user_compact.username,
            admin: user_compact.administrator,
        },
    }))
}

/// Verify a supplied JWT.
///
/// # Errors
///
/// This function will return an error if unable to verify the supplied payload as a valid jwt.
pub async fn verify_token_handler(payload: web::Json<Token>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    // Verify if JWT is valid
    let _claims = app_data.json_web_token.verify(&payload.token).await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: "Token is valid.".to_string(),
    }))
}

/// Renew a supplied JWT.
///
/// # Errors
///
/// This function will return an error if unable to verify the supplied
/// payload as a valid JWT.
pub async fn renew_token_handler(payload: web::Json<Token>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let (token, user_compact) = app_data.authentication_service.renew_token(&payload.token).await?;

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
/// This function will return if the user could not be banned.
pub async fn ban_handler(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user_id = app_data.auth.get_user_id_from_request(&req).await?;
    let to_be_banned_username = req.match_info().get("user").ok_or(ServiceError::InternalServerError)?;

    app_data.ban_service.ban_user(to_be_banned_username, &user_id).await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: format!("Banned user: {to_be_banned_username}"),
    }))
}
