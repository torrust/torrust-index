use std::sync::Arc;

use torrust_index_backend::databases::database::connect_database;

use crate::common::contexts::user::fixtures::random_user_registration;
use crate::common::contexts::user::forms::{LoginForm, RegisteredUser};
use crate::common::contexts::user::responses::{LoggedInUserData, SuccessfulLoginResponse};
use crate::e2e::environment::TestEnv;

pub async fn logged_in_admin() -> LoggedInUserData {
    let user = logged_in_user().await;

    // todo: get from E2E config file `config-idx-back.toml.local`
    let connect_url = "sqlite://storage/database/torrust_index_backend_e2e_testing.db?mode=rwc";

    let database = Arc::new(connect_database(connect_url).await.expect("Database error."));

    let user_profile = database.get_user_profile_from_username(&user.username).await.unwrap();

    database.grant_admin_role(user_profile.user_id).await.unwrap();

    user
}

pub async fn logged_in_user() -> LoggedInUserData {
    let client = TestEnv::default().unauthenticated_client();

    let registered_user = registered_user().await;

    let response = client
        .login_user(LoginForm {
            login: registered_user.username.clone(),
            password: registered_user.password.clone(),
        })
        .await;

    let res: SuccessfulLoginResponse = serde_json::from_str(&response.body).unwrap();
    res.data
}

pub async fn registered_user() -> RegisteredUser {
    let client = TestEnv::default().unauthenticated_client();

    let form = random_user_registration();

    let registered_user = form.clone();

    let _response = client.register_user(form).await;

    registered_user
}
