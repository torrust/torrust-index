use std::sync::Arc;

use jsonwebtoken::{encode, EncodingKey, Header};
use lettre::message::{MessageBuilder, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};

use crate::config::Configuration;
use crate::errors::ServiceError;
use crate::utils::clock;
use crate::web::api::v1::routes::API_VERSION_URL_PREFIX;

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

#[derive(TemplateOnce)]
#[template(path = "../templates/verify.html")]
struct VerifyTemplate {
    username: String,
    verification_url: String,
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

    /// Send Verification Email
    ///
    /// # Errors
    ///
    /// This function will return an error if unable to send an email.
    pub async fn send_verification_mail(
        &self,
        to: &str,
        username: &str,
        user_id: i64,
        base_url: &str,
    ) -> Result<(), ServiceError> {
        let builder = self.get_builder(to).await;
        let verification_url = self.get_verification_url(user_id, base_url).await;

        let mail_body = format!(
            r#"
                Welcome to Torrust, {username}!

                Please click the confirmation link below to verify your account.
                {verification_url}

                If this account wasn't made by you, you can ignore this email.
            "#
        );

        let ctx = VerifyTemplate {
            username: String::from(username),
            verification_url,
        };

        let mail = builder
            .subject("Torrust - Email verification")
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_PLAIN)
                            .body(mail_body),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_HTML)
                            .body(
                                ctx.render_once()
                                    .expect("value `ctx` must have some internal error passed into it"),
                            ),
                    ),
            )
            .expect("the `multipart` builder had an error");

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
            .from(settings.mail.from.parse().unwrap())
            .reply_to(settings.mail.reply_to.parse().unwrap())
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

        let mut base_url = &base_url.to_string();
        if let Some(cfg_base_url) = &settings.net.base_url {
            base_url = cfg_base_url;
        }

        format!("{base_url}/{API_VERSION_URL_PREFIX}/user/email/verify/{token}")
    }
}

pub type Mailer = AsyncSmtpTransport<Tokio1Executor>;
