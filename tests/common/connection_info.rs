#[derive(Clone)]
pub struct ConnectionInfo {
    pub bind_address: String,
    pub token: Option<String>,
}

impl ConnectionInfo {
    pub fn new(bind_address: &str, token: &str) -> Self {
        Self {
            bind_address: bind_address.to_string(),
            token: Some(token.to_string()),
        }
    }

    pub fn anonymous(bind_address: &str) -> Self {
        Self {
            bind_address: bind_address.to_string(),
            token: None,
        }
    }
}
