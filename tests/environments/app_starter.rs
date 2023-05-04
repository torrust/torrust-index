use std::net::SocketAddr;

use log::info;
use tokio::sync::{oneshot, RwLock};
use torrust_index_backend::app;
use torrust_index_backend::config::{Configuration, TorrustConfig};

/// It launches the app and provides a way to stop it.
pub struct AppStarter {
    configuration: TorrustConfig,
    /// The application binary state (started or not):
    ///  - `None`: if the app is not started,
    ///  - `RunningState`: if the app was started.
    running_state: Option<RunningState>,
}

impl AppStarter {
    #[must_use]
    pub fn with_custom_configuration(configuration: TorrustConfig) -> Self {
        Self {
            configuration,
            running_state: None,
        }
    }

    /// # Panics
    ///
    /// Will panic if the app was dropped after spawning it.
    pub async fn start(&mut self) {
        let configuration = Configuration {
            settings: RwLock::new(self.configuration.clone()),
        };

        // Open a channel to communicate back with this function
        let (tx, rx) = oneshot::channel::<AppStarted>();

        // Launch the app in a separate task
        let app_handle = tokio::spawn(async move {
            let app = app::run(configuration).await;

            // Send the socket address back to the main thread
            tx.send(AppStarted {
                socket_addr: app.socket_address,
            })
            .expect("the app starter should not be dropped");

            app.api_server.await
        });

        // Wait until the app is started
        let socket_addr = match rx.await {
            Ok(msg) => msg.socket_addr,
            Err(e) => panic!("the app was dropped: {e}"),
        };

        let running_state = RunningState { app_handle, socket_addr };

        info!("Test environment started. Listening on {}", running_state.socket_addr);

        // Update the app state
        self.running_state = Some(running_state);
    }

    pub fn stop(&mut self) {
        match &self.running_state {
            Some(running_state) => {
                running_state.app_handle.abort();
                self.running_state = None;
            }
            None => {}
        }
    }

    #[must_use]
    pub fn server_configuration(&self) -> TorrustConfig {
        self.configuration.clone()
    }

    #[must_use]
    pub fn server_socket_addr(&self) -> Option<SocketAddr> {
        self.running_state.as_ref().map(|running_state| running_state.socket_addr)
    }

    #[must_use]
    pub fn database_connect_url(&self) -> String {
        self.configuration.database.connect_url.clone()
    }
}

#[derive(Debug)]
pub struct AppStarted {
    pub socket_addr: SocketAddr,
}

/// Stores the app state when it is running.
pub struct RunningState {
    app_handle: tokio::task::JoinHandle<std::result::Result<(), std::io::Error>>,
    pub socket_addr: SocketAddr,
}

impl Drop for AppStarter {
    /// Child threads spawned with `tokio::spawn()` and tasks spawned with
    /// `async { }` blocks will not be automatically killed when the owner of
    /// the struct that spawns them goes out of scope.
    ///
    /// The `tokio::spawn()` function and `async { }` blocks create an
    /// independent task that runs on a separate thread or the same thread,
    /// respectively. The task will continue to run until it completes, even if
    /// the owner of the struct that spawned it goes out of scope.
    ///
    /// However, it's important to note that dropping the owner of the struct
    /// may cause the task to be orphaned, which means that the task is no
    /// longer associated with any parent task or thread. Orphaned tasks can
    /// continue running in the background, consuming system resources, and may
    /// eventually cause issues if left unchecked.
    ///
    /// To avoid orphaned tasks, we ensure that the app ois stopped when the
    /// owner of the struct goes out of scope.
    ///
    /// This avoids having to call `TestEnv::stop()` explicitly at the end of
    /// each test.
    fn drop(&mut self) {
        // Stop the app when the owner of the struct goes out of scope
        self.stop();
    }
}
