use crate::e2e::response::Response;

pub fn assert_response_title(response: &Response, title: &str) {
    let title_element = format!("<title>{title}</title>");

    assert!(
        response.body.contains(&title),
        ":\n  response does not contain the title element: `\"{title_element}\"`."
    );
}

pub fn assert_text_ok(response: &Response) {
    assert_eq!(response.status, 200);
    assert_eq!(response.content_type, "text/html; charset=utf-8");
}

pub fn assert_json_ok(response: &Response) {
    assert_eq!(response.status, 200);
    assert_eq!(response.content_type, "application/json");
}
