pub mod image;

use bytes::Bytes;
use indexmap::IndexMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("entry size limit exceeds total capacity")]
    EntrySizeLimitExceedsTotalCapacity,
    #[error("bytes exceed entry size limit")]
    BytesExceedEntrySizeLimit,
    #[error("cache capacity is too small")]
    CacheCapacityIsTooSmall,
}

#[derive(Debug, Clone)]
pub struct BytesCacheEntry {
    pub bytes: Bytes,
}

// Individual entry destined for the byte cache.
impl BytesCacheEntry {
    pub const fn new(bytes: Bytes) -> Self {
        Self { bytes }
    }
}
#[allow(clippy::module_name_repetitions)]
pub struct BytesCache {
    bytes_table: IndexMap<String, BytesCacheEntry>,
    total_capacity: usize,
    entry_size_limit: usize,
}

impl BytesCache {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bytes_table: IndexMap::new(),
            total_capacity: 0,
            entry_size_limit: 0,
        }
    }

    // With a total capacity in bytes.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let mut new = Self::new();

        new.total_capacity = capacity;

        new
    }

    // With a limit for individual entry sizes.
    #[must_use]
    pub fn with_entry_size_limit(entry_size_limit: usize) -> Self {
        let mut new = Self::new();

        new.entry_size_limit = entry_size_limit;

        new
    }

    /// Helper to create a new bytes cache with both an individual entry and size limit.
    ///
    /// # Errors
    ///
    /// This function will return `Error::EntrySizeLimitExceedsTotalCapacity` if the specified size is too large.
    ///
    pub fn with_capacity_and_entry_size_limit(capacity: usize, entry_size_limit: usize) -> Result<Self, Error> {
        if entry_size_limit > capacity {
            return Err(Error::EntrySizeLimitExceedsTotalCapacity);
        }

        let mut new = Self::new();

        new.total_capacity = capacity;
        new.entry_size_limit = entry_size_limit;

        Ok(new)
    }

    #[allow(clippy::unused_async)]
    pub async fn get(&self, key: &str) -> Option<BytesCacheEntry> {
        self.bytes_table.get(key).cloned()
    }

    // Return the amount of entries in the map.
    #[allow(clippy::unused_async)]
    pub async fn len(&self) -> usize {
        self.bytes_table.len()
    }

    #[allow(clippy::unused_async)]
    pub async fn is_empty(&self) -> bool {
        self.bytes_table.is_empty()
    }

    // Size of all the entry bytes combined.
    #[must_use]
    pub fn total_size(&self) -> usize {
        let mut size: usize = 0;

        for (_, entry) in &self.bytes_table {
            size += entry.bytes.len();
        }

        size
    }

    /// Adds a image to the cache.
    ///
    /// # Errors
    ///
    /// This function will return an error if there is not enough free size.
    ///
    // Insert bytes using key.
    // TODO: Freed space might need to be reserved. Hold and pass write lock between functions?
    // For TO DO above: semaphore: Arc<tokio::sync::Semaphore>, might be a solution.
    #[allow(clippy::unused_async)]
    pub async fn set(&mut self, key: String, bytes: Bytes) -> Result<Option<BytesCacheEntry>, Error> {
        if bytes.len() > self.entry_size_limit {
            return Err(Error::BytesExceedEntrySizeLimit);
        }

        // Remove the old entry so that a new entry will be added as last in the queue.
        drop(self.bytes_table.shift_remove(&key));

        let bytes_cache_entry = BytesCacheEntry::new(bytes);

        self.free_size(bytes_cache_entry.bytes.len())?;

        Ok(self.bytes_table.insert(key, bytes_cache_entry))
    }

    // Free space. Size amount in bytes.
    fn free_size(&mut self, size: usize) -> Result<(), Error> {
        // Size may not exceed the total capacity of the bytes cache.
        if size > self.total_capacity {
            return Err(Error::CacheCapacityIsTooSmall);
        }

        let cache_size = self.total_size();
        let size_to_be_freed = size.saturating_sub(self.total_capacity - cache_size);
        let mut size_freed: usize = 0;

        while size_freed < size_to_be_freed {
            let oldest_entry = self
                .pop()
                .expect("bytes cache has no more entries, yet there isn't enough space.");

            size_freed += oldest_entry.bytes.len();
        }

        Ok(())
    }

    // Remove and return the oldest entry.
    pub fn pop(&mut self) -> Option<BytesCacheEntry> {
        self.bytes_table.shift_remove_index(0).map(|(_, entry)| entry)
    }
}

impl Default for BytesCache {
    fn default() -> Self {
        Self::new()
    }
}
