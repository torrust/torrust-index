use std::env;

use torrust_index::databases::database;
use torrust_index::web::api::Version;

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

    /// Provides a database connect URL to connect to the database. For example:
    ///
    /// `sqlite://storage/database/torrust_index_e2e_testing.db?mode=rwc`.
    ///
    /// It's used to run SQL queries against the database needed for some tests.
    pub fn database_connect_url(&self) -> Option<String> {
        let internal_connect_url = self
            .starting_settings
            .as_ref()
            .map(|settings| settings.database.connect_url.clone());

        match self.state() {
            State::RunningShared => {
                if let Some(db_path) = internal_connect_url {
                    let maybe_db_driver = database::get_driver(&db_path);

                    return match maybe_db_driver {
                        Ok(db_driver) => match db_driver {
                            database::Driver::Sqlite3 => Some(db_path),
                            database::Driver::Mysql => Some(Self::overwrite_mysql_host(&db_path, "localhost")),
                        },
                        Err(_) => None,
                    };
                }
                None
            }
            State::RunningIsolated => internal_connect_url,
            State::Stopped => None,
        }
    }

    /// It overrides the "Host" in a `SQLx` database connection URL. For example:
    ///
    /// For:
    ///
    /// `mysql://root:root_secret_password@mysql:3306/torrust_index_e2e_testing`.
    ///
    /// It changes the `mysql` host name to `localhost`:
    ///
    /// `mysql://root:root_secret_password@localhost:3306/torrust_index_e2e_testing`.
    ///
    /// For E2E tests, we use docker compose, internally the index connects to
    /// the database using the "mysql" host, which is the docker compose service
    /// name, but tests connects directly to the localhost since the `MySQL`
    /// is exposed to the host.
    fn overwrite_mysql_host(db_path: &str, new_host: &str) -> String {
        db_path.replace("@mysql:", &format!("@{new_host}:"))
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
