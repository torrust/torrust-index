#[derive(Clone)]
pub struct ConnectionInfo {
    pub bind_address: String,
    pub base_path: String,
    pub token: Option<String>,
}

impl ConnectionInfo {
    #[must_use]
    pub fn new(bind_address: &str, base_path: &str, token: &str) -> Self {
        Self {
            bind_address: bind_address.to_string(),
            base_path: base_path.to_string(),
            token: Some(token.to_string()),
        }
    }

    #[must_use]
    pub fn anonymous(bind_address: &str, base_path: &str) -> Self {
        Self {
            bind_address: bind_address.to_string(),
            base_path: base_path.to_string(),
            token: None,
        }
    }
}
