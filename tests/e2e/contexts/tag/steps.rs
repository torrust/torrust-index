use crate::common::client::Client;
use crate::common::contexts::tag::fixtures::random_tag_name;
use crate::common::contexts::tag::forms::AddTagForm;
use crate::common::contexts::tag::responses::ListResponse;
use crate::common::responses::TextResponse;
use crate::e2e::contexts::user::steps::new_logged_in_admin;
use crate::e2e::environment::TestEnv;

pub async fn add_random_tag(env: &TestEnv) -> (i64, String) {
    let tag_name = random_tag_name();

    add_tag(&tag_name, env).await;

    let tag_id = get_tag_id(&tag_name, env).await;

    (tag_id, tag_name)
}

pub async fn add_tag(tag_name: &str, env: &TestEnv) -> TextResponse {
    let logged_in_admin = new_logged_in_admin(env).await;
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

    client
        .add_tag(AddTagForm {
            name: tag_name.to_string(),
        })
        .await
}

pub async fn get_tag_id(tag_name: &str, env: &TestEnv) -> i64 {
    let logged_in_admin = new_logged_in_admin(env).await;
    let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

    let response = client.get_tags().await;

    let res: ListResponse = serde_json::from_str(&response.body).unwrap();

    res.find_tag_id(tag_name)
}
