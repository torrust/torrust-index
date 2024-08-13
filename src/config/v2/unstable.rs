use serde::{Deserialize, Serialize};

/// Unstable configuration options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Unstable {
    /// The casbin configuration used for authorization.
    #[serde(default = "Unstable::default_auth")]
    pub auth: Option<Auth>,
}

impl Default for Unstable {
    fn default() -> Self {
        Self {
            auth: Self::default_auth(),
        }
    }
}

impl Unstable {
    fn default_auth() -> Option<Auth> {
        None
    }
}

/// Unstable auth configuration options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Auth {
    /// The casbin configuration used for authorization.
    #[serde(default = "Auth::default_casbin")]
    pub casbin: Option<Casbin>,
}

impl Default for Auth {
    fn default() -> Self {
        Self {
            casbin: Self::default_casbin(),
        }
    }
}

impl Auth {
    fn default_casbin() -> Option<Casbin> {
        None
    }
}

/// Authentication options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Casbin {
    /// The model. See <https://casbin.org>.
    pub model: String,

    /// The policy. See <https://casbin.org>.
    pub policy: String,
}
