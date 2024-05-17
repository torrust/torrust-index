use serde::{Deserialize, Serialize};

/// Information displayed to the user in the website.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Website {
    /// The name of the website.
    pub name: String,
}

impl Default for Website {
    fn default() -> Self {
        Self {
            name: "Torrust".to_string(),
        }
    }
}
