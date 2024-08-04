//! API contract for `category` context.

use torrust_index::web::api;

use crate::common::asserts::assert_json_ok_response;
use crate::common::client::Client;
use crate::common::contexts::category::asserts::{assert_added_category_response, assert_deleted_category_response};
use crate::common::contexts::category::fixtures::random_category_name;
use crate::common::contexts::category::forms::{AddCategoryForm, DeleteCategoryForm};
use crate::common::contexts::category::responses::ListResponse;
use crate::e2e::environment::TestEnv;
use crate::e2e::web::api::v1::contexts::category::steps::{add_category, add_random_category};
use crate::e2e::web::api::v1::contexts::user::steps::{new_logged_in_admin, new_logged_in_user};

#[tokio::test]
async fn it_should_return_an_empty_category_list_when_there_are_no_categories() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    let response = client.get_categories().await;

    assert_json_ok_response(&response);
}

#[tokio::test]
async fn it_should_return_a_category_list() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    add_random_category(&env).await;

    let response = client.get_categories().await;

    let res: ListResponse = serde_json::from_str(&response.body).unwrap();

    // There should be at least the category we added.
    // Since this is an E2E test and it could be run in a shared test env,
    // there might be more categories.
    assert!(res.data.len() > 1);
    if let Some(content_type) = &response.content_type {
        assert_eq!(content_type, "application/json");
    }
    assert_eq!(response.status, 200);
}

#[tokio::test]
async fn it_should_not_allow_adding_a_new_category_to_unauthenticated_users() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    let response = client
        .add_category(AddCategoryForm {
            name: "CATEGORY NAME".to_string(),
            icon: None,
        })
        .await;

    assert_eq!(response.status, 403);
}

#[tokio::test]
async fn it_should_not_allow_adding_a_new_category_to_non_admins() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

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
    env.start(api::Version::V1).await;

    let logged_in_admin = new_logged_in_admin(&env).await;
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

    let category_name = random_category_name();

    let response = client
        .add_category(AddCategoryForm {
            name: category_name.to_string(),
            icon: None,
        })
        .await;

    assert_added_category_response(&response, &category_name);
}

#[tokio::test]
async fn it_should_not_allow_adding_empty_categories() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let logged_in_admin = new_logged_in_admin(&env).await;
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

    let invalid_category_names = vec![String::new(), " ".to_string()];

    for invalid_name in invalid_category_names {
        let response = client
            .add_category(AddCategoryForm {
                name: invalid_name,
                icon: None,
            })
            .await;

        assert_eq!(response.status, 400);
    }
}

#[tokio::test]
async fn it_should_not_allow_adding_duplicated_categories() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let added_category_name = add_random_category(&env).await;

    // Try to add the same category again
    let response = add_category(&added_category_name, &env).await;

    assert_eq!(response.status, 400);
}

#[tokio::test]
async fn it_should_allow_admins_to_delete_categories() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let logged_in_admin = new_logged_in_admin(&env).await;
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

    let added_category_name = add_random_category(&env).await;

    let response = client
        .delete_category(DeleteCategoryForm {
            name: added_category_name.to_string(),
            icon: None,
        })
        .await;

    assert_deleted_category_response(&response, &added_category_name);
}

#[tokio::test]
async fn it_should_not_allow_non_admins_to_delete_categories() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let added_category_name = add_random_category(&env).await;

    let logged_in_non_admin = new_logged_in_user(&env).await;
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_non_admin.token);

    let response = client
        .delete_category(DeleteCategoryForm {
            name: added_category_name.to_string(),
            icon: None,
        })
        .await;

    assert_eq!(response.status, 403);
}

#[tokio::test]
async fn it_should_not_allow_guests_to_delete_categories() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    let added_category_name = add_random_category(&env).await;

    let response = client
        .delete_category(DeleteCategoryForm {
            name: added_category_name.to_string(),
            icon: None,
        })
        .await;

    assert_eq!(response.status, 403);
}
