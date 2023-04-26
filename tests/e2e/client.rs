use serde::Serialize;

use super::contexts::category::{AddCategoryForm, DeleteCategoryForm};
use super::contexts::settings::UpdateSettingsForm;
use super::contexts::user::{LoginForm, RegistrationForm, TokenRenewalForm, TokenVerificationForm, Username};
use crate::e2e::connection_info::ConnectionInfo;
use crate::e2e::http::{Query, ReqwestQuery};
use crate::e2e::response::Response;

/// API Client
pub struct Client {
    http_client: Http,
}

impl Client {
    pub fn new(connection_info: ConnectionInfo) -> Self {
        Self {
            http_client: Http::new(connection_info),
        }
    }

    // Context: about

    pub async fn about(&self) -> Response {
        self.http_client.get("about", Query::empty()).await
    }

    pub async fn license(&self) -> Response {
        self.http_client.get("about/license", Query::empty()).await
    }

    // Context: category

    pub async fn get_categories(&self) -> Response {
        self.http_client.get("category", Query::empty()).await
    }

    pub async fn add_category(&self, add_category_form: AddCategoryForm) -> Response {
        self.http_client.post("category", &add_category_form).await
    }

    pub async fn delete_category(&self, delete_category_form: DeleteCategoryForm) -> Response {
        self.http_client.delete_with_body("category", &delete_category_form).await
    }

    // Context: root

    pub async fn root(&self) -> Response {
        self.http_client.get("", Query::empty()).await
    }

    // Context: settings

    pub async fn get_public_settings(&self) -> Response {
        self.http_client.get("settings/public", Query::empty()).await
    }

    pub async fn get_site_name(&self) -> Response {
        self.http_client.get("settings/name", Query::empty()).await
    }

    pub async fn get_settings(&self) -> Response {
        self.http_client.get("settings", Query::empty()).await
    }

    pub async fn update_settings(&self, update_settings_form: UpdateSettingsForm) -> Response {
        self.http_client.post("settings", &update_settings_form).await
    }

    // Context: user

    pub async fn register_user(&self, registration_form: RegistrationForm) -> Response {
        self.http_client.post("user/register", &registration_form).await
    }

    pub async fn login_user(&self, registration_form: LoginForm) -> Response {
        self.http_client.post("user/login", &registration_form).await
    }

    pub async fn verify_token(&self, token_verification_form: TokenVerificationForm) -> Response {
        self.http_client.post("user/token/verify", &token_verification_form).await
    }

    pub async fn renew_token(&self, token_verification_form: TokenRenewalForm) -> Response {
        self.http_client.post("user/token/renew", &token_verification_form).await
    }

    pub async fn ban_user(&self, username: Username) -> Response {
        self.http_client.delete(&format!("user/ban/{}", &username.value)).await
    }
}

/// Generic HTTP Client
struct Http {
    connection_info: ConnectionInfo,
    base_path: String,
}

impl Http {
    pub fn new(connection_info: ConnectionInfo) -> Self {
        Self {
            connection_info,
            base_path: "/".to_string(),
        }
    }

    pub async fn get(&self, path: &str, params: Query) -> Response {
        let response = match &self.connection_info.token {
            Some(token) => reqwest::Client::builder()
                .build()
                .unwrap()
                .get(self.base_url(path).clone())
                .query(&ReqwestQuery::from(params))
                .bearer_auth(token)
                .send()
                .await
                .unwrap(),
            None => reqwest::Client::builder()
                .build()
                .unwrap()
                .get(self.base_url(path).clone())
                .query(&ReqwestQuery::from(params))
                .send()
                .await
                .unwrap(),
        };
        Response::from(response).await
    }

    pub async fn post<T: Serialize + ?Sized>(&self, path: &str, form: &T) -> Response {
        let response = match &self.connection_info.token {
            Some(token) => reqwest::Client::new()
                .post(self.base_url(path).clone())
                .bearer_auth(token)
                .json(&form)
                .send()
                .await
                .unwrap(),
            None => reqwest::Client::new()
                .post(self.base_url(path).clone())
                .json(&form)
                .send()
                .await
                .unwrap(),
        };
        Response::from(response).await
    }

    async fn delete(&self, path: &str) -> Response {
        let response = match &self.connection_info.token {
            Some(token) => reqwest::Client::new()
                .delete(self.base_url(path).clone())
                .bearer_auth(token)
                .send()
                .await
                .unwrap(),
            None => reqwest::Client::new()
                .delete(self.base_url(path).clone())
                .send()
                .await
                .unwrap(),
        };
        Response::from(response).await
    }

    async fn delete_with_body<T: Serialize + ?Sized>(&self, path: &str, form: &T) -> Response {
        let response = match &self.connection_info.token {
            Some(token) => reqwest::Client::new()
                .delete(self.base_url(path).clone())
                .bearer_auth(token)
                .json(&form)
                .send()
                .await
                .unwrap(),
            None => reqwest::Client::new()
                .delete(self.base_url(path).clone())
                .json(&form)
                .send()
                .await
                .unwrap(),
        };
        Response::from(response).await
    }

    fn base_url(&self, path: &str) -> String {
        format!("http://{}{}{path}", &self.connection_info.bind_address, &self.base_path)
    }
}
