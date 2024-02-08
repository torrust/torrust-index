//! Action that a user can perform on a Index website.
use log::debug;
use thiserror::Error;

use crate::web::api::client::v1::client::Client;
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
}

/// It uploads a torrent file to the Torrust Index.
///
/// # Errors
///
/// It returns an error if the torrent already exists in the database.
///
/// # Panics
///
/// Panics if the response body is not a valid JSON.
pub async fn upload_torrent(client: &Client, upload_torrent_form: UploadTorrentMultipartForm) -> Result<UploadedTorrent, Error> {
    let categories = get_categories(client).await;

    if !contains_category_with_name(&categories, &upload_torrent_form.category) {
        add_category(client, &upload_torrent_form.category).await;
    }

    let response = client.upload_torrent(upload_torrent_form.into()).await;

    debug!(target:"seeder", "response: {}", response.status);

    if response.status == 400 {
        if response.body.contains("This torrent already exists in our database") {
            return Err(Error::TorrentInfoHashAlreadyExists);
        }

        if response.body.contains("This torrent title has already been used") {
            return Err(Error::TorrentTitleAlreadyExists);
        }
    }

    assert!(response.is_json_and_ok(), "Error uploading torrent: {}", response.body);

    let uploaded_torrent_response: UploadedTorrentResponse =
        serde_json::from_str(&response.body).expect("a valid JSON response should be returned from the Torrust Index API");

    Ok(uploaded_torrent_response.data)
}

/// It logs in the user and returns the user data.
///
/// # Panics
///
/// Panics if the response body is not a valid JSON.
pub async fn login(client: &Client, username: &str, password: &str) -> LoggedInUserData {
    let response = client
        .login_user(LoginForm {
            login: username.to_owned(),
            password: password.to_owned(),
        })
        .await;

    let res: SuccessfulLoginResponse = serde_json::from_str(&response.body).unwrap_or_else(|_| {
        panic!(
            "a valid JSON response should be returned after login. Received: {}",
            response.body
        )
    });

    res.data
}

/// It returns all the index categories.
///
/// # Panics
///
/// Panics if the response body is not a valid JSON.
pub async fn get_categories(client: &Client) -> Vec<ListItem> {
    let response = client.get_categories().await;

    let res: ListResponse = serde_json::from_str(&response.body).unwrap();

    res.data
}

/// It adds a new category.
pub async fn add_category(client: &Client, name: &str) -> TextResponse {
    client
        .add_category(AddCategoryForm {
            name: name.to_owned(),
            icon: None,
        })
        .await
}

/// It checks if the category list contains the given category.
fn contains_category_with_name(items: &[ListItem], category_name: &str) -> bool {
    items.iter().any(|item| item.name == category_name)
}
