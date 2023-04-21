use crate::e2e::asserts::assert_json_ok;
use crate::e2e::env::TestEnv;
use crate::e2e::http::Query;

#[tokio::test]
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_return_an_empty_category_list_when_there_are_no_categories() {
    let client = TestEnv::default().unauthenticated_client();

    let response = client.get("category", Query::empty()).await;

    assert_json_ok(&response);
}

/* todo:
    - it_should_not_allow_adding_a_new_category_to_unauthenticated_clients
    - it should allow adding a new category to authenticated clients
    - it should not allow adding a new category with an empty name
    - it should not allow adding a new category with an empty icon
    - ...
*/
