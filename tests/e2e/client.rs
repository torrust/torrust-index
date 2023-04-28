use reqwest::multipart;
use serde::Serialize;

use super::contexts::category::{AddCategoryForm, DeleteCategoryForm};
use super::contexts::settings::UpdateSettingsForm;
use super::contexts::torrent::requests::{TorrentId, UpdateTorrentFrom};
use super::contexts::user::{LoginForm, RegistrationForm, TokenRenewalForm, TokenVerificationForm, Username};
use super::responses::{self, BinaryResponse};
use crate::e2e::connection_info::ConnectionInfo;
use crate::e2e::http::{Query, ReqwestQuery};
use crate::e2e::responses::TextResponse;

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

    pub async fn about(&self) -> TextResponse {
        self.http_client.get("about", Query::empty()).await
    }

    pub async fn license(&self) -> TextResponse {
        self.http_client.get("about/license", Query::empty()).await
    }

    // Context: category

    pub async fn get_categories(&self) -> TextResponse {
        self.http_client.get("category", Query::empty()).await
    }

    pub async fn add_category(&self, add_category_form: AddCategoryForm) -> TextResponse {
        self.http_client.post("category", &add_category_form).await
    }

    pub async fn delete_category(&self, delete_category_form: DeleteCategoryForm) -> TextResponse {
        self.http_client.delete_with_body("category", &delete_category_form).await
    }

    // Context: root

    pub async fn root(&self) -> TextResponse {
        self.http_client.get("", Query::empty()).await
    }

    // Context: settings

    pub async fn get_public_settings(&self) -> TextResponse {
        self.http_client.get("settings/public", Query::empty()).await
    }

    pub async fn get_site_name(&self) -> TextResponse {
        self.http_client.get("settings/name", Query::empty()).await
    }

    pub async fn get_settings(&self) -> TextResponse {
        self.http_client.get("settings", Query::empty()).await
    }

    pub async fn update_settings(&self, update_settings_form: UpdateSettingsForm) -> TextResponse {
        self.http_client.post("settings", &update_settings_form).await
    }

    // Context: torrent

    pub async fn get_torrents(&self, params: Query) -> TextResponse {
        self.http_client.get("torrents", params).await
    }

    pub async fn get_torrent(&self, id: TorrentId) -> TextResponse {
        self.http_client.get(&format!("torrent/{id}"), Query::empty()).await
    }

    pub async fn delete_torrent(&self, id: TorrentId) -> TextResponse {
        self.http_client.delete(&format!("torrent/{id}")).await
    }

    pub async fn update_torrent(&self, id: TorrentId, update_torrent_form: UpdateTorrentFrom) -> TextResponse {
        self.http_client.put(&format!("torrent/{id}"), &update_torrent_form).await
    }

    pub async fn upload_torrent(&self, form: multipart::Form) -> TextResponse {
        self.http_client.post_multipart("torrent/upload", form).await
    }

    pub async fn download_torrent(&self, id: TorrentId) -> responses::BinaryResponse {
        self.http_client
            .get_binary(&format!("torrent/download/{id}"), Query::empty())
            .await
    }

    // Context: user

    pub async fn register_user(&self, registration_form: RegistrationForm) -> TextResponse {
        self.http_client.post("user/register", &registration_form).await
    }

    pub async fn login_user(&self, registration_form: LoginForm) -> TextResponse {
        self.http_client.post("user/login", &registration_form).await
    }

    pub async fn verify_token(&self, token_verification_form: TokenVerificationForm) -> TextResponse {
        self.http_client.post("user/token/verify", &token_verification_form).await
    }

    pub async fn renew_token(&self, token_verification_form: TokenRenewalForm) -> TextResponse {
        self.http_client.post("user/token/renew", &token_verification_form).await
    }

    pub async fn ban_user(&self, username: Username) -> TextResponse {
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

    pub async fn get(&self, path: &str, params: Query) -> TextResponse {
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
        TextResponse::from(response).await
    }

    pub async fn get_binary(&self, path: &str, params: Query) -> BinaryResponse {
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
        BinaryResponse::from(response).await
    }

    pub async fn post<T: Serialize + ?Sized>(&self, path: &str, form: &T) -> TextResponse {
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
        TextResponse::from(response).await
    }

    pub async fn post_multipart(&self, path: &str, form: multipart::Form) -> TextResponse {
        let response = match &self.connection_info.token {
            Some(token) => reqwest::Client::builder()
                .build()
                .unwrap()
                .post(self.base_url(path).clone())
                .multipart(form)
                .bearer_auth(token)
                .send()
                .await
                .unwrap(),
            None => reqwest::Client::builder()
                .build()
                .unwrap()
                .post(self.base_url(path).clone())
                .multipart(form)
                .send()
                .await
                .unwrap(),
        };
        TextResponse::from(response).await
    }

    pub async fn put<T: Serialize + ?Sized>(&self, path: &str, form: &T) -> TextResponse {
        let response = match &self.connection_info.token {
            Some(token) => reqwest::Client::new()
                .put(self.base_url(path).clone())
                .bearer_auth(token)
                .json(&form)
                .send()
                .await
                .unwrap(),
            None => reqwest::Client::new()
                .put(self.base_url(path).clone())
                .json(&form)
                .send()
                .await
                .unwrap(),
        };
        TextResponse::from(response).await
    }

    async fn delete(&self, path: &str) -> TextResponse {
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
        TextResponse::from(response).await
    }

    async fn delete_with_body<T: Serialize + ?Sized>(&self, path: &str, form: &T) -> TextResponse {
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
        TextResponse::from(response).await
    }

    fn base_url(&self, path: &str) -> String {
        format!("http://{}{}{path}", &self.connection_info.bind_address, &self.base_path)
    }
}
