use serde::{Deserialize, Serialize};

/// Core configuration for the API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Api {
    /// The default page size for torrent lists.
    #[serde(default = "Api::default_default_torrent_page_size")]
    pub default_torrent_page_size: u8,
    /// The maximum page size for torrent lists.
    #[serde(default = "Api::default_max_torrent_page_size")]
    pub max_torrent_page_size: u8,
}

impl Default for Api {
    fn default() -> Self {
        Self {
            default_torrent_page_size: Api::default_default_torrent_page_size(),
            max_torrent_page_size: Api::default_max_torrent_page_size(),
        }
    }
}

impl Api {
    fn default_default_torrent_page_size() -> u8 {
        10
    }

    fn default_max_torrent_page_size() -> u8 {
        30
    }
}
