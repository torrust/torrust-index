use reqwest::{multipart, Url};

use super::connection_info::ConnectionInfo;
use super::contexts::category::forms::{AddCategoryForm, DeleteCategoryForm};
use super::contexts::tag::forms::{AddTagForm, DeleteTagForm};
use super::contexts::torrent::forms::UpdateTorrentForm;
use super::contexts::torrent::requests::InfoHash;
use super::contexts::user::forms::{LoginForm, RegistrationForm, TokenRenewalForm, TokenVerificationForm, Username};
use super::http::{Http, Query};
use super::responses::{self, TextResponse};

/// API Client
pub struct Client {
    http_client: Http,
}

impl Client {
    // todo: forms in POST requests can be passed by reference.

    fn base_path() -> String {
        "/v1".to_string()
    }

    #[must_use]
    pub fn unauthenticated(base_url: &Url) -> Self {
        Self::new(ConnectionInfo::anonymous(base_url, &Self::base_path()))
    }

    #[must_use]
    pub fn authenticated(base_url: &Url, token: &str) -> Self {
        Self::new(ConnectionInfo::new(base_url, &Self::base_path(), token))
    }

    #[must_use]
    pub fn new(connection_info: ConnectionInfo) -> Self {
        Self {
            http_client: Http::new(connection_info),
        }
    }

    /// It checks if the server is running.
    pub async fn server_is_running(&self) -> bool {
        let response = self.http_client.inner_get("").await;
        response.is_ok()
    }

    // Context: about

    /// # Panics
    ///
    /// Will panic if the request fails.
    pub async fn about(&self) -> TextResponse {
        self.http_client.get("/about", Query::empty()).await.unwrap()
    }

    /// # Panics
    ///
    /// Will panic if the request fails.    
    pub async fn license(&self) -> TextResponse {
        self.http_client.get("/about/license", Query::empty()).await.unwrap()
    }

    // Context: category

    /// # Panics
    ///
    /// Will panic if the request fails.
    pub async fn get_categories(&self) -> TextResponse {
        self.http_client.get("/category", Query::empty()).await.unwrap()
    }

    /// # Panics
    ///
    /// Will panic if the request fails.
    pub async fn add_category(&self, add_category_form: AddCategoryForm) -> TextResponse {
        self.http_client.post("/category", &add_category_form).await.unwrap()
    }

    /// # Panics
    ///
    /// Will panic if the request fails.
    pub async fn delete_category(&self, delete_category_form: DeleteCategoryForm) -> TextResponse {
        self.http_client
            .delete_with_body("/category", &delete_category_form)
            .await
            .unwrap()
    }

    // Context: tag

    /// # Panics
    ///
    /// Will panic if the request fails.    
    pub async fn get_tags(&self) -> TextResponse {
        // code-review: some endpoint are using plural
        // (for instance, `get_categories`) and some singular.
        self.http_client.get("/tags", Query::empty()).await.unwrap()
    }

    /// # Panics
    ///
    /// Will panic if the request fails.       
    pub async fn add_tag(&self, add_tag_form: AddTagForm) -> TextResponse {
        self.http_client.post("/tag", &add_tag_form).await.unwrap()
    }

    /// # Panics
    ///
    /// Will panic if the request fails.   
    pub async fn delete_tag(&self, delete_tag_form: DeleteTagForm) -> TextResponse {
        self.http_client.delete_with_body("/tag", &delete_tag_form).await.unwrap()
    }

    // Context: root

    /// # Panics
    ///
    /// Will panic if the request fails.    
    pub async fn root(&self) -> TextResponse {
        self.http_client.get("", Query::empty()).await.unwrap()
    }

    // Context: settings

    /// # Panics
    ///
    /// Will panic if the request fails.    
    pub async fn get_public_settings(&self) -> TextResponse {
        self.http_client.get("/settings/public", Query::empty()).await.unwrap()
    }

    /// # Panics
    ///
    /// Will panic if the request fails.
    pub async fn get_site_name(&self) -> TextResponse {
        self.http_client.get("/settings/name", Query::empty()).await.unwrap()
    }

    /// # Panics
    ///
    /// Will panic if the request fails.
    pub async fn get_settings(&self) -> TextResponse {
        self.http_client.get("/settings", Query::empty()).await.unwrap()
    }

    // Context: torrent

    /// # Panics
    ///
    /// Will panic if the request fails.
    pub async fn get_torrents(&self, params: Query) -> TextResponse {
        self.http_client.get("/torrents", params).await.unwrap()
    }

    /// # Panics
    ///
    /// Will panic if the request fails.    
    pub async fn get_torrent(&self, info_hash: &InfoHash) -> TextResponse {
        self.http_client
            .get(&format!("/torrent/{info_hash}"), Query::empty())
            .await
            .unwrap()
    }

    /// # Panics
    ///
    /// Will panic if the request fails.   
    pub async fn delete_torrent(&self, info_hash: &InfoHash) -> TextResponse {
        self.http_client.delete(&format!("/torrent/{info_hash}")).await.unwrap()
    }

    /// # Panics
    ///
    /// Will panic if the request fails.   
    pub async fn update_torrent(&self, info_hash: &InfoHash, update_torrent_form: UpdateTorrentForm) -> TextResponse {
        self.http_client
            .put(&format!("/torrent/{info_hash}"), &update_torrent_form)
            .await
            .unwrap()
    }

    /// # Panics
    ///
    /// Will panic if the request fails.   
    pub async fn upload_torrent(&self, form: multipart::Form) -> TextResponse {
        self.http_client.post_multipart("/torrent/upload", form).await.unwrap()
    }

    /// # Panics
    ///
    /// Will panic if the request fails.   
    pub async fn download_torrent(&self, info_hash: &InfoHash) -> responses::BinaryResponse {
        self.http_client
            .get_binary(&format!("/torrent/download/{info_hash}"), Query::empty())
            .await
            .unwrap()
    }

    // Context: user

    /// # Panics
    ///
    /// Will panic if the request fails.   
    pub async fn register_user(&self, registration_form: RegistrationForm) -> TextResponse {
        self.http_client.post("/user/register", &registration_form).await.unwrap()
    }

    /// # Panics
    ///
    /// Will panic if the request fails.   
    pub async fn login_user(&self, registration_form: LoginForm) -> TextResponse {
        self.http_client.post("/user/login", &registration_form).await.unwrap()
    }

    /// # Panics
    ///
    /// Will panic if the request fails.   
    pub async fn verify_token(&self, token_verification_form: TokenVerificationForm) -> TextResponse {
        self.http_client
            .post("/user/token/verify", &token_verification_form)
            .await
            .unwrap()
    }

    /// # Panics
    ///
    /// Will panic if the request fails.   
    pub async fn renew_token(&self, token_verification_form: TokenRenewalForm) -> TextResponse {
        self.http_client
            .post("/user/token/renew", &token_verification_form)
            .await
            .unwrap()
    }

    /// # Panics
    ///
    /// Will panic if the request fails.   
    pub async fn ban_user(&self, username: Username) -> TextResponse {
        self.http_client
            .delete(&format!("/user/ban/{}", &username.value))
            .await
            .unwrap()
    }
}
