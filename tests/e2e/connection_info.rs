pub fn anonymous_connection(bind_address: &str) -> ConnectionInfo {
    ConnectionInfo::anonymous(bind_address)
}

#[derive(Clone)]
pub struct ConnectionInfo {
    pub bind_address: String,
}

impl ConnectionInfo {
    pub fn anonymous(bind_address: &str) -> Self {
        Self {
            bind_address: bind_address.to_string(),
        }
    }
}
