use serde::{Deserialize, Serialize};

/// Core configuration for the API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Api {
    /// The default page size for torrent lists.
    pub default_torrent_page_size: u8,
    /// The maximum page size for torrent lists.
    pub max_torrent_page_size: u8,
}

impl Default for Api {
    fn default() -> Self {
        Self {
            default_torrent_page_size: 10,
            max_torrent_page_size: 30,
        }
    }
}
