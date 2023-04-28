use serde::{Deserialize, Serialize};

use crate::e2e::asserts::assert_json_ok;
use crate::e2e::contexts::category::fixtures::{add_category, random_category_name};
use crate::e2e::contexts::user::fixtures::{logged_in_admin, logged_in_user};
use crate::e2e::environment::TestEnv;

// Request data

#[derive(Serialize)]
pub struct AddCategoryForm {
    pub name: String,
    pub icon: Option<String>,
}

pub type DeleteCategoryForm = AddCategoryForm;

// Response data

#[derive(Deserialize)]
pub struct AddedCategoryResponse {
    pub data: String,
}

#[derive(Deserialize, Debug)]
pub struct ListResponse {
    pub data: Vec<ListItem>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ListItem {
    pub category_id: i64,
    pub name: String,
    pub num_torrents: i64,
}

#[tokio::test]
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_return_an_empty_category_list_when_there_are_no_categories() {
    let client = TestEnv::default().unauthenticated_client();

    let response = client.get_categories().await;

    assert_json_ok(&response);
}

#[tokio::test]
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_return_a_category_list() {
    // Add a category
    let category_name = random_category_name();
    let response = add_category(&category_name).await;
    assert_eq!(response.status, 200);

    let client = TestEnv::default().unauthenticated_client();

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
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_not_allow_adding_a_new_category_to_unauthenticated_users() {
    let client = TestEnv::default().unauthenticated_client();

    let response = client
        .add_category(AddCategoryForm {
            name: "CATEGORY NAME".to_string(),
            icon: None,
        })
        .await;

    assert_eq!(response.status, 401);
}

#[tokio::test]
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_not_allow_adding_a_new_category_to_non_admins() {
    let logged_non_admin = logged_in_user().await;
    let client = TestEnv::default().authenticated_client(&logged_non_admin.token);

    let response = client
        .add_category(AddCategoryForm {
            name: "CATEGORY NAME".to_string(),
            icon: None,
        })
        .await;

    assert_eq!(response.status, 403);
}

#[tokio::test]
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_allow_admins_to_add_new_categories() {
    let logged_in_admin = logged_in_admin().await;
    let client = TestEnv::default().authenticated_client(&logged_in_admin.token);

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
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_not_allow_adding_duplicated_categories() {
    // Add a category
    let random_category_name = random_category_name();
    let response = add_category(&random_category_name).await;
    assert_eq!(response.status, 200);

    // Try to add the same category again
    let response = add_category(&random_category_name).await;
    assert_eq!(response.status, 400);
}

#[tokio::test]
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_allow_admins_to_delete_categories() {
    let logged_in_admin = logged_in_admin().await;
    let client = TestEnv::default().authenticated_client(&logged_in_admin.token);

    // Add a category
    let category_name = random_category_name();
    let response = add_category(&category_name).await;
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
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_not_allow_non_admins_to_delete_categories() {
    // Add a category
    let category_name = random_category_name();
    let response = add_category(&category_name).await;
    assert_eq!(response.status, 200);

    let logged_in_non_admin = logged_in_user().await;
    let client = TestEnv::default().authenticated_client(&logged_in_non_admin.token);

    let response = client
        .delete_category(DeleteCategoryForm {
            name: category_name.to_string(),
            icon: None,
        })
        .await;

    assert_eq!(response.status, 403);
}

#[tokio::test]
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_not_allow_guests_to_delete_categories() {
    // Add a category
    let category_name = random_category_name();
    let response = add_category(&category_name).await;
    assert_eq!(response.status, 200);

    let client = TestEnv::default().unauthenticated_client();

    let response = client
        .delete_category(DeleteCategoryForm {
            name: category_name.to_string(),
            icon: None,
        })
        .await;

    assert_eq!(response.status, 401);
}

/* todo:
    - it should allow adding a new category to authenticated clients
    - it should not allow adding a new category with an empty name
    - it should allow adding a new category with an optional icon
    - ...
*/

pub mod fixtures {

    use rand::Rng;

    use super::AddCategoryForm;
    use crate::e2e::contexts::user::fixtures::logged_in_admin;
    use crate::e2e::environment::TestEnv;
    use crate::e2e::responses::TextResponse;

    pub fn software_predefined_category_name() -> String {
        "software".to_string()
    }

    pub fn software_predefined_category_id() -> i64 {
        5
    }

    pub async fn add_category(category_name: &str) -> TextResponse {
        let logged_in_admin = logged_in_admin().await;
        let client = TestEnv::default().authenticated_client(&logged_in_admin.token);

        client
            .add_category(AddCategoryForm {
                name: category_name.to_string(),
                icon: None,
            })
            .await
    }

    pub fn random_category_name() -> String {
        format!("category name {}", random_id())
    }

    fn random_id() -> u64 {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..1_000_000)
    }
}
