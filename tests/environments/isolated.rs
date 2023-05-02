use tempfile::TempDir;
use torrust_index_backend::config::{TorrustConfig, FREE_PORT};

use super::app_starter::AppStarter;
use crate::common::client::Client;
use crate::common::connection_info::{anonymous_connection, authenticated_connection};
use crate::common::random;

/// Provides an isolated test environment for testing. The environment is
/// launched with a temporary directory and a default ephemeral configuration
/// before running the test.
pub struct TestEnv {
    pub app_starter: AppStarter,
    pub temp_dir: TempDir,
}

impl TestEnv {
    /// Provides a running app instance for integration tests.
    pub async fn running() -> Self {
        let mut env = TestEnv::with_test_configuration();
        env.start().await;
        env
    }

    /// Provides a test environment with a default configuration for testing
    /// application.
    #[must_use]
    pub fn with_test_configuration() -> Self {
        let temp_dir = TempDir::new().expect("failed to create a temporary directory");

        let configuration = ephemeral(&temp_dir);

        let app_starter = AppStarter::with_custom_configuration(configuration);

        Self { app_starter, temp_dir }
    }

    /// Starts the app.
    pub async fn start(&mut self) {
        self.app_starter.start().await;
    }

    /// Provides an unauthenticated client for integration tests.
    #[must_use]
    pub fn unauthenticated_client(&self) -> Client {
        Client::new(anonymous_connection(
            &self
                .server_socket_addr()
                .expect("app should be started to get the server socket address"),
        ))
    }

    /// Provides an authenticated client for integration tests.
    #[must_use]
    pub fn _authenticated_client(&self, token: &str) -> Client {
        Client::new(authenticated_connection(
            &self
                .server_socket_addr()
                .expect("app should be started to get the server socket address"),
            token,
        ))
    }

    /// Provides the API server socket address.
    fn server_socket_addr(&self) -> Option<String> {
        self.app_starter.server_socket_addr().map(|addr| addr.to_string())
    }
}

/// Provides a configuration with ephemeral data for testing.
fn ephemeral(temp_dir: &TempDir) -> TorrustConfig {
    let mut configuration = TorrustConfig::default();

    // Ephemeral API port
    configuration.net.port = FREE_PORT;

    // Ephemeral SQLite database
    configuration.database.connect_url = format!("sqlite://{}?mode=rwc", random_database_file_path_in(temp_dir));

    configuration
}

fn random_database_file_path_in(temp_dir: &TempDir) -> String {
    let random_db_id = random::string(16);
    let db_file_name = format!("data_{random_db_id}.db");
    temp_dir.path().join(db_file_name).to_string_lossy().to_string()
}
