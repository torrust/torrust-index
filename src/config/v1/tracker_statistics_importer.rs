use serde::{Deserialize, Serialize};

/// Configuration for the tracker statistics importer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackerStatisticsImporter {
    /// The interval in seconds to get statistics from the tracker.
    pub torrent_info_update_interval: u64,
    /// The port the Importer API is listening on. Default to `3002`.
    pub port: u16,
}

impl Default for TrackerStatisticsImporter {
    fn default() -> Self {
        Self {
            torrent_info_update_interval: 3600,
            port: 3002,
        }
    }
}
