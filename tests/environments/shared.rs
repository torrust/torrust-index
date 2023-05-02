use crate::common::client::Client;
use crate::common::connection_info::{anonymous_connection, authenticated_connection};

/// Provides a shared test environment for testing. All tests shared the same
/// application instance.
pub struct TestEnv {
    pub authority: String,
}

impl TestEnv {
    /// Provides a wrapper for an external running app instance.
    ///
    /// # Panics
    ///
    /// Will panic if the app is not running. This function requires the app to
    /// be running to provide a valid environment.
    pub async fn running() -> Self {
        let env = Self::default();
        let client = env.unauthenticated_client();
        let is_running = client.server_is_running().await;
        assert!(is_running, "Test server is not running on {}", env.authority);
        env
    }

    #[must_use]
    pub fn unauthenticated_client(&self) -> Client {
        Client::new(anonymous_connection(&self.authority))
    }

    #[must_use]
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
