use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::{Arc, LazyLock};

use lettre::message::{MessageBuilder, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use serde_json::value::{Value, to_value};
use tera::{Context, Tera, try_get_value};
use thiserror::Error;
use tracing::error;

use crate::config::Configuration;
use crate::errors::UserError;
use crate::jwt::JsonWebToken;
// Re-export so existing `use crate::mailer::VerifyClaims` paths keep compiling.
pub use crate::jwt::VerifyClaims;
use crate::web::api::server::v1::routes::API_VERSION_URL_PREFIX;

/// Default verify-email template, compiled into the binary.
const VERIFY_EMAIL_DEFAULT: &str = include_str!("../templates/verify.html");

pub(crate) static TEMPLATES: LazyLock<Result<Tera, MailTemplateError>> = LazyLock::new(build_templates);

#[derive(Debug, Error)]
pub(crate) enum MailTemplateError {
    #[error("failed to read templates/verify.html: {source}")]
    ReadOverride { source: std::io::Error },

    #[error("failed to register email template: {source}")]
    RegisterTemplate { source: tera::Error },

    #[error("failed to initialize email templates: {message}")]
    Initialize { message: String },

    #[error("failed to render email template: {source}")]
    Render { source: tera::Error },
}

fn build_templates() -> Result<Tera, MailTemplateError> {
    let mut tera = Tera::default();

    // Allow deployers to override the template by placing a file at
    // `templates/verify.html` relative to the working directory.
    // Falls back to the compiled-in default when the file is absent.
    let template = match std::fs::read_to_string("templates/verify.html") {
        Ok(contents) => contents,
        Err(err) if err.kind() == ErrorKind::NotFound => VERIFY_EMAIL_DEFAULT.to_string(),
        Err(source) => return Err(MailTemplateError::ReadOverride { source }),
    };

    tera.add_raw_template("html_verify_email", &template)
        .map_err(|source| MailTemplateError::RegisterTemplate { source })?;

    tera.autoescape_on(vec![".html", ".sql"]);
    tera.register_filter("do_nothing", do_nothing_filter);
    Ok(tera)
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
    json_web_token: Arc<JsonWebToken>,
    mailer: Arc<Mailer>,
}

impl Service {
    pub async fn new(cfg: Arc<Configuration>, json_web_token: Arc<JsonWebToken>) -> Self {
        let mailer = Arc::new(Self::get_mailer(&cfg).await);

        Self {
            cfg,
            json_web_token,
            mailer,
        }
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
    pub async fn send_verification_mail(&self, to: &str, username: &str, user_id: i64, base_url: &str) -> Result<(), UserError> {
        let builder = self.get_builder(to).await;
        let verification_url = self.get_verification_url(user_id, base_url).await?;

        let mail = build_letter(verification_url.as_str(), username, builder)?;

        match self.mailer.send(mail).await {
            Ok(_res) => Ok(()),
            Err(e) => {
                error!(error = %e, "Failed to send email");
                Err(UserError::FailedToSendVerificationEmail)
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

    async fn get_verification_url(&self, user_id: i64, base_url: &str) -> Result<String, UserError> {
        let token = self
            .json_web_token
            .sign_email_verification(user_id)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        let settings = self.cfg.settings.read().await;
        let base_url = settings
            .net
            .base_url
            .as_ref()
            .map_or_else(|| base_url.to_string(), std::string::ToString::to_string);
        drop(settings);

        Ok(format!("{base_url}/{API_VERSION_URL_PREFIX}/user/email/verify/{token}"))
    }
}

pub(crate) fn build_letter(verification_url: &str, username: &str, builder: MessageBuilder) -> Result<Message, UserError> {
    let (plain_body, html_body) = build_content(verification_url, username).map_err(|error| {
        error!(%error, "failed to build verification email content");
        UserError::InternalServerError
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

pub(crate) fn build_content(verification_url: &str, username: &str) -> Result<(String, String), MailTemplateError> {
    let plain_body = format!(
        "
                Welcome to Torrust, {username}!

                Please click the confirmation link below to verify your account.
                {verification_url}

                If this account wasn't made by you, you can ignore this email.
            "
    );
    let mut context = Context::new();
    context.insert("verification", &verification_url);
    context.insert("username", &username);
    let templates = TEMPLATES.as_ref().map_err(|error| MailTemplateError::Initialize {
        message: error.to_string(),
    })?;
    let html_body = templates
        .render("html_verify_email", &context)
        .map_err(|source| MailTemplateError::Render { source })?;
    Ok((plain_body, html_body))
}

pub type Mailer = AsyncSmtpTransport<Tokio1Executor>;
