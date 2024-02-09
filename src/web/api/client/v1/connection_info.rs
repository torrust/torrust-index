use std::{fmt, str::FromStr};

use reqwest::Url;

#[derive(Clone)]
pub enum Scheme {
    Http,
    Https,
}

impl fmt::Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scheme::Http => write!(f, "http"),
            Scheme::Https => write!(f, "https"),
        }
    }
}

impl FromStr for Scheme {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "http" => Ok(Scheme::Http),
            "https" => Ok(Scheme::Https),
            _ => Err(()),
        }
    }
}

#[derive(Clone)]
pub struct ConnectionInfo {
    pub scheme: Scheme,
    pub bind_address: String,
    pub base_path: String,
    pub token: Option<String>,
}

impl ConnectionInfo {
    /// # Panics
    ///
    /// Will panic if the the base URL does not have a valid scheme: `http` or `https`.
    #[must_use]
    pub fn new(base_url: &Url, base_path: &str, token: &str) -> Self {
        Self {
            scheme: base_url
                .scheme()
                .parse()
                .expect("base API URL scheme should be 'http' or 'https"),
            bind_address: base_url.authority().to_string(),
            base_path: base_path.to_string(),
            token: Some(token.to_string()),
        }
    }

    /// # Panics
    ///
    /// Will panic if the the base URL does not have a valid scheme: `http` or `https`.    
    #[must_use]
    pub fn anonymous(base_url: &Url, base_path: &str) -> Self {
        Self {
            scheme: base_url
                .scheme()
                .parse()
                .expect("base API URL scheme should be 'http' or 'https"),
            bind_address: base_url.authority().to_string(),
            base_path: base_path.to_string(),
            token: None,
        }
    }
}
