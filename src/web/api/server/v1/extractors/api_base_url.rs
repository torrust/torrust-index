//! Axum extractor that resolves the API base URL for the current request.
//!
//! The URL is determined by:
//!
//! 1. The configured `net.base_url` if set, or
//! 2. Falling back to the `Host` header from the incoming request.
use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::response::Response;

use crate::common::AppData;

/// Extractor that resolves the API base URL for the current request.
///
/// # Example
///
/// ```ignore
/// async fn handler(ExtractApiBaseUrl(base_url): ExtractApiBaseUrl) {
///     // base_url is something like "http://localhost:3001"
/// }
/// ```
pub struct ExtractApiBaseUrl(pub String);

impl<S> FromRequestParts<S> for ExtractApiBaseUrl
where
    Arc<AppData>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_data = Arc::from_ref(state);

        let base_url = if let Some(url) = app_data.cfg.get_api_base_url().await {
            url
        } else {
            let host = parts.headers.get("host").and_then(|v| v.to_str().ok()).unwrap_or_default();

            // HTTPS is not supported yet.
            // See https://github.com/torrust/torrust-index/issues/131
            format!("http://{host}")
        };

        Ok(Self(base_url))
    }
}
