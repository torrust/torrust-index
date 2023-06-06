//! API contract for `category` context.
use torrust_index_backend::web::api;

use crate::common::asserts::assert_json_ok;
use crate::common::client::Client;
use crate::common::contexts::category::fixtures::random_category_name;
use crate::common::contexts::category::forms::{AddCategoryForm, DeleteCategoryForm};
use crate::common::contexts::category::responses::{AddedCategoryResponse, ListResponse};
use crate::e2e::contexts::category::steps::add_category;
use crate::e2e::contexts::user::steps::{new_logged_in_admin, new_logged_in_user};
use crate::e2e::environment::TestEnv;

/* todo:
    - it should allow adding a new category to authenticated clients
    - it should not allow adding a new category with an empty name
    - it should allow adding a new category with an optional icon
    - ...
*/

#[tokio::test]
async fn it_should_return_an_empty_category_list_when_there_are_no_categories() {
    let mut env = TestEnv::new();
    env.start(api::Implementation::ActixWeb).await;
    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    let response = client.get_categories().await;

    assert_json_ok(&response);
}

#[tokio::test]
async fn it_should_return_a_category_list() {
    let mut env = TestEnv::new();
    env.start(api::Implementation::ActixWeb).await;
    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    // Add a category
    let category_name = random_category_name();
    let response = add_category(&category_name, &env).await;
    assert_eq!(response.status, 200);

    let response = client.get_categories().await;

    let res: ListResponse = serde_json::from_str(&response.body).unwrap();

    // There should be at least the category we added.
    // Since this is an E2E test, there might be more categories.
    assert!(res.data.len() > 1);
    if let Some(content_type) = &response.content_type {
        assert_eq!(content_type, "application/json");
    }
    assert_eq!(response.status, 200);
}

#[tokio::test]
async fn it_should_not_allow_adding_a_new_category_to_unauthenticated_users() {
    let mut env = TestEnv::new();
    env.start(api::Implementation::ActixWeb).await;
    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    let response = client
        .add_category(AddCategoryForm {
            name: "CATEGORY NAME".to_string(),
            icon: None,
        })
        .await;

    assert_eq!(response.status, 401);
}

#[tokio::test]
async fn it_should_not_allow_adding_a_new_category_to_non_admins() {
    let mut env = TestEnv::new();
    env.start(api::Implementation::ActixWeb).await;

    let logged_non_admin = new_logged_in_user(&env).await;

    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_non_admin.token);

    let response = client
        .add_category(AddCategoryForm {
            name: "CATEGORY NAME".to_string(),
            icon: None,
        })
        .await;

    assert_eq!(response.status, 403);
}

#[tokio::test]
async fn it_should_allow_admins_to_add_new_categories() {
    let mut env = TestEnv::new();
    env.start(api::Implementation::ActixWeb).await;

    let logged_in_admin = new_logged_in_admin(&env).await;
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

    let category_name = random_category_name();

    let response = client
        .add_category(AddCategoryForm {
            name: category_name.to_string(),
            icon: None,
        })
        .await;

    let res: AddedCategoryResponse = serde_json::from_str(&response.body).unwrap();

    assert_eq!(res.data, category_name);
    if let Some(content_type) = &response.content_type {
        assert_eq!(content_type, "application/json");
    }
    assert_eq!(response.status, 200);
}

#[tokio::test]
async fn it_should_not_allow_adding_duplicated_categories() {
    let mut env = TestEnv::new();
    env.start(api::Implementation::ActixWeb).await;

    // Add a category
    let random_category_name = random_category_name();
    let response = add_category(&random_category_name, &env).await;
    assert_eq!(response.status, 200);

    // Try to add the same category again
    let response = add_category(&random_category_name, &env).await;
    assert_eq!(response.status, 400);
}

#[tokio::test]
async fn it_should_allow_admins_to_delete_categories() {
    let mut env = TestEnv::new();
    env.start(api::Implementation::ActixWeb).await;

    let logged_in_admin = new_logged_in_admin(&env).await;
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

    // Add a category
    let category_name = random_category_name();
    let response = add_category(&category_name, &env).await;
    assert_eq!(response.status, 200);

    let response = client
        .delete_category(DeleteCategoryForm {
            name: category_name.to_string(),
            icon: None,
        })
        .await;

    let res: AddedCategoryResponse = serde_json::from_str(&response.body).unwrap();

    assert_eq!(res.data, category_name);
    if let Some(content_type) = &response.content_type {
        assert_eq!(content_type, "application/json");
    }
    assert_eq!(response.status, 200);
}

#[tokio::test]
async fn it_should_not_allow_non_admins_to_delete_categories() {
    let mut env = TestEnv::new();
    env.start(api::Implementation::ActixWeb).await;

    // Add a category
    let category_name = random_category_name();
    let response = add_category(&category_name, &env).await;
    assert_eq!(response.status, 200);

    let logged_in_non_admin = new_logged_in_user(&env).await;
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_non_admin.token);

    let response = client
        .delete_category(DeleteCategoryForm {
            name: category_name.to_string(),
            icon: None,
        })
        .await;

    assert_eq!(response.status, 403);
}

#[tokio::test]
async fn it_should_not_allow_guests_to_delete_categories() {
    let mut env = TestEnv::new();
    env.start(api::Implementation::ActixWeb).await;
    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    // Add a category
    let category_name = random_category_name();
    let response = add_category(&category_name, &env).await;
    assert_eq!(response.status, 200);

    let response = client
        .delete_category(DeleteCategoryForm {
            name: category_name.to_string(),
            icon: None,
        })
        .await;

    assert_eq!(response.status, 401);
}
