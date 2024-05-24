use std::collections::HashMap;
use std::sync::Arc;

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

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();

        match tera.add_template_file("templates/verify.html", Some("html_verify_email")) {
            Ok(()) => {}
            Err(e) => {
                println!("Parsing error(s): {e}");
                ::std::process::exit(1);
            }
        };

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

impl Service {
    pub async fn new(cfg: Arc<Configuration>) -> Service {
        let mailer = Arc::new(Self::get_mailer(&cfg).await);

        Self { cfg, mailer }
    }

    async fn get_mailer(cfg: &Configuration) -> Mailer {
        let settings = cfg.settings.read().await;

        if !settings.mail.username.is_empty() && !settings.mail.password.is_empty() {
            // SMTP authentication
            let creds = Credentials::new(settings.mail.username.clone(), settings.mail.password.clone());

            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&settings.mail.server)
                .port(settings.mail.port)
                .credentials(creds)
                .authentication(vec![Mechanism::Login, Mechanism::Xoauth2, Mechanism::Plain])
                .build()
        } else {
            // SMTP without authentication
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&settings.mail.server)
                .port(settings.mail.port)
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

        let mail = build_letter(verification_url.as_str(), username, builder)?;

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
        let key = settings.auth.secret_key.as_bytes();

        // Create non expiring token that is only valid for email-verification
        let claims = VerifyClaims {
            iss: String::from("email-verification"),
            sub: user_id,
            exp: clock::now() + 315_569_260, // 10 years from now
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(key)).unwrap();

        let base_url = match &settings.net.base_url {
            Some(url) => url.to_string(),
            None => base_url.to_string(),
        };

        format!("{base_url}/{API_VERSION_URL_PREFIX}/user/email/verify/{token}")
    }
}

fn build_letter(verification_url: &str, username: &str, builder: MessageBuilder) -> Result<Message, ServiceError> {
    let (plain_body, html_body) = build_content(verification_url, username).map_err(|e| {
        log::error!("{e}");
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

fn build_content(verification_url: &str, username: &str) -> Result<(String, String), tera::Error> {
    let plain_body = format!(
        r#"
                Welcome to Torrust, {username}!

                Please click the confirmation link below to verify your account.
                {verification_url}

                If this account wasn't made by you, you can ignore this email.
            "#
    );
    let mut context = Context::new();
    context.insert("verification", &verification_url);
    context.insert("username", &username);
    let html_body = TEMPLATES.render("html_verify_email", &context)?;
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

        let _letter = build_letter("https://a.b.c/", "user", builder).unwrap();
    }

    #[test]
    fn it_should_build_content() {
        let (plain_body, html_body) = build_content("https://a.b.c/", "user").unwrap();
        assert_ne!(plain_body, "");
        assert_ne!(html_body, "");
    }
}
