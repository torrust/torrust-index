//! Image cache proxy.
//!
//! The image cache proxy is a service that allows users to proxy images
//! through the server.
//!
//! Sample URL:
//!
//! <http://0.0.0.0:3001/v1/proxy/image/https%3A%2F%2Fupload.wikimedia.org%2Fwikipedia%2Fcommons%2Fthumb%2F2%2F21%2FMandel_zoom_00_mandelbrot_set.jpg%2F1280px-Mandel_zoom_00_mandelbrot_set.jpg>
use std::sync::Arc;

use bytes::Bytes;

use crate::cache::image::manager::{Error, ImageCacheService};
use crate::models::user::UserId;
use crate::services::user::Repository;

pub struct Service {
    image_cache_service: Arc<ImageCacheService>,
    user_repository: Arc<Box<dyn Repository>>,
}

impl Service {
    #[must_use]
    pub fn new(image_cache_service: Arc<ImageCacheService>, user_repository: Arc<Box<dyn Repository>>) -> Self {
        Self {
            image_cache_service,
            user_repository,
        }
    }

    /// It gets image by URL and caches it.
    ///
    /// # Errors
    ///
    /// It returns an error if:
    ///
    /// * The image URL is unreachable.
    /// * The image URL is not an image.
    /// * The image is too big.
    /// * The user quota is met.
    pub async fn get_image_by_url(&self, url: &str, user_id: &UserId) -> Result<Bytes, Error> {
        let user = self.user_repository.get_compact(user_id).await.ok();

        self.image_cache_service.get_image_by_url(url, user).await
    }
}
