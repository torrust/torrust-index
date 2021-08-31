use actix_web::{web, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use pbkdf2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Pbkdf2,
};

use crate::models::User;
use crate::AppData;
use crate::errors::{ServiceError, ServiceResult};
use crate::CONFIG;
use std::borrow::Cow;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .service(web::resource("/register").route(web::post().to(register)))
            .service(web::resource("/login").route(web::post().to(login)))
    );
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Register {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Login {
    pub login: String,
    pub password: String,
}

pub async fn register(payload: web::Json<Register>, data: AppData) -> ServiceResult<impl Responder> {
    if payload.password != payload.confirm_password {
        return Err(ServiceError::PasswordsDontMatch);
    }

    let password_length = payload.password.len();
    if password_length <= CONFIG.auth.min_password_length {
        return Err(ServiceError::PasswordTooShort);
    }
    if password_length >= CONFIG.auth.max_password_length {
        return Err(ServiceError::PasswordTooLong);
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash;
    if let Ok(password) = Pbkdf2.hash_password(payload.password.as_bytes(), &salt) {
        password_hash = password.to_string();
    } else {
        return Err(ServiceError::InternalServerError);
    }

    if payload.username.contains('@') {
        return Err(ServiceError::UsernameInvalid)
    }

    let res = sqlx::query!(
        "INSERT INTO torrust_users (username, email, password) VALUES ($1, $2, $3)",
        payload.username,
        payload.email,
        password_hash,
    )
        .execute(&data.db)
        .await;

    if let Err(sqlx::Error::Database(err)) = res {
        return if err.code() == Some(Cow::from("2067")) {
            if err.message().contains("torrust_users.username") {
                Err(ServiceError::UsernameTaken)
            } else if err.message().contains("torrust_users.email") {
                Err(ServiceError::EmailTaken)
            } else {
                Err(ServiceError::InternalServerError)
            }
        } else {
            Err(sqlx::Error::Database(err).into())
        };
    }

    Ok(HttpResponse::Ok())
}

pub async fn login(payload: web::Json<Login>, data: AppData) -> ServiceResult<impl Responder> {
    let res;
    if payload.login.contains('@') {
        res = sqlx::query_as!(
            User,
            "SELECT * FROM torrust_users WHERE email = ?",
            payload.login,
        )
            .fetch_one(&data.db)
            .await
    } else {
        res = sqlx::query_as!(
            User,
            "SELECT * FROM torrust_users WHERE username = ?",
            payload.login,
        )
            .fetch_one(&data.db)
            .await
    }

    match res {
        Ok(user) => {
            let parsed_hash = PasswordHash::new(&user.password)?;

            if !Pbkdf2.verify_password(payload.password.as_bytes(), &parsed_hash).is_ok() {
                return Err(ServiceError::WrongPassword);
            }

            return Ok(HttpResponse::Ok());
        }
        Err(sqlx::Error::RowNotFound) => Err(ServiceError::AccountNotFound),
        Err(_) => Err(ServiceError::InternalServerError),
    }
}