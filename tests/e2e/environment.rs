use crate::common::client::Client;
use crate::common::connection_info::{anonymous_connection, authenticated_connection};

pub struct TestEnv {
    pub authority: String,
}

impl TestEnv {
    pub async fn running() -> Self {
        let env = Self::default();
        let client = env.unauthenticated_client();
        assert!(
            client.server_is_running().await,
            "Test server is not running on {}",
            env.authority
        );
        env
    }

    pub fn unauthenticated_client(&self) -> Client {
        Client::new(anonymous_connection(&self.authority))
    }

    pub fn authenticated_client(&self, token: &str) -> Client {
        Client::new(authenticated_connection(&self.authority, token))
    }
}

impl Default for TestEnv {
    fn default() -> Self {
        Self {
            authority: "localhost:3000".to_string(),
        }
    }
}
