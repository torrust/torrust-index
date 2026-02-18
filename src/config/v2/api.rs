use serde::{Deserialize, Serialize};

/// Core configuration for the API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Api {
    /// The default page size for torrent lists.
    #[serde(default = "Api::default_default_torrent_page_size")]
    pub default_torrent_page_size: u8,

    /// The maximum page size for torrent lists.
    #[serde(default = "Api::default_max_torrent_page_size")]
    pub max_torrent_page_size: u8,

    /// The default page size for user profile lists.
    #[serde(default = "Api::default_user_profile_page_size")]
    pub default_user_profile_page_size: u8,

    /// The maximum page size for user profile lists.
    #[serde(default = "Api::default_max_user_profile_page_size")]
    pub max_user_profile_page_size: u8,
}

impl Default for Api {
    fn default() -> Self {
        Self {
            default_torrent_page_size: Self::default_default_torrent_page_size(),
            max_torrent_page_size: Self::default_max_torrent_page_size(),
            default_user_profile_page_size: Self::default_user_profile_page_size(),
            max_user_profile_page_size: Self::default_max_user_profile_page_size(),
        }
    }
}

impl Api {
    const fn default_default_torrent_page_size() -> u8 {
        10
    }

    const fn default_max_torrent_page_size() -> u8 {
        100
    }

    const fn default_user_profile_page_size() -> u8 {
        10
    }

    const fn default_max_user_profile_page_size() -> u8 {
        100
    }
}
