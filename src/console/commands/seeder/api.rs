//! Action that a user can perform on a Index website.
use thiserror::Error;
use tracing::debug;

use crate::web::api::client::v1::client::{Client, Error as ClientError};
use crate::web::api::client::v1::contexts::category::forms::AddCategoryForm;
use crate::web::api::client::v1::contexts::category::responses::{ListItem, ListResponse};
use crate::web::api::client::v1::contexts::torrent::forms::UploadTorrentMultipartForm;
use crate::web::api::client::v1::contexts::torrent::responses::{UploadedTorrent, UploadedTorrentResponse};
use crate::web::api::client::v1::contexts::user::forms::LoginForm;
use crate::web::api::client::v1::contexts::user::responses::{LoggedInUserData, SuccessfulLoginResponse};
use crate::web::api::client::v1::responses::TextResponse;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Torrent with the same info-hash already exist in the database")]
    TorrentInfoHashAlreadyExists,
    #[error("Torrent with the same title already exist in the database")]
    TorrentTitleAlreadyExists,
    #[error("failed to fetch categories: {error:?}")]
    FetchCategories { error: ClientError },
    #[error("category list API returned an unexpected response: status {status}, content type {content_type:?}")]
    UnexpectedCategoryListResponse { status: u16, content_type: Option<String> },
    #[error("failed to parse category list response: {source}")]
    ParseCategoryListResponse { source: serde_json::Error },
    #[error("failed to add category: {error:?}")]
    AddCategory { error: ClientError },
    #[error("add category API returned an unexpected response: status {status}, content type {content_type:?}")]
    UnexpectedAddCategoryResponse { status: u16, content_type: Option<String> },
    #[error("failed to build upload multipart form: {source}")]
    BuildUploadForm { source: reqwest::Error },
    #[error("failed to upload torrent: {error:?}")]
    UploadTorrent { error: ClientError },
    #[error("upload torrent API returned an unexpected response: status {status}, content type {content_type:?}")]
    UnexpectedUploadTorrentResponse { status: u16, content_type: Option<String> },
    #[error("failed to parse upload torrent response: {source}")]
    ParseUploadTorrentResponse { source: serde_json::Error },
    #[error("failed to login: {error:?}")]
    Login { error: ClientError },
    #[error("login API returned an unexpected response: status {status}, content type {content_type:?}")]
    UnexpectedLoginResponse { status: u16, content_type: Option<String> },
    #[error("failed to parse login response: {source}")]
    ParseLoginResponse { source: serde_json::Error },
}

/// It uploads a torrent file to the Torrust Index.
///
/// # Errors
///
/// It returns an error if the torrent already exists in the database or if an
/// API response cannot be handled.
pub async fn upload_torrent(client: &Client, upload_torrent_form: UploadTorrentMultipartForm) -> Result<UploadedTorrent, Error> {
    let categories = get_categories(client).await?;

    if !contains_category_with_name(&categories, &upload_torrent_form.category) {
        add_category(client, &upload_torrent_form.category).await?;
    }

    // todo: if we receive timeout error we should retry later. Otherwise we
    // have to restart the seeder manually.

    let form = upload_torrent_form
        .try_into()
        .map_err(|source| Error::BuildUploadForm { source })?;
    let response = client
        .upload_torrent(form)
        .await
        .map_err(|error| Error::UploadTorrent { error })?;

    debug!(target:"seeder", "response: {}", response.status);

    if response.status == 400 {
        if response.body.contains("This torrent already exists in our database") {
            return Err(Error::TorrentInfoHashAlreadyExists);
        }

        if response.body.contains("This torrent title has already been used") {
            return Err(Error::TorrentTitleAlreadyExists);
        }
    }

    if !response.is_json_and_ok() {
        return Err(Error::UnexpectedUploadTorrentResponse {
            status: response.status,
            content_type: response.content_type,
        });
    }

    let uploaded_torrent_response: UploadedTorrentResponse =
        serde_json::from_str(&response.body).map_err(|source| Error::ParseUploadTorrentResponse { source })?;

    Ok(uploaded_torrent_response.data)
}

/// It logs in the user and returns the user data.
///
/// # Errors
///
/// Returns an error if the API request fails or the response cannot be handled.
pub async fn login(client: &Client, username: &str, password: &str) -> Result<LoggedInUserData, Error> {
    let response = client
        .login_user(LoginForm {
            login: username.to_owned(),
            password: password.to_owned(),
        })
        .await
        .map_err(|error| Error::Login { error })?;

    if !response.is_json_and_ok() {
        return Err(Error::UnexpectedLoginResponse {
            status: response.status,
            content_type: response.content_type,
        });
    }

    let res: SuccessfulLoginResponse =
        serde_json::from_str(&response.body).map_err(|source| Error::ParseLoginResponse { source })?;

    Ok(res.data)
}

/// It returns all the index categories.
///
/// # Errors
///
/// Returns an error if the API request fails or the response cannot be handled.
pub async fn get_categories(client: &Client) -> Result<Vec<ListItem>, Error> {
    let response = client
        .get_categories()
        .await
        .map_err(|error| Error::FetchCategories { error })?;

    if !response.is_json_and_ok() {
        return Err(Error::UnexpectedCategoryListResponse {
            status: response.status,
            content_type: response.content_type,
        });
    }

    let res: ListResponse = serde_json::from_str(&response.body).map_err(|source| Error::ParseCategoryListResponse { source })?;

    Ok(res.data)
}

/// It adds a new category.
///
/// # Errors
///
/// Returns an error if the API request fails or the response cannot be handled.
pub async fn add_category(client: &Client, name: &str) -> Result<TextResponse, Error> {
    let response = client
        .add_category(AddCategoryForm {
            name: name.to_owned(),
            icon: None,
        })
        .await
        .map_err(|error| Error::AddCategory { error })?;

    if !response.is_json_and_ok() {
        return Err(Error::UnexpectedAddCategoryResponse {
            status: response.status,
            content_type: response.content_type,
        });
    }

    Ok(response)
}

/// It checks if the category list contains the given category.
fn contains_category_with_name(items: &[ListItem], category_name: &str) -> bool {
    items.iter().any(|item| item.name == category_name)
}
