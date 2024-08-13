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
        match client.server_is_running().await {
            Ok(()) => {}
            Err(err) => panic!("Test server is not running on {}. Error: {err}", env.authority),
        }
        env
    }

    /// Provides the API server socket address.
    #[must_use]
    pub fn server_socket_addr(&self) -> Option<String> {
        // If the E2E configuration uses port 0 in the future instead of a
        // predefined port (right now we are using port 3001) we will
        // need to pass an env var with the port used by the server.
        Some(self.authority.clone())
    }
}

impl Default for TestEnv {
    fn default() -> Self {
        Self {
            authority: "127.0.0.1:3001".to_string(),
        }
    }
}
