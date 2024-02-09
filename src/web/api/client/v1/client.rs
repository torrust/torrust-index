use reqwest::{multipart, Url};

use super::connection_info::ConnectionInfo;
use super::contexts::category::forms::{AddCategoryForm, DeleteCategoryForm};
use super::contexts::tag::forms::{AddTagForm, DeleteTagForm};
use super::contexts::torrent::forms::UpdateTorrentForm;
use super::contexts::torrent::requests::InfoHash;
use super::contexts::user::forms::{LoginForm, RegistrationForm, TokenRenewalForm, TokenVerificationForm, Username};
use super::http::{Http, Query};
use super::responses::{self, TextResponse};

#[derive(Debug)]
pub enum Error {
    HttpError(reqwest::Error),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::HttpError(err)
    }
}

/// API Client
pub struct Client {
    http_client: Http,
}

impl Client {
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

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn about(&self) -> Result<TextResponse, Error> {
        self.http_client.get("/about", Query::empty()).await.map_err(Error::from)
    }

    /// # Errors
    ///
    /// Will panic if the request fails.  
    pub async fn license(&self) -> Result<TextResponse, Error> {
        self.http_client
            .get("/about/license", Query::empty())
            .await
            .map_err(Error::from)
    }

    // Context: category

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn get_categories(&self) -> Result<TextResponse, Error> {
        self.http_client.get("/category", Query::empty()).await.map_err(Error::from)
    }

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn add_category(&self, add_category_form: AddCategoryForm) -> Result<TextResponse, Error> {
        self.http_client
            .post("/category", &add_category_form)
            .await
            .map_err(Error::from)
    }

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn delete_category(&self, delete_category_form: DeleteCategoryForm) -> Result<TextResponse, Error> {
        self.http_client
            .delete_with_body("/category", &delete_category_form)
            .await
            .map_err(Error::from)
    }

    // Context: tag

    /// # Errors
    ///
    /// Will panic if the request fails.  
    pub async fn get_tags(&self) -> Result<TextResponse, Error> {
        self.http_client.get("/tags", Query::empty()).await.map_err(Error::from)
    }

    /// # Errors
    ///
    /// Will panic if the request fails.   
    pub async fn add_tag(&self, add_tag_form: AddTagForm) -> Result<TextResponse, Error> {
        self.http_client.post("/tag", &add_tag_form).await.map_err(Error::from)
    }

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn delete_tag(&self, delete_tag_form: DeleteTagForm) -> Result<TextResponse, Error> {
        self.http_client
            .delete_with_body("/tag", &delete_tag_form)
            .await
            .map_err(Error::from)
    }

    // Context: root

    /// # Errors
    ///
    /// Will panic if the request fails.   
    pub async fn root(&self) -> Result<TextResponse, Error> {
        self.http_client.get("", Query::empty()).await.map_err(Error::from)
    }

    // Context: settings

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn get_public_settings(&self) -> Result<TextResponse, Error> {
        self.http_client
            .get("/settings/public", Query::empty())
            .await
            .map_err(Error::from)
    }

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn get_site_name(&self) -> Result<TextResponse, Error> {
        self.http_client
            .get("/settings/name", Query::empty())
            .await
            .map_err(Error::from)
    }

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn get_settings(&self) -> Result<TextResponse, Error> {
        self.http_client.get("/settings", Query::empty()).await.map_err(Error::from)
    }

    // Context: torrent

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn get_torrents(&self, params: Query) -> Result<TextResponse, Error> {
        self.http_client.get("/torrents", params).await.map_err(Error::from)
    }

    /// # Errors
    ///
    /// Will panic if the request fails.  
    pub async fn get_torrent(&self, info_hash: &InfoHash) -> Result<TextResponse, Error> {
        self.http_client
            .get(&format!("/torrent/{info_hash}"), Query::empty())
            .await
            .map_err(Error::from)
    }

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn delete_torrent(&self, info_hash: &InfoHash) -> Result<TextResponse, Error> {
        self.http_client
            .delete(&format!("/torrent/{info_hash}"))
            .await
            .map_err(Error::from)
    }

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn update_torrent(
        &self,
        info_hash: &InfoHash,
        update_torrent_form: UpdateTorrentForm,
    ) -> Result<TextResponse, Error> {
        self.http_client
            .put(&format!("/torrent/{info_hash}"), &update_torrent_form)
            .await
            .map_err(Error::from)
    }

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn upload_torrent(&self, form: multipart::Form) -> Result<TextResponse, Error> {
        self.http_client
            .post_multipart("/torrent/upload", form)
            .await
            .map_err(Error::from)
    }

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn download_torrent(&self, info_hash: &InfoHash) -> Result<responses::BinaryResponse, Error> {
        self.http_client
            .get_binary(&format!("/torrent/download/{info_hash}"), Query::empty())
            .await
            .map_err(Error::from)
    }

    // Context: user

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn register_user(&self, registration_form: RegistrationForm) -> Result<TextResponse, Error> {
        self.http_client
            .post("/user/register", &registration_form)
            .await
            .map_err(Error::from)
    }

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn login_user(&self, registration_form: LoginForm) -> Result<TextResponse, Error> {
        self.http_client
            .post("/user/login", &registration_form)
            .await
            .map_err(Error::from)
    }

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn verify_token(&self, token_verification_form: TokenVerificationForm) -> Result<TextResponse, Error> {
        self.http_client
            .post("/user/token/verify", &token_verification_form)
            .await
            .map_err(Error::from)
    }

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn renew_token(&self, token_verification_form: TokenRenewalForm) -> Result<TextResponse, Error> {
        self.http_client
            .post("/user/token/renew", &token_verification_form)
            .await
            .map_err(Error::from)
    }

    /// # Errors
    ///
    /// Will panic if the request fails.
    pub async fn ban_user(&self, username: Username) -> Result<TextResponse, Error> {
        self.http_client
            .delete(&format!("/user/ban/{}", &username.value))
            .await
            .map_err(Error::from)
    }
}
