use reqwest::multipart;
use serde::Serialize;

use super::connection_info::ConnectionInfo;
use super::responses::{BinaryResponse, TextResponse};

pub type ReqwestQuery = Vec<ReqwestQueryParam>;
pub type ReqwestQueryParam = (String, String);

/// URL Query component
#[derive(Default, Debug)]
pub struct Query {
    params: Vec<QueryParam>,
}

impl Query {
    #[must_use]
    pub fn empty() -> Self {
        Self { params: vec![] }
    }

    #[must_use]
    pub fn with_params(params: Vec<QueryParam>) -> Self {
        Self { params }
    }

    pub fn add_param(&mut self, param: QueryParam) {
        self.params.push(param);
    }
}

impl From<Query> for ReqwestQuery {
    fn from(url_search_params: Query) -> Self {
        url_search_params
            .params
            .iter()
            .map(|param| ReqwestQueryParam::from((*param).clone()))
            .collect()
    }
}

/// URL query param
#[derive(Clone, Debug)]
pub struct QueryParam {
    name: String,
    value: String,
}

impl QueryParam {
    #[must_use]
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}

impl From<QueryParam> for ReqwestQueryParam {
    fn from(param: QueryParam) -> Self {
        (param.name, param.value)
    }
}

/// Generic HTTP Client
pub struct Http {
    connection_info: ConnectionInfo,
}

impl Http {
    #[must_use]
    pub fn new(connection_info: ConnectionInfo) -> Self {
        Self { connection_info }
    }

    /// # Panics
    ///
    /// Will panic if there was an error while sending request, redirect loop
    /// was detected or redirect limit was exhausted.    
    pub async fn get(&self, path: &str, params: Query) -> TextResponse {
        let response = match &self.connection_info.token {
            Some(token) => reqwest::Client::builder()
                .build()
                .unwrap()
                .get(self.base_url(path).clone())
                .query(&ReqwestQuery::from(params))
                .bearer_auth(token)
                .send()
                .await
                .unwrap(),
            None => reqwest::Client::builder()
                .build()
                .unwrap()
                .get(self.base_url(path).clone())
                .query(&ReqwestQuery::from(params))
                .send()
                .await
                .unwrap(),
        };
        TextResponse::from(response).await
    }

    /// # Panics
    ///
    /// Will panic if there was an error while sending request, redirect loop
    /// was detected or redirect limit was exhausted.    
    pub async fn get_binary(&self, path: &str, params: Query) -> BinaryResponse {
        let response = match &self.connection_info.token {
            Some(token) => reqwest::Client::builder()
                .build()
                .unwrap()
                .get(self.base_url(path).clone())
                .query(&ReqwestQuery::from(params))
                .bearer_auth(token)
                .send()
                .await
                .unwrap(),
            None => reqwest::Client::builder()
                .build()
                .unwrap()
                .get(self.base_url(path).clone())
                .query(&ReqwestQuery::from(params))
                .send()
                .await
                .unwrap(),
        };
        // todo: If the response is a JSON, it returns the JSON body in a byte
        //   array. This is not the expected behavior.
        //  - Rename BinaryResponse to BinaryTorrentResponse
        //  - Return an error if the response is not a bittorrent file
        BinaryResponse::from(response).await
    }

    /// # Errors
    ///
    /// Will fail if there was an error while sending request, redirect loop
    /// was detected or redirect limit was exhausted.
    ///
    /// # Panics
    ///
    /// This method fails it can't build a `reqwest` client.
    pub async fn inner_get(&self, path: &str) -> Result<reqwest::Response, reqwest::Error> {
        reqwest::Client::builder()
            .build()
            .unwrap()
            .get(self.base_url(path).clone())
            .send()
            .await
    }

    /// # Panics
    ///
    /// Will panic if there was an error while sending request, redirect loop
    /// was detected or redirect limit was exhausted.    
    pub async fn post<T: Serialize + ?Sized>(&self, path: &str, form: &T) -> TextResponse {
        let response = match &self.connection_info.token {
            Some(token) => reqwest::Client::new()
                .post(self.base_url(path).clone())
                .bearer_auth(token)
                .json(&form)
                .send()
                .await
                .unwrap(),
            None => reqwest::Client::new()
                .post(self.base_url(path).clone())
                .json(&form)
                .send()
                .await
                .unwrap(),
        };
        TextResponse::from(response).await
    }

    /// # Panics
    ///
    /// Will panic if there was an error while sending request, redirect loop
    /// was detected or redirect limit was exhausted.    
    pub async fn post_multipart(&self, path: &str, form: multipart::Form) -> TextResponse {
        let response = match &self.connection_info.token {
            Some(token) => reqwest::Client::builder()
                .build()
                .unwrap()
                .post(self.base_url(path).clone())
                .multipart(form)
                .bearer_auth(token)
                .send()
                .await
                .expect("failed to send multipart request with token"),
            None => reqwest::Client::builder()
                .build()
                .unwrap()
                .post(self.base_url(path).clone())
                .multipart(form)
                .send()
                .await
                .expect("failed to send multipart request without token"),
        };
        TextResponse::from(response).await
    }

    /// # Panics
    ///
    /// Will panic if there was an error while sending request, redirect loop
    /// was detected or redirect limit was exhausted.
    pub async fn put<T: Serialize + ?Sized>(&self, path: &str, form: &T) -> TextResponse {
        let response = match &self.connection_info.token {
            Some(token) => reqwest::Client::new()
                .put(self.base_url(path).clone())
                .bearer_auth(token)
                .json(&form)
                .send()
                .await
                .unwrap(),
            None => reqwest::Client::new()
                .put(self.base_url(path).clone())
                .json(&form)
                .send()
                .await
                .unwrap(),
        };
        TextResponse::from(response).await
    }

    /// # Panics
    ///
    /// Will panic if there was an error while sending request, redirect loop
    /// was detected or redirect limit was exhausted.    
    pub async fn delete(&self, path: &str) -> TextResponse {
        let response = match &self.connection_info.token {
            Some(token) => reqwest::Client::new()
                .delete(self.base_url(path).clone())
                .bearer_auth(token)
                .send()
                .await
                .unwrap(),
            None => reqwest::Client::new()
                .delete(self.base_url(path).clone())
                .send()
                .await
                .unwrap(),
        };
        TextResponse::from(response).await
    }

    /// # Panics
    ///
    /// Will panic if there was an error while sending request, redirect loop
    /// was detected or redirect limit was exhausted.
    pub async fn delete_with_body<T: Serialize + ?Sized>(&self, path: &str, form: &T) -> TextResponse {
        let response = match &self.connection_info.token {
            Some(token) => reqwest::Client::new()
                .delete(self.base_url(path).clone())
                .bearer_auth(token)
                .json(&form)
                .send()
                .await
                .unwrap(),
            None => reqwest::Client::new()
                .delete(self.base_url(path).clone())
                .json(&form)
                .send()
                .await
                .unwrap(),
        };
        TextResponse::from(response).await
    }

    fn base_url(&self, path: &str) -> String {
        format!(
            "{}://{}{}{path}",
            &self.connection_info.scheme, &self.connection_info.bind_address, &self.connection_info.base_path
        )
    }
}
