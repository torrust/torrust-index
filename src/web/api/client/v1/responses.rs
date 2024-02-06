use reqwest::Response as ReqwestResponse;

#[derive(Debug)]
pub struct TextResponse {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: String,
}

impl TextResponse {
    /// # Panics
    ///
    /// Will panic if:
    ///
    /// - It can't map the content type in the response header to string.
    /// - It can't get the response bytes.
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

    #[must_use]
    pub fn is_json_and_ok(&self) -> bool {
        self.is_ok() && self.is_json()
    }

    #[must_use]
    pub fn is_json(&self) -> bool {
        if let Some(content_type) = &self.content_type {
            return content_type == "application/json";
        }
        false
    }

    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.status == 200
    }
}

#[derive(Debug)]
pub struct BinaryResponse {
    pub status: u16,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

impl BinaryResponse {
    /// # Panics
    ///
    /// Will panic if:
    ///
    /// - It can't map the content type in the response header to string.
    /// - It can't get the response bytes.
    pub async fn from(response: ReqwestResponse) -> Self {
        Self {
            status: response.status().as_u16(),
            content_type: response
                .headers()
                .get("content-type")
                .map(|content_type| content_type.to_str().unwrap().to_owned()),
            bytes: response.bytes().await.unwrap().to_vec(),
        }
    }

    #[must_use]
    pub fn is_a_bit_torrent_file(&self) -> bool {
        self.is_ok() && self.is_bittorrent_content_type()
    }

    #[must_use]
    pub fn is_bittorrent_content_type(&self) -> bool {
        if let Some(content_type) = &self.content_type {
            return content_type == "application/x-bittorrent";
        }
        false
    }

    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.status == 200
    }
}
