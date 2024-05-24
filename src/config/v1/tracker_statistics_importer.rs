use serde::{Deserialize, Serialize};

/// Configuration for the tracker statistics importer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackerStatisticsImporter {
    /// The interval in seconds to get statistics from the tracker.
    #[serde(default = "TrackerStatisticsImporter::default_torrent_info_update_interval")]
    pub torrent_info_update_interval: u64,
    /// The port the Importer API is listening on. Default to `3002`.
    #[serde(default = "TrackerStatisticsImporter::default_port")]
    pub port: u16,
}

impl Default for TrackerStatisticsImporter {
    fn default() -> Self {
        Self {
            torrent_info_update_interval: Self::default_torrent_info_update_interval(),
            port: Self::default_port(),
        }
    }
}

impl TrackerStatisticsImporter {
    fn default_torrent_info_update_interval() -> u64 {
        3600
    }

    fn default_port() -> u16 {
        3002
    }
}
