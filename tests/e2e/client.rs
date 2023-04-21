use reqwest::Response as ReqwestResponse;

use crate::e2e::connection_info::ConnectionInfo;
use crate::e2e::http::{Query, ReqwestQuery};
use crate::e2e::response::Response;

/// API Client
pub struct Client {
    connection_info: ConnectionInfo,
    base_path: String,
}

impl Client {
    pub fn new(connection_info: ConnectionInfo) -> Self {
        Self {
            connection_info,
            base_path: "/".to_string(),
        }
    }

    pub async fn root(&self) -> Response {
        self.get("", Query::empty()).await
    }

    pub async fn about(&self) -> Response {
        self.get("about", Query::empty()).await
    }

    pub async fn license(&self) -> Response {
        self.get("about/license", Query::empty()).await
    }

    pub async fn get(&self, path: &str, params: Query) -> Response {
        self.get_request_with_query(path, params).await
    }

    /*
    pub async fn post(&self, path: &str) -> Response {
        reqwest::Client::new().post(self.base_url(path).clone()).send().await.unwrap()
    }

    async fn delete(&self, path: &str) -> Response {
        reqwest::Client::new()
            .delete(self.base_url(path).clone())
            .send()
            .await
            .unwrap()
    }

    pub async fn get_request(&self, path: &str) -> Response {
        get(&self.base_url(path), None).await
    }
    */

    pub async fn get_request_with_query(&self, path: &str, params: Query) -> Response {
        get(&self.base_url(path), Some(params)).await
    }

    fn base_url(&self, path: &str) -> String {
        format!("http://{}{}{path}", &self.connection_info.bind_address, &self.base_path)
    }
}

async fn get(path: &str, query: Option<Query>) -> Response {
    let response: ReqwestResponse = match query {
        Some(params) => reqwest::Client::builder()
            .build()
            .unwrap()
            .get(path)
            .query(&ReqwestQuery::from(params))
            .send()
            .await
            .unwrap(),
        None => reqwest::Client::builder().build().unwrap().get(path).send().await.unwrap(),
    };
    Response::from(response).await
}
