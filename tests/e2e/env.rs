use crate::e2e::client::Client;
use crate::e2e::connection_info::anonymous_connection;

pub struct TestEnv {
    pub authority: String,
}

impl TestEnv {
    pub fn guess_client(&self) -> Client {
        Client::new(anonymous_connection(&self.authority))
    }
}

impl Default for TestEnv {
    fn default() -> Self {
        Self {
            authority: "localhost:3000".to_string(),
        }
    }
}
