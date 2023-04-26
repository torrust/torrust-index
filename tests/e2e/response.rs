use reqwest::Response as ReqwestResponse;

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: String,
}

impl Response {
    pub async fn from(response: ReqwestResponse) -> Self {
        Self {
            status: response.status().as_u16(),
            content_type: response
                .headers()
                .get("content-type")
                .map(|content_type| content_type.to_str().unwrap().to_owned()),
            body: response.text().await.unwrap(),
        }
    }
}
