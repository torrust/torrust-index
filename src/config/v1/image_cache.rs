use serde::{Deserialize, Serialize};

/// Configuration for the image proxy cache.
///
/// Users have a cache quota per period. For example: 100MB per day.
/// When users are navigating the site, they will be downloading images that are
/// embedded in the torrent description. These images will be cached in the
/// proxy. The proxy will not download new images if the user has reached the
/// quota.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageCache {
    /// Cache size in bytes.
    #[serde(default = "ImageCache::default_capacity")]
    pub capacity: usize,

    /// Maximum size in bytes for a single image.
    #[serde(default = "ImageCache::default_entry_size_limit")]
    pub entry_size_limit: usize,

    /// Maximum time in seconds to wait for downloading the image form the original source.
    #[serde(default = "ImageCache::default_max_request_timeout_ms")]
    pub max_request_timeout_ms: u64,

    /// Users have a cache quota per period. For example: 100MB per day.
    /// This is the maximum size in bytes (100MB in bytes).    
    #[serde(default = "ImageCache::default_user_quota_bytes")]
    pub user_quota_bytes: usize,

    /// Users have a cache quota per period. For example: 100MB per day.
    /// This is the period in seconds (1 day in seconds).
    #[serde(default = "ImageCache::default_user_quota_period_seconds")]
    pub user_quota_period_seconds: u64,
}

impl Default for ImageCache {
    fn default() -> Self {
        Self {
            max_request_timeout_ms: Self::default_max_request_timeout_ms(),
            capacity: Self::default_capacity(),
            entry_size_limit: Self::default_entry_size_limit(),
            user_quota_period_seconds: Self::default_user_quota_period_seconds(),
            user_quota_bytes: Self::default_user_quota_bytes(),
        }
    }
}

impl ImageCache {
    fn default_max_request_timeout_ms() -> u64 {
        1000
    }

    fn default_capacity() -> usize {
        128_000_000
    }

    fn default_entry_size_limit() -> usize {
        4_000_000
    }

    fn default_user_quota_period_seconds() -> u64 {
        3600
    }

    fn default_user_quota_bytes() -> usize {
        64_000_000
    }
}
