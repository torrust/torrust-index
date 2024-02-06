//! API routes for the [`user`](crate::web::api::server::v1::contexts::user) API context.
//!
//! Refer to the [API endpoint documentation](crate::web::api::server::v1::contexts::user).
use std::sync::Arc;

use axum::routing::{delete, get, post};
use axum::Router;

use super::handlers::{
    ban_handler, email_verification_handler, login_handler, registration_handler, renew_token_handler, verify_token_handler,
};
use crate::common::AppData;

/// Routes for the [`user`](crate::web::api::server::v1::contexts::user) API context.
pub fn router(app_data: Arc<AppData>) -> Router {
    Router::new()
        // Registration
        .route("/register", post(registration_handler).with_state(app_data.clone()))
        // code-review: should this be part of the REST API?
        // - This endpoint should only verify the email.
        // - There should be an independent service (web app) serving the email verification page.
        //   The wep app can user this endpoint to verify the email and render the page accordingly.
        .route(
            "/email/verify/:token",
            get(email_verification_handler).with_state(app_data.clone()),
        )
        // Authentication
        .route("/login", post(login_handler).with_state(app_data.clone()))
        .route("/token/verify", post(verify_token_handler).with_state(app_data.clone()))
        .route("/token/renew", post(renew_token_handler).with_state(app_data.clone()))
        // User ban
        // code-review: should not this be a POST method? We add the user to the blacklist. We do not delete the user.
        .route("/ban/:user", delete(ban_handler).with_state(app_data))
}
