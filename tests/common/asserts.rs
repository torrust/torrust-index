// Text responses

use super::responses::TextResponse;

pub fn assert_response_title(response: &TextResponse, title: &str) {
    let title_element = format!("<title>{title}</title>");

    assert!(
        response.body.contains(title),
        ":\n  response does not contain the title element: `\"{title_element}\"`."
    );
}

pub fn assert_text_ok(response: &TextResponse) {
    assert_eq!(response.status, 200);
    if let Some(content_type) = &response.content_type {
        assert_eq!(content_type, "text/html; charset=utf-8");
    }
}

pub fn _assert_text_bad_request(response: &TextResponse) {
    assert_eq!(response.status, 400);
    if let Some(content_type) = &response.content_type {
        assert_eq!(content_type, "text/plain; charset=utf-8");
    }
}

// JSON responses

pub fn assert_json_ok(response: &TextResponse) {
    assert_eq!(response.status, 200);
    if let Some(content_type) = &response.content_type {
        assert_eq!(content_type, "application/json");
    }
}
