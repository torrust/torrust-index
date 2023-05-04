use std::env;

use crate::common::contexts::settings::{Auth, Database, ImageCache, Mail, Net, Settings, Tracker, Website};
use crate::environments::{self, isolated, shared};

enum State {
    Stopped,
    RunningShared,
    RunningIsolated,
}

pub struct TestEnv {
    mode: State,
    shared: Option<shared::TestEnv>,
    isolated: Option<isolated::TestEnv>,
}

impl TestEnv {
    // todo: this class needs a big refactor:
    // - It should load the `server_settings` rom both shared or isolated env.
    //   And `tracker_url`, `server_socket_addr`, `database_connect_url` methods
    //   should get the values from `server_settings`.
    // - We should consider extracting a trait for test environments, so we can
    //   only one attribute like `AppStarter`.

    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_isolated(&self) -> bool {
        matches!(self.mode, State::RunningIsolated)
    }

    pub async fn start(&mut self) {
        let e2e_shared = "TORRUST_IDX_BACK_E2E_SHARED"; // bool

        if let Ok(_val) = env::var(e2e_shared) {
            let env = shared::TestEnv::running().await;
            self.mode = State::RunningShared;
            self.shared = Some(env);
        }

        let isolated_env = isolated::TestEnv::running().await;
        self.mode = State::RunningIsolated;
        self.isolated = Some(isolated_env);
    }

    pub fn tracker_url(&self) -> String {
        // todo: get from `server_settings`
        match self.mode {
            // todo: for shared instance, get it from env var
            // `TORRUST_IDX_BACK_CONFIG` or `TORRUST_IDX_BACK_CONFIG_PATH`
            State::RunningShared => "udp://tracker:6969".to_string(),
            // todo
            State::RunningIsolated => "udp://localhost:6969".to_string(),
            State::Stopped => panic!("TestEnv is not running"),
        }
    }

    /// Some test requires the real tracker to be running, so they can only
    /// be run in shared mode.
    pub fn provides_a_tracker(&self) -> bool {
        matches!(self.mode, State::RunningShared)
    }

    pub fn server_socket_addr(&self) -> Option<String> {
        // todo: get from `server_settings`
        match self.mode {
            // todo: for shared instance, get it from env var
            // `TORRUST_IDX_BACK_CONFIG` or `TORRUST_IDX_BACK_CONFIG_PATH`
            State::RunningShared => match &self.shared {
                Some(env) => env.server_socket_addr(),
                None => panic!("TestEnv is not running"),
            },
            State::RunningIsolated => match &self.isolated {
                Some(env) => env.server_socket_addr(),
                None => panic!("TestEnv is not running"),
            },
            State::Stopped => panic!("TestEnv is not running"),
        }
    }

    pub fn database_connect_url(&self) -> Option<String> {
        // todo: get from `server_settings`
        match self.mode {
            State::Stopped => None,
            State::RunningShared => Some("sqlite://storage/database/torrust_index_backend_e2e_testing.db?mode=rwc".to_string()),
            State::RunningIsolated => self
                .isolated
                .as_ref()
                .map(environments::isolated::TestEnv::database_connect_url),
        }
    }

    pub fn server_settings(&self) -> Option<Settings> {
        // todo:
        // - For shared instance, get it from env var: `TORRUST_IDX_BACK_CONFIG` or `TORRUST_IDX_BACK_CONFIG_PATH`.
        // - For isolated instance, get it from the isolated env configuration (`TorrustConfig`).
        match self.mode {
            State::Stopped => None,
            State::RunningShared => Some(Settings {
                website: Website {
                    name: "Torrust".to_string(),
                },
                tracker: Tracker {
                    url: self.tracker_url(),
                    mode: "Public".to_string(),
                    api_url: "http://tracker:1212".to_string(),
                    token: "MyAccessToken".to_string(),
                    token_valid_seconds: 7_257_600,
                },
                net: Net {
                    port: 3000,
                    base_url: None,
                },
                auth: Auth {
                    email_on_signup: "Optional".to_string(),
                    min_password_length: 6,
                    max_password_length: 64,
                    secret_key: "MaxVerstappenWC2021".to_string(),
                },
                database: Database {
                    connect_url: self.database_connect_url().unwrap(),
                    torrent_info_update_interval: 3600,
                },
                mail: Mail {
                    email_verification_enabled: false,
                    from: "example@email.com".to_string(),
                    reply_to: "noreply@email.com".to_string(),
                    username: String::new(),
                    password: String::new(),
                    server: "mailcatcher".to_string(),
                    port: 1025,
                },
                image_cache: ImageCache {
                    max_request_timeout_ms: 1000,
                    capacity: 128_000_000,
                    entry_size_limit: 4_000_000,
                    user_quota_period_seconds: 3600,
                    user_quota_bytes: 64_000_000,
                },
            }),
            State::RunningIsolated => Some(Settings {
                website: Website {
                    name: "Torrust".to_string(),
                },
                tracker: Tracker {
                    url: self.tracker_url(),
                    mode: "Public".to_string(),
                    api_url: "http://localhost:1212".to_string(),
                    token: "MyAccessToken".to_string(),
                    token_valid_seconds: 7_257_600,
                },
                net: Net { port: 0, base_url: None },
                auth: Auth {
                    email_on_signup: "Optional".to_string(),
                    min_password_length: 6,
                    max_password_length: 64,
                    secret_key: "MaxVerstappenWC2021".to_string(),
                },
                database: Database {
                    connect_url: self.database_connect_url().unwrap(),
                    torrent_info_update_interval: 3600,
                },
                mail: Mail {
                    email_verification_enabled: false,
                    from: "example@email.com".to_string(),
                    reply_to: "noreply@email.com".to_string(),
                    username: String::new(),
                    password: String::new(),
                    server: String::new(),
                    port: 25,
                },
                image_cache: ImageCache {
                    max_request_timeout_ms: 1000,
                    capacity: 128_000_000,
                    entry_size_limit: 4_000_000,
                    user_quota_period_seconds: 3600,
                    user_quota_bytes: 64_000_000,
                },
            }),
        }
    }
}

impl Default for TestEnv {
    fn default() -> Self {
        Self {
            mode: State::Stopped,
            shared: None,
            isolated: None,
        }
    }
}
