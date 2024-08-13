//! API context: `user`.
//!
//! This API context is responsible for handling:
//!
//! - User registration
//! - User authentication
//! - User ban
//!
//! For more information about the API authentication, refer to the [`auth`](crate::web::api::server::v1::auth)
//! module.
//!
//! # Endpoints
//!
//! Registration:
//!
//! - [Registration](#registration)
//! - [Email verification](#email-verification)
//!
//! Authentication:
//!
//! - [Login](#login)
//! - [Token verification](#token-verification)
//! - [Token renewal](#token-renewal)
//!
//! User ban:
//!
//! - [Ban a user](#ban-a-user)
//!
//! # Registration
//!
//! `POST /v1/user/register`
//!
//! It registers a new user.
//!
//! **Post parameters**
//!
//! Name | Type | Description | Required | Example
//! ---|---|---|---|---
//! `username` | `String` | The username | Yes | `indexadmin`
//! `email` | `Option<String>` | The user's email  | No | `indexadmin@torrust.com`
//! `password` | `String` | The password  | Yes | `BenoitMandelbrot1924`
//! `confirm_password` | `String` | Same password again  | Yes | `BenoitMandelbrot1924`
//!
//! **NOTICE**: Email could be optional, depending on the configuration.
//!
//! ```toml
//! [auth]
//! user_claim_token_pepper = "MaxVerstappenWC2021"
//! ```
//!
//! Refer to the [`RegistrationForm`](crate::web::api::server::v1::contexts::user::forms::RegistrationForm)
//! struct for more information about the registration form.
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --request POST \
//!   --data '{"username":"indexadmin","email":"indexadmin@torrust.com","password":"BenoitMandelbrot1924","confirm_password":"BenoitMandelbrot1924"}' \
//!   http://127.0.0.1:3001/v1/user/register
//! ```
//!
//! For more information about the registration process, refer to the [`auth`](crate::web::api::server::v1::auth)
//! module.
//!
//! # Email verification
//!
//! `GET /v1/user/email/verify/{token}`
//!
//! If email on signup is enabled, the user will receive an email with a
//! verification link. The link will contain a token that can be used to verify
//! the email address.
//!
//! This endpoint will verify the email address and update the user's email
//! verification status. It also shows an text page with the result of the
//! verification.
//!
//! **Example response** `200`
//!
//! ```text
//! Email verified, you can close this page.
//! ```
//!
//! # Login
//!
//! `POST /v1/user/login`
//!
//! It logs in a user.
//!
//! **Post parameters**
//!
//! Name | Type | Description | Required | Example
//! ---|---|---|---|---
//! `login` | `String` | The password  | Yes | `indexadmin`
//! `password` | `String` | The password  | Yes | `BenoitMandelbrot1924`
//!
//! Refer to the [`LoginForm`](crate::web::api::server::v1::contexts::user::forms::LoginForm)
//! struct for more information about the login form.
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --request POST \
//!   --data '{"login":"indexadmin","password":"BenoitMandelbrot1924"}' \
//!   http://127.0.0.1:3001/v1/user/login
//! ```
//!
//! For more information about the login process, refer to the [`auth`](crate::web::api::server::v1::auth)
//! module.
//!
//! # Token verification
//!
//! `POST /v1/user/token/verify`
//!
//! It logs in a user.
//!
//! **Post parameters**
//!
//! Name | Type | Description | Required | Example
//! ---|---|---|---|---
//! `token` | `String` | The token you want to verify  | Yes | `eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI`
//!
//! Refer to the [`JsonWebToken`](crate::web::api::server::v1::contexts::user::forms::JsonWebToken)
//! struct for more information about the token.
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --request POST \
//!   --data '{"token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI"}' \
//!   http://127.0.0.1:3001/v1/user/token/verify
//! ```
//!
//! **Example response** `200`
//!
//! For a valid token:
//!
//! ```json
//! {
//!   "data":"Token is valid."
//! }
//! ```
//!
//! And for an invalid token:
//!
//! ```json
//! {
//!   "data":"Token invalid."
//! }
//! ```
//!
//! # Token renewal
//!
//! `POST /v1/user/token/verify`
//!
//! It renew a user's token.
//!
//! The token must be valid and not expired. And it's only renewed if it is
//! valid for less than one week.
//!
//! **Post parameters**
//!
//! Name | Type | Description | Required | Example
//! ---|---|---|---|---
//! `token` | `String` | The current valid token | Yes | `eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI`
//!
//! Refer to the [`JsonWebToken`](crate::web::api::server::v1::contexts::user::forms::JsonWebToken)
//! struct for more information about the token.
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --request POST \
//!   --data '{"token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI"}' \
//!   http://127.0.0.1:3001/v1/user/token/renew
//! ```
//!
//! **Example response** `200`
//!
//! If you try to renew a token that is still valid for more than one week:
//!
//! ```json
//! {
//!   "data": {
//!     "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI",
//!     "username": "indexadmin",
//!     "admin": true
//!   }
//! }
//! ```
//!
//! You will get the same token. If a new token is generated, the response will
//! be the same but with the new token.
//!
//! **WARNING**: The token is associated to the user's role. The application does not support
//! changing the role of a user. If you change the user's role manually in the
//! database, the token will still be valid but with the same role. That should
//! only be done for testing purposes.
//!
//! # Ban a user
//!
//! `DELETE /v1/user/ban/{user}`
//!
//! It add a user to the banned user list.
//!
//! Only admin can ban other users.
//!
//! **Path parameters**
//!
//! Name | Type | Description | Required | Example
//! ---|---|---|---|---
//! `user` | `String` | username | Yes | `indexadmin`
//!
//! **Example request**
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI" \
//!   --request DELETE \
//!   http://127.0.0.1:3001/v1/user/ban/indexadmin
//! ```
//!
//! **Example response** `200`
//!
//! If you try to renew a token that is still valid for more than one week:
//!
//! ```json
//! {
//!   "data": "Banned user: indexadmin"
//! }
//! ```
//!
//! **WARNING**: The admin can ban themselves. If they do, they will not be able
//! to unban themselves. The only way to unban themselves is to manually remove
//! the user from the banned user list in the database.
pub mod forms;
pub mod handlers;
pub mod responses;
pub mod routes;
