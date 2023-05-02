use crate::common::client::Client;

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
        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());
        let is_running = client.server_is_running().await;
        assert!(is_running, "Test server is not running on {}", env.authority);
        env
    }

    /// Provides the API server socket address.
    pub fn server_socket_addr(&self) -> Option<String> {
        Some(self.authority.clone())
    }
}

impl Default for TestEnv {
    fn default() -> Self {
        Self {
            authority: "localhost:3000".to_string(),
        }
    }
}
