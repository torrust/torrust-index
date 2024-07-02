use serde::{Deserialize, Serialize};

/// Information displayed to the user in the website.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Website {
    /// The name of the website.
    #[serde(default = "Website::default_name")]
    pub name: String,
}

impl Default for Website {
    fn default() -> Self {
        Self {
            name: Self::default_name(),
        }
    }
}

impl Website {
    fn default_name() -> String {
        "Torrust".to_string()
    }
}
