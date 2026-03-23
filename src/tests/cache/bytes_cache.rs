//! Tests for `crate::cache::BytesCache` (the re-exported wrapper).

use bytes::Bytes;

use crate::cache::BytesCache;

#[tokio::test]
async fn set_bytes_cache_with_capacity_and_entry_size_limit_should_succeed() {
    let mut bytes_cache = BytesCache::with_capacity_and_entry_size_limit(6, 6).unwrap();
    let bytes: Bytes = Bytes::from("abcdef");

    assert!(bytes_cache.set("1".to_string(), bytes).await.is_ok());
}

#[tokio::test]
async fn given_a_bytes_cache_with_a_capacity_and_entry_size_limit_it_should_allow_adding_new_entries_if_the_limit_is_not_exceeded()
 {
    let bytes: Bytes = Bytes::from("abcdef");

    let mut bytes_cache = BytesCache::with_capacity_and_entry_size_limit(bytes.len() * 2, bytes.len()).unwrap();

    // Add first entry (6 bytes)
    assert!(bytes_cache.set("key1".to_string(), bytes.clone()).await.is_ok());

    // Add second entry (6 bytes)
    assert!(bytes_cache.set("key2".to_string(), bytes).await.is_ok());

    // Both entries were added because we did not reach the limit
    assert_eq!(bytes_cache.len().await, 2);
}

#[tokio::test]
async fn given_a_bytes_cache_with_a_capacity_and_entry_size_limit_it_should_not_allow_adding_new_entries_if_the_capacity_is_exceeded()
 {
    let bytes: Bytes = Bytes::from("abcdef");

    let mut bytes_cache = BytesCache::with_capacity_and_entry_size_limit(bytes.len() * 2 - 1, bytes.len()).unwrap();

    // Add first entry (6 bytes)
    assert!(bytes_cache.set("key1".to_string(), bytes.clone()).await.is_ok());

    // Add second entry (6 bytes)
    assert!(bytes_cache.set("key2".to_string(), bytes).await.is_ok());

    // Only one entry is in the cache, because otherwise the total capacity would have been exceeded
    assert_eq!(bytes_cache.len().await, 1);
}

#[tokio::test]
async fn set_bytes_cache_with_capacity_and_entry_size_limit_should_fail() {
    let mut bytes_cache = BytesCache::with_capacity_and_entry_size_limit(6, 5).unwrap();
    let bytes: Bytes = Bytes::from("abcdef");

    assert!(bytes_cache.set("1".to_string(), bytes).await.is_err());
}
