pub fn anonymous_connection(bind_address: &str) -> ConnectionInfo {
    ConnectionInfo::anonymous(bind_address)
}

pub fn authenticated_connection(bind_address: &str, token: &str) -> ConnectionInfo {
    ConnectionInfo::new(bind_address, token)
}

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
