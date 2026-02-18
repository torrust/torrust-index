use std::collections::HashMap;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use lazy_static::lazy_static;
use lettre::message::{MessageBuilder, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use serde::{Deserialize, Serialize};
use serde_json::value::{to_value, Value};
use tera::{try_get_value, Context, Tera};

use crate::config::Configuration;
use crate::errors::ServiceError;
use crate::utils::clock;
use crate::web::api::server::v1::routes::API_VERSION_URL_PREFIX;

/// How long an email verification token is valid.
pub const VERIFY_TOKEN_EXPIRY_SECS: u64 = 86_400; // 24 hours

/// How long a password reset token is valid.
pub const RESET_TOKEN_EXPIRY_SECS: u64 = 3_600; // 1 hour

/// Returns an ISO 8601 UTC timestamp for `secs` seconds from now.
///
/// Example output: `2026-02-19T14:30:00Z`.
///
/// # Panics
///
/// Panics if `secs` overflows an `i64` or falls outside the range
/// supported by `chrono::TimeDelta`.
#[must_use]
pub fn expiry_timestamp(secs: u64) -> String {
    let dt = Utc::now() + TimeDelta::try_seconds(i64::try_from(secs).expect("expiry fits in i64")).expect("expiry within range");
    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();

        // Use `include_str!` to embed templates at compile time so that
        // template loading does not depend on the process working directory.
        // Figment's `Jail` (used in config tests) changes the process-wide
        // CWD to a temp directory, which causes `add_template_file` with
        // relative paths to fail when the lazy_static is first accessed from
        // a concurrent test thread.
        tera.add_raw_template("html_verify_email", include_str!("../templates/verify.html"))
            .expect("Failed to parse the email verification template");

        tera.add_raw_template("html_reset_password", include_str!("../templates/reset_password.html"))
            .expect("Failed to parse the reset password template");

        tera.autoescape_on(vec![".html", ".sql"]);
        tera.register_filter("do_nothing", do_nothing_filter);
        tera
    };
}

/// This function is a dummy filter for tera.
///
/// # Panics
///
/// Panics if unable to convert values.
///
/// # Errors
///
/// This function will return an error if...
#[allow(clippy::implicit_hasher)]
pub fn do_nothing_filter(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = try_get_value!("do_nothing_filter", "value", String, value);
    Ok(to_value(s).unwrap())
}

pub struct Service {
    cfg: Arc<Configuration>,
    mailer: Arc<Mailer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyClaims {
    pub iss: String,
    pub sub: i64,
    pub exp: u64,
}

/// JWT claims for password reset tokens.
#[derive(Debug, Serialize, Deserialize)]
pub struct ResetClaims {
    pub iss: String,
    pub sub: i64,
    pub exp: u64,
    /// Fingerprint of the current password hash; invalidates the token if the
    /// password is changed before the token is used.
    pub pwd: u64,
}

/// Compute a fingerprint of the password hash for embedding in reset tokens.
///
/// The token self-invalidates when the password changes because the stored
/// fingerprint will no longer match.
///
/// **Note:** This uses [`std::collections::hash_map::DefaultHasher`] whose
/// output is *not* guaranteed to be stable across Rust compiler versions.
/// This is acceptable because every reset token already carries a short
/// expiry ([`RESET_TOKEN_EXPIRY_SECS`]), so no token will survive a
/// toolchain upgrade.
#[must_use]
pub fn password_fingerprint(password_hash: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    password_hash.hash(&mut hasher);
    hasher.finish()
}

impl Service {
    pub async fn new(cfg: Arc<Configuration>) -> Self {
        let mailer = Arc::new(Self::get_mailer(&cfg).await);

        Self { cfg, mailer }
    }

    async fn get_mailer(cfg: &Configuration) -> Mailer {
        let settings = cfg.settings.read().await;

        if !settings.mail.smtp.credentials.username.is_empty() && !settings.mail.smtp.credentials.password.is_empty() {
            // SMTP authentication
            let creds = Credentials::new(
                settings.mail.smtp.credentials.username.clone(),
                settings.mail.smtp.credentials.password.clone(),
            );

            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&settings.mail.smtp.server)
                .port(settings.mail.smtp.port)
                .credentials(creds)
                .authentication(vec![Mechanism::Login, Mechanism::Xoauth2, Mechanism::Plain])
                .build()
        } else {
            // SMTP without authentication
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&settings.mail.smtp.server)
                .port(settings.mail.smtp.port)
                .build()
        }
    }

    /// Send Verification Email.
    ///
    /// # Errors
    ///
    /// This function will return an error if unable to send an email.
    ///
    /// # Panics
    ///
    /// This function will panic if the multipart builder had an error.
    pub async fn send_verification_mail(
        &self,
        to: &str,
        username: &str,
        user_id: i64,
        base_url: &str,
    ) -> Result<(), ServiceError> {
        let builder = self.get_builder(to).await;
        let verification_url = self.get_verification_url(user_id, base_url).await;
        let expiry = expiry_timestamp(VERIFY_TOKEN_EXPIRY_SECS);

        let mail = build_letter(verification_url.as_str(), username, &expiry, builder)?;

        match self.mailer.send(mail).await {
            Ok(_res) => Ok(()),
            Err(e) => {
                eprintln!("Failed to send email: {e}");
                Err(ServiceError::FailedToSendVerificationEmail)
            }
        }
    }

    async fn get_builder(&self, to: &str) -> MessageBuilder {
        let settings = self.cfg.settings.read().await;

        Message::builder()
            .from(settings.mail.from.clone())
            .reply_to(settings.mail.reply_to.clone())
            .to(to.parse().unwrap())
    }

    async fn get_verification_url(&self, user_id: i64, base_url: &str) -> String {
        let settings = self.cfg.settings.read().await;

        // create verification JWT
        let key = settings.auth.user_claim_token_pepper.as_bytes();

        let claims = VerifyClaims {
            iss: String::from("email-verification"),
            sub: user_id,
            exp: clock::now() + VERIFY_TOKEN_EXPIRY_SECS,
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(key)).unwrap();

        // Prefer the frontend URL prefix from config; fall back to the API URL.
        let url_prefix = settings.website.email_verification_url_prefix.clone().unwrap_or_else(|| {
            let api_base = settings
                .net
                .base_url
                .as_ref()
                .map_or_else(|| base_url.to_string(), std::string::ToString::to_string);
            format!("{api_base}/{API_VERSION_URL_PREFIX}/user/email/verify")
        });
        drop(settings);

        format!("{url_prefix}/{token}")
    }

    /// Send a password reset link email.
    ///
    /// # Errors
    ///
    /// This function will return an error if unable to send an email.
    ///
    /// # Panics
    ///
    /// This function will panic if the multipart builder had an error.
    pub async fn send_reset_password_mail(
        &self,
        to: &str,
        username: &str,
        user_id: i64,
        password_hash: &str,
        base_url: &str,
    ) -> Result<(), ServiceError> {
        let builder = self.get_builder(to).await;
        let reset_url = self.get_reset_url(user_id, password_hash, base_url).await;
        let expiry = expiry_timestamp(RESET_TOKEN_EXPIRY_SECS);

        let mail = build_reset_password_letter(&reset_url, username, &expiry, builder)?;

        match self.mailer.send(mail).await {
            Ok(_res) => Ok(()),
            Err(e) => {
                tracing::error!("Failed to send reset password email: {e}");
                Err(ServiceError::FailedToSendResetPassword)
            }
        }
    }

    async fn get_reset_url(&self, user_id: i64, password_hash: &str, base_url: &str) -> String {
        let settings = self.cfg.settings.read().await;

        let key = settings.auth.user_claim_token_pepper.as_bytes();

        let claims = ResetClaims {
            iss: String::from("password-reset"),
            sub: user_id,
            exp: clock::now() + RESET_TOKEN_EXPIRY_SECS,
            pwd: password_fingerprint(password_hash),
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(key)).unwrap();

        // Prefer the frontend URL prefix from config; fall back to the API URL.
        let url_prefix = settings.website.password_reset_url_prefix.clone().unwrap_or_else(|| {
            let api_base = settings
                .net
                .base_url
                .as_ref()
                .map_or_else(|| base_url.to_string(), std::string::ToString::to_string);
            format!("{api_base}/{API_VERSION_URL_PREFIX}/user/password-reset")
        });
        drop(settings);

        format!("{url_prefix}/{token}")
    }
}

fn build_letter(verification_url: &str, username: &str, expiry: &str, builder: MessageBuilder) -> Result<Message, ServiceError> {
    let (plain_body, html_body) = build_content(verification_url, username, expiry).map_err(|e| {
        tracing::error!("{e}");
        ServiceError::InternalServerError
    })?;

    Ok(builder
        .subject("Torrust - Email verification")
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(lettre::message::header::ContentType::TEXT_PLAIN)
                        .body(plain_body),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(lettre::message::header::ContentType::TEXT_HTML)
                        .body(html_body),
                ),
        )
        .expect("the `multipart` builder had an error"))
}

fn build_content(verification_url: &str, username: &str, expiry: &str) -> Result<(String, String), tera::Error> {
    let plain_body = format!(
        "
                Welcome to Torrust, {username}!

                Please click the confirmation link below to verify your account.
                {verification_url}

                This link expires on {expiry}.

                If this account wasn't made by you, you can ignore this email.
            "
    );
    let mut context = Context::new();
    context.insert("verification", &verification_url);
    context.insert("username", &username);
    context.insert("expiry", &expiry);
    let html_body = TEMPLATES.render("html_verify_email", &context)?;
    Ok((plain_body, html_body))
}

fn build_reset_password_letter(
    reset_url: &str,
    username: &str,
    expiry: &str,
    builder: MessageBuilder,
) -> Result<Message, ServiceError> {
    let (plain_body, html_body) = build_reset_password_content(reset_url, username, expiry).map_err(|e| {
        tracing::error!("{e}");
        ServiceError::InternalServerError
    })?;

    Ok(builder
        .subject("Torrust - Password reset")
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(lettre::message::header::ContentType::TEXT_PLAIN)
                        .body(plain_body),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(lettre::message::header::ContentType::TEXT_HTML)
                        .body(html_body),
                ),
        )
        .expect("the `multipart` builder had an error"))
}

fn build_reset_password_content(reset_url: &str, username: &str, expiry: &str) -> Result<(String, String), tera::Error> {
    let plain_body = format!(
        "
                Hello, {username}!

                We received a request to reset your password.
                Click the following link to reset your password:
                {reset_url}

                This link expires on {expiry}.

                If you did not request this, you can safely ignore this email.
            "
    );
    let mut context = Context::new();
    context.insert("reset_url", &reset_url);
    context.insert("username", &username);
    context.insert("expiry", &expiry);
    let html_body = TEMPLATES.render("html_reset_password", &context)?;
    Ok((plain_body, html_body))
}

pub type Mailer = AsyncSmtpTransport<Tokio1Executor>;

#[cfg(test)]
mod tests {
    use lettre::Message;

    use super::{build_content, build_letter};

    #[test]
    fn it_should_build_a_letter() {
        let builder = Message::builder()
            .from("from@a.b.c".parse().unwrap())
            .reply_to("reply@a.b.c".parse().unwrap())
            .to("to@a.b.c".parse().unwrap());

        let _letter = build_letter("https://a.b.c/", "user", "2026-02-19 14:30 UTC", builder).unwrap();
    }

    #[test]
    fn it_should_build_content() {
        let (plain_body, html_body) = build_content("https://a.b.c/", "user", "2026-02-19 14:30 UTC").unwrap();
        assert_ne!(plain_body, "");
        assert_ne!(html_body, "");
    }
}
