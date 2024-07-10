use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use bytes::Bytes;
use tokio::sync::RwLock;

use crate::cache::BytesCache;
use crate::config::Configuration;
use crate::models::user::UserId;

pub enum Error {
    UrlIsUnreachable,
    UrlIsNotAnImage,
    ImageTooBig,
    UserQuotaMet,
    Unauthenticated,
}

type UserQuotas = HashMap<i64, ImageCacheQuota>;

/// Returns the current time in seconds.
///
/// # Panics
///
/// This function will panic if the current time is before the UNIX EPOCH.
#[must_use]
pub fn now_in_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!")
        .as_secs()
}

#[derive(Clone)]
pub struct ImageCacheQuota {
    pub user_id: i64,
    pub usage: usize,
    pub max_usage: usize,
    pub date_start_secs: u64,
    pub period_secs: u64,
}

impl ImageCacheQuota {
    #[must_use]
    pub fn new(user_id: i64, max_usage: usize, period_secs: u64) -> Self {
        Self {
            user_id,
            usage: 0,
            max_usage,
            date_start_secs: now_in_secs(),
            period_secs,
        }
    }

    /// Add Usage Quota
    ///
    /// # Errors
    ///
    /// This function will return a `Error::UserQuotaMet` if user quota has been met.
    pub fn add_usage(&mut self, amount: usize) -> Result<(), Error> {
        // Check if quota needs to be reset.
        if now_in_secs() - self.date_start_secs > self.period_secs {
            self.reset();
        }

        if self.is_reached() {
            return Err(Error::UserQuotaMet);
        }

        self.usage = self.usage.saturating_add(amount);

        Ok(())
    }

    pub fn reset(&mut self) {
        self.usage = 0;
        self.date_start_secs = now_in_secs();
    }

    #[must_use]
    pub fn is_reached(&self) -> bool {
        self.usage >= self.max_usage
    }
}

pub struct ImageCacheService {
    image_cache: RwLock<BytesCache>,
    user_quotas: RwLock<UserQuotas>,
    reqwest_client: reqwest::Client,
    cfg: Arc<Configuration>,
}

impl ImageCacheService {
    /// Create a new image cache service.
    ///
    /// # Panics
    ///
    /// This function will panic if the image cache could not be created.
    pub async fn new(cfg: Arc<Configuration>) -> Self {
        let settings = cfg.settings.read().await;

        let image_cache =
            BytesCache::with_capacity_and_entry_size_limit(settings.image_cache.capacity, settings.image_cache.entry_size_limit)
                .expect("Could not create image cache.");

        let reqwest_client = reqwest::Client::builder()
            .timeout(Duration::from_millis(settings.image_cache.max_request_timeout_ms))
            .build()
            .expect("unable to build client request");

        drop(settings);

        Self {
            image_cache: RwLock::new(image_cache),
            user_quotas: RwLock::new(HashMap::new()),
            reqwest_client,
            cfg,
        }
    }

    /// Get an image from the url and insert it into the cache if it isn't cached already.
    /// Unauthenticated users can only get already cached images.
    ///
    /// # Errors
    ///
    /// Return a `Error::Unauthenticated` if the user has not been authenticated.
    pub async fn get_image_by_url(&self, url: &str, user_id: UserId) -> Result<Bytes, Error> {
        if let Some(entry) = self.image_cache.read().await.get(url).await {
            return Ok(entry.bytes);
        }
        self.check_user_quota(&user_id).await?;

        let image_bytes = self.get_image_from_url_as_bytes(url).await?;

        self.check_image_size(&image_bytes).await?;

        // These two functions could be executed after returning the image to the client,
        // but than we would need a dedicated task or thread that executes these functions.
        // This can be problematic if a task is spawned after every user request.
        // Since these functions execute very fast, I don't see a reason to further optimize this.
        // For now.
        self.update_image_cache(url, &image_bytes).await?;

        self.update_user_quota(&user_id, image_bytes.len()).await?;

        Ok(image_bytes)
    }

    async fn get_image_from_url_as_bytes(&self, url: &str) -> Result<Bytes, Error> {
        let res = self
            .reqwest_client
            .clone()
            .get(url)
            .send()
            .await
            .map_err(|_| Error::UrlIsUnreachable)?;

        // code-review: we could get a HTTP 304 response, which doesn't contain a body (the image bytes).

        if let Some(content_type) = res.headers().get("Content-Type") {
            if content_type != "image/jpeg" && content_type != "image/png" {
                return Err(Error::UrlIsNotAnImage);
            }
        } else {
            return Err(Error::UrlIsNotAnImage);
        }

        res.bytes().await.map_err(|_| Error::UrlIsNotAnImage)
    }

    async fn check_user_quota(&self, user_id: &UserId) -> Result<(), Error> {
        if let Some(quota) = self.user_quotas.read().await.get(user_id) {
            if quota.is_reached() {
                return Err(Error::UserQuotaMet);
            }
        }

        Ok(())
    }

    async fn check_image_size(&self, image_bytes: &Bytes) -> Result<(), Error> {
        let settings = self.cfg.settings.read().await;

        if image_bytes.len() > settings.image_cache.entry_size_limit {
            return Err(Error::ImageTooBig);
        }

        Ok(())
    }

    async fn update_image_cache(&self, url: &str, image_bytes: &Bytes) -> Result<(), Error> {
        if self
            .image_cache
            .write()
            .await
            .set(url.to_string(), image_bytes.clone())
            .await
            .is_err()
        {
            return Err(Error::ImageTooBig);
        }

        Ok(())
    }

    async fn update_user_quota(&self, user_id: &UserId, amount: usize) -> Result<(), Error> {
        let settings = self.cfg.settings.read().await;

        let mut quota = self
            .user_quotas
            .read()
            .await
            .get(&user_id)
            .cloned()
            .unwrap_or(ImageCacheQuota::new(
                *user_id,
                settings.image_cache.user_quota_bytes,
                settings.image_cache.user_quota_period_seconds,
            ));

        let _ = quota.add_usage(amount);

        let _ = self.user_quotas.write().await.insert(*user_id, quota);

        Ok(())
    }
}
