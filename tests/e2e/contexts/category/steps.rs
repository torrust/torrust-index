use crate::common::client::Client;
use crate::common::contexts::category::forms::AddCategoryForm;
use crate::common::responses::TextResponse;
use crate::e2e::contexts::user::steps::new_logged_in_admin;
use crate::e2e::environment::TestEnv;

pub async fn add_category(category_name: &str, env: &TestEnv) -> TextResponse {
    let logged_in_admin = new_logged_in_admin(env).await;
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

    client
        .add_category(AddCategoryForm {
            name: category_name.to_string(),
            icon: None,
        })
        .await
}
