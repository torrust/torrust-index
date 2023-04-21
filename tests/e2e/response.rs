use reqwest::Response as ReqwestResponse;

pub struct Response {
    pub status: u16,
    pub content_type: String,
    pub body: String,
}

impl Response {
    pub async fn from(response: ReqwestResponse) -> Self {
        Self {
            status: response.status().as_u16(),
            content_type: response.headers().get("content-type").unwrap().to_str().unwrap().to_owned(),
            body: response.text().await.unwrap(),
        }
    }
}
