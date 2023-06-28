use std::env;

use torrust_index_backend::web::api::Version;

use super::config::{init_shared_env_configuration, ENV_VAR_E2E_SHARED};
use crate::common::contexts::settings::Settings;
use crate::environments::{isolated, shared};

enum State {
    Stopped,
    RunningShared,
    RunningIsolated,
}

/// Test environment for E2E tests. It's a wrapper around the shared or isolated
/// test environment.
///
/// Shared test environment:
///
/// - It's a out-of-process test environment.
/// - It has to be started before running the tests.
/// - All tests run against the same instance of the server.
///
/// Isolated test environment:
///
/// - It's an in-process test environment.
/// - It's started automatically when the test starts.
/// - Each test runs against a different instance of the server.
#[derive(Default)]
pub struct TestEnv {
    /// Copy of the settings when the test environment was started.
    starting_settings: Option<Settings>,
    /// Shared independent test environment if we start using it.
    shared: Option<shared::TestEnv>,
    /// Isolated test environment if we start an isolate test environment.
    isolated: Option<isolated::TestEnv>,
}

impl TestEnv {
    // code-review: consider extracting a trait for test environments. The state
    // could be only `Running` or `Stopped`, and we could have a single
    // attribute with the current started test environment (`Option<RunningTEstEnv>`).

    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_shared(&self) -> bool {
        self.shared.is_some()
    }

    pub fn is_isolated(&self) -> bool {
        self.isolated.is_some()
    }

    /// It starts the test environment. It can be a shared or isolated test
    /// environment depending on the value of the `ENV_VAR_E2E_SHARED` env var.
    pub async fn start(&mut self, api_version: Version) {
        let e2e_shared = ENV_VAR_E2E_SHARED; // bool

        if let Ok(_e2e_test_env_is_shared) = env::var(e2e_shared) {
            // Using the shared test env.
            let shared_env = shared::TestEnv::running().await;

            self.shared = Some(shared_env);
            self.starting_settings = self.server_settings_for_shared_env().await;
        } else {
            // Using an isolated test env.
            let isolated_env = isolated::TestEnv::running(api_version).await;

            self.isolated = Some(isolated_env);
            self.starting_settings = self.server_settings_for_isolated_env();
        }
    }

    /// Some test requires the real tracker to be running, so they can only
    /// be run in shared mode.
    pub fn provides_a_tracker(&self) -> bool {
        self.is_shared()
    }

    /// Returns the server starting settings if the servers was already started.
    /// We do not know the settings until we start the server.
    pub fn server_settings(&self) -> Option<Settings> {
        self.starting_settings.as_ref().cloned()
    }

    /// Provides the API server socket address.
    /// For example: `localhost:3001`.
    pub fn server_socket_addr(&self) -> Option<String> {
        match self.state() {
            State::RunningShared => self.shared.as_ref().unwrap().server_socket_addr(),
            State::RunningIsolated => self.isolated.as_ref().unwrap().server_socket_addr(),
            State::Stopped => None,
        }
    }

    /// Provides the database connect URL.
    /// For example: `sqlite://storage/database/torrust_index_backend_e2e_testing.db?mode=rwc`.
    pub fn database_connect_url(&self) -> Option<String> {
        self.starting_settings
            .as_ref()
            .map(|settings| settings.database.connect_url.clone())
    }

    fn state(&self) -> State {
        if self.is_shared() {
            return State::RunningShared;
        }

        if self.is_isolated() {
            return State::RunningIsolated;
        }

        State::Stopped
    }

    fn server_settings_for_isolated_env(&self) -> Option<Settings> {
        self.isolated
            .as_ref()
            .map(|env| Settings::from(env.app_starter.server_configuration()))
    }

    async fn server_settings_for_shared_env(&self) -> Option<Settings> {
        let configuration = init_shared_env_configuration().await;
        let settings = configuration.settings.read().await;
        Some(Settings::from(settings.clone()))
    }
}
