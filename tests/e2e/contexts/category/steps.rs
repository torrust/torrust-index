use crate::common::contexts::category::forms::AddCategoryForm;
use crate::common::responses::TextResponse;
use crate::e2e::contexts::user::steps::logged_in_admin;
use crate::e2e::environment::TestEnv;

pub async fn add_category(category_name: &str) -> TextResponse {
    let logged_in_admin = logged_in_admin().await;
    let client = TestEnv::running().await.authenticated_client(&logged_in_admin.token);

    client
        .add_category(AddCategoryForm {
            name: category_name.to_string(),
            icon: None,
        })
        .await
}
