use crate::e2e::client::Client;
use crate::e2e::connection_info::connection_with_no_token;

#[tokio::test]
#[cfg_attr(not(feature = "e2e-tests"), ignore)]
async fn it_should_load_the_about_page_at_the_api_entrypoint() {
    let client = Client::new(connection_with_no_token("localhost:3000"));

    let response = client.entrypoint().await;

    assert_eq!(response.status(), 200);
    assert_eq!(response.headers().get("content-type").unwrap(), "text/html; charset=utf-8");

    let title = format!("<title>About</title>");
    let response_text = response.text().await.unwrap();

    assert!(
        response_text.contains(&title),
        ":\n  response: `\"{response_text}\"`\n  does not contain: `\"{title}\"`."
    );
}
