use std::sync::Arc;

use torrust_index_backend::databases::database::connect_database;

use crate::common::client::Client;
use crate::common::contexts::user::fixtures::random_user_registration;
use crate::common::contexts::user::forms::{LoginForm, RegisteredUser};
use crate::common::contexts::user::responses::{LoggedInUserData, SuccessfulLoginResponse};
use crate::e2e::environment::TestEnv;

pub async fn new_logged_in_admin(env: &TestEnv) -> LoggedInUserData {
    let user = new_logged_in_user(env).await;

    let database = Arc::new(
        connect_database(&env.database_connect_url().unwrap())
            .await
            .expect("Database error."),
    );

    let user_profile = database.get_user_profile_from_username(&user.username).await.unwrap();

    database.grant_admin_role(user_profile.user_id).await.unwrap();

    user
}

pub async fn new_logged_in_user(env: &TestEnv) -> LoggedInUserData {
    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    let registered_user = new_registered_user(env).await;

    let response = client
        .login_user(LoginForm {
            login: registered_user.username.clone(),
            password: registered_user.password.clone(),
        })
        .await;

    let res: SuccessfulLoginResponse = serde_json::from_str(&response.body).unwrap();

    let user = res.data;

    if !user.admin {
        return user;
    }

    // The first registered user is always an admin, so we need to register
    // a second user to ge a non admin user.

    let second_registered_user = new_registered_user(env).await;

    let response = client
        .login_user(LoginForm {
            login: second_registered_user.username.clone(),
            password: second_registered_user.password.clone(),
        })
        .await;

    let res: SuccessfulLoginResponse = serde_json::from_str(&response.body).unwrap();

    res.data
}

pub async fn new_registered_user(env: &TestEnv) -> RegisteredUser {
    let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

    let form = random_user_registration();

    let registered_user = form.clone();

    let _response = client.register_user(form).await;

    registered_user
}
