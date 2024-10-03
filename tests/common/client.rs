use std::time::Duration;

use reqwest::{multipart, Error};
use serde::Serialize;

use super::connection_info::ConnectionInfo;
use super::contexts::category::forms::{AddCategoryForm, DeleteCategoryForm};
use super::contexts::tag::forms::{AddTagForm, DeleteTagForm};
use super::contexts::torrent::forms::UpdateTorrentFrom;
use super::contexts::torrent::requests::InfoHash;
use super::contexts::user::forms::{
    ChangePasswordForm, LoginForm, RegistrationForm, TokenRenewalForm, TokenVerificationForm, Username,
};
use super::http::{Query, ReqwestQuery};
use super::responses::{self, BinaryResponse, TextResponse};

/// API Client
pub struct Client {
    http_client: Http,
}

impl Client {
    // todo: forms in POST requests can be passed by reference.

    fn base_path() -> String {
        "v1".to_string()
    }

    /// Remove last '/' char in the address if present.
    ///
    /// For example: <https://localhost/> to <https://localhost/>.
    fn base_url(bind_address: &str) -> String {
        let mut url = bind_address.to_owned();
        if url.ends_with('/') {
            url.pop();
        }
        url
    }

    pub fn unauthenticated(bind_address: &str) -> Self {
        Self::new(ConnectionInfo::anonymous(&Self::base_url(bind_address), &Self::base_path()))
    }

    pub fn authenticated(bind_address: &str, token: &str) -> Self {
        Self::new(ConnectionInfo::new(&Self::base_url(bind_address), &Self::base_path(), token))
    }

    pub fn new(connection_info: ConnectionInfo) -> Self {
        Self {
            http_client: Http::new(connection_info),
        }
    }

    /// It checks if the server is running.
    pub async fn server_is_running(&self) -> Result<(), Error> {
        let response = self.http_client.inner_get("").await;
        match response {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    // Context: about

    pub async fn about(&self) -> TextResponse {
        self.http_client.get("/about", Query::empty()).await
    }

    pub async fn license(&self) -> TextResponse {
        self.http_client.get("/about/license", Query::empty()).await
    }

    // Context: category

    pub async fn get_categories(&self) -> TextResponse {
        self.http_client.get("/category", Query::empty()).await
    }

    pub async fn add_category(&self, add_category_form: AddCategoryForm) -> TextResponse {
        self.http_client.post("/category", &add_category_form).await
    }

    pub async fn delete_category(&self, delete_category_form: DeleteCategoryForm) -> TextResponse {
        self.http_client.delete_with_body("/category", &delete_category_form).await
    }

    // Context: tag

    pub async fn get_tags(&self) -> TextResponse {
        // code-review: some endpoint are using plural
        // (for instance, `get_categories`) and some singular.
        self.http_client.get("/tags", Query::empty()).await
    }

    pub async fn add_tag(&self, add_tag_form: AddTagForm) -> TextResponse {
        self.http_client.post("/tag", &add_tag_form).await
    }

    pub async fn delete_tag(&self, delete_tag_form: DeleteTagForm) -> TextResponse {
        self.http_client.delete_with_body("/tag", &delete_tag_form).await
    }

    // Context: root

    pub async fn root(&self) -> TextResponse {
        self.http_client.get("", Query::empty()).await
    }

    // Context: settings

    pub async fn get_public_settings(&self) -> TextResponse {
        self.http_client.get("/settings/public", Query::empty()).await
    }

    pub async fn get_site_name(&self) -> TextResponse {
        self.http_client.get("/settings/name", Query::empty()).await
    }

    pub async fn get_settings(&self) -> TextResponse {
        self.http_client.get("/settings", Query::empty()).await
    }

    // Context: torrent

    pub async fn get_torrents(&self, params: Query) -> TextResponse {
        self.http_client.get("/torrents", params).await
    }

    pub async fn get_torrent(&self, info_hash: &InfoHash) -> TextResponse {
        self.http_client.get(&format!("/torrent/{info_hash}"), Query::empty()).await
    }

    pub async fn delete_torrent(&self, info_hash: &InfoHash) -> TextResponse {
        self.http_client.delete(&format!("/torrent/{info_hash}")).await
    }

    pub async fn update_torrent(&self, info_hash: &InfoHash, update_torrent_form: UpdateTorrentFrom) -> TextResponse {
        self.http_client
            .put(&format!("/torrent/{info_hash}"), &update_torrent_form)
            .await
    }

    pub async fn upload_torrent(&self, form: multipart::Form) -> TextResponse {
        self.http_client.post_multipart("/torrent/upload", form).await
    }

    pub async fn download_torrent(&self, info_hash: &InfoHash) -> responses::BinaryResponse {
        self.http_client
            .get_binary(&format!("/torrent/download/{info_hash}"), Query::empty())
            .await
    }

    // Context: user

    pub async fn register_user(&self, registration_form: RegistrationForm) -> TextResponse {
        self.http_client.post("/user/register", &registration_form).await
    }

    pub async fn login_user(&self, registration_form: LoginForm) -> TextResponse {
        self.http_client.post("/user/login", &registration_form).await
    }

    pub async fn change_password(&self, username: Username, change_password_form: ChangePasswordForm) -> TextResponse {
        self.http_client
            .post(&format!("/user/{}/change-password", &username.value), &change_password_form)
            .await
    }

    pub async fn verify_token(&self, token_verification_form: TokenVerificationForm) -> TextResponse {
        self.http_client.post("/user/token/verify", &token_verification_form).await
    }

    pub async fn renew_token(&self, token_verification_form: TokenRenewalForm) -> TextResponse {
        self.http_client.post("/user/token/renew", &token_verification_form).await
    }

    pub async fn ban_user(&self, username: Username) -> TextResponse {
        self.http_client.delete(&format!("/user/ban/{}", &username.value)).await
    }

    // Context: proxy

    /* pub async fn get_image_by_url(&self, url: &str) -> TextResponse {
        self.http_client.get(&format!("/proxy/image/{url}"), Query::empty()).await
    } */
}

/// Generic HTTP Client
struct Http {
    connection_info: ConnectionInfo,
    /// The timeout is applied from when the request starts connecting until the
    /// response body has finished.
    timeout: Duration,
}

impl Http {
    pub fn new(connection_info: ConnectionInfo) -> Self {
        Self {
            connection_info,
            timeout: Duration::from_secs(5),
        }
    }

    pub async fn get(&self, path: &str, params: Query) -> TextResponse {
        let response = match &self.connection_info.token {
            Some(token) => reqwest::Client::builder()
                .timeout(self.timeout)
                .build()
                .unwrap()
                .get(self.base_url(path).clone())
                .query(&ReqwestQuery::from(params))
                .bearer_auth(token)
                .send()
                .await
                .unwrap(),
            None => reqwest::Client::builder()
                .timeout(self.timeout)
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
                .timeout(self.timeout)
                .build()
                .unwrap()
                .get(self.base_url(path).clone())
                .query(&ReqwestQuery::from(params))
                .bearer_auth(token)
                .send()
                .await
                .unwrap(),
            None => reqwest::Client::builder()
                .timeout(self.timeout)
                .build()
                .unwrap()
                .get(self.base_url(path).clone())
                .query(&ReqwestQuery::from(params))
                .send()
                .await
                .unwrap(),
        };
        // todo: If the response is a JSON, it returns the JSON body in a byte
        //   array. This is not the expected behavior.
        //  - Rename BinaryResponse to BinaryTorrentResponse
        //  - Return an error if the response is not a bittorrent file
        BinaryResponse::from(response).await
    }

    pub async fn inner_get(&self, path: &str) -> Result<reqwest::Response, reqwest::Error> {
        reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .unwrap()
            .get(self.base_url(path).clone())
            .send()
            .await
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
                .timeout(self.timeout)
                .build()
                .unwrap()
                .post(self.base_url(path).clone())
                .multipart(form)
                .bearer_auth(token)
                .send()
                .await
                .expect("failed to send multipart request with token"),
            None => reqwest::Client::builder()
                .timeout(self.timeout)
                .build()
                .unwrap()
                .post(self.base_url(path).clone())
                .multipart(form)
                .send()
                .await
                .expect("failed to send multipart request without token"),
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
        format!(
            "http://{}/{}{path}", // DevSkim: ignore DS137138
            &self.connection_info.bind_address, &self.connection_info.base_path
        )
    }
}
