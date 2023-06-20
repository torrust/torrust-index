//! API contract for `tag` context.

use torrust_index_backend::web::api;

use crate::common::asserts::assert_json_ok_response;
use crate::common::client::Client;
use crate::common::contexts::tag::asserts::{assert_added_tag_response, assert_deleted_tag_response};
use crate::common::contexts::tag::fixtures::random_tag_name;
use crate::common::contexts::tag::forms::{AddTagForm, DeleteTagForm};
use crate::common::contexts::tag::responses::ListResponse;
use crate::e2e::environment::TestEnv;
use crate::e2e::web::api::v1::contexts::tag::steps::{add_random_tag, add_tag};
use crate::e2e::web::api::v1::contexts::user::steps::{new_logged_in_admin, new_logged_in_user};

#[tokio::test]
async fn it_should_return_an_empty_tag_list_when_there_are_no_tags() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    let response = client.get_tags().await;

    assert_json_ok_response(&response);
}

#[tokio::test]
async fn it_should_return_a_tag_list() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    // Add a tag
    let tag_name = random_tag_name();
    let response = add_tag(&tag_name, &env).await;
    assert_eq!(response.status, 200);

    let response = client.get_tags().await;

    let res: ListResponse = serde_json::from_str(&response.body).unwrap();

    // There should be at least the tag we added.
    // Since this is an E2E test that could be executed in a shred env,
    // there might be more tags.
    assert!(!res.data.is_empty());
    if let Some(content_type) = &response.content_type {
        assert_eq!(content_type, "application/json");
    }
    assert_eq!(response.status, 200);
}

#[tokio::test]
async fn it_should_not_allow_adding_a_new_tag_to_unauthenticated_users() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    let response = client
        .add_tag(AddTagForm {
            name: "TAG NAME".to_string(),
        })
        .await;

    assert_eq!(response.status, 401);
}

#[tokio::test]
async fn it_should_not_allow_adding_a_new_tag_to_non_admins() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let logged_non_admin = new_logged_in_user(&env).await;

    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_non_admin.token);

    let response = client
        .add_tag(AddTagForm {
            name: "TAG NAME".to_string(),
        })
        .await;

    assert_eq!(response.status, 403);
}

#[tokio::test]
async fn it_should_allow_admins_to_add_new_tags() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let logged_in_admin = new_logged_in_admin(&env).await;
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

    let tag_name = random_tag_name();

    let response = client
        .add_tag(AddTagForm {
            name: tag_name.to_string(),
        })
        .await;

    assert_added_tag_response(&response, &tag_name);
}

#[tokio::test]
async fn it_should_allow_adding_duplicated_tags() {
    // code-review: is this an intended behavior?

    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    // Add a tag
    let random_tag_name = random_tag_name();
    let response = add_tag(&random_tag_name, &env).await;
    assert_eq!(response.status, 200);

    // Try to add the same tag again
    let response = add_tag(&random_tag_name, &env).await;
    assert_eq!(response.status, 200);
}

#[tokio::test]
async fn it_should_allow_adding_a_tag_with_an_empty_name() {
    // code-review: is this an intended behavior?

    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let empty_tag_name = String::new();
    let response = add_tag(&empty_tag_name, &env).await;
    assert_eq!(response.status, 200);
}

#[tokio::test]
async fn it_should_allow_admins_to_delete_tags() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let logged_in_admin = new_logged_in_admin(&env).await;
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

    let (tag_id, _tag_name) = add_random_tag(&env).await;

    let response = client.delete_tag(DeleteTagForm { tag_id }).await;

    assert_deleted_tag_response(&response, tag_id);
}

#[tokio::test]
async fn it_should_not_allow_non_admins_to_delete_tags() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let logged_in_non_admin = new_logged_in_user(&env).await;
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_non_admin.token);

    let (tag_id, _tag_name) = add_random_tag(&env).await;

    let response = client.delete_tag(DeleteTagForm { tag_id }).await;

    assert_eq!(response.status, 403);
}

#[tokio::test]
async fn it_should_not_allow_guests_to_delete_tags() {
    let mut env = TestEnv::new();
    env.start(api::Version::V1).await;

    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    let (tag_id, _tag_name) = add_random_tag(&env).await;

    let response = client.delete_tag(DeleteTagForm { tag_id }).await;

    assert_eq!(response.status, 401);
}
