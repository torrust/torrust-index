//! Configuration for the application.
//!
//! Re-exports the parsing surface from the
//! [`torrust_index_config`] crate (extracted per ADR-T-009 phase 3)
//! and adds the runtime [`Configuration`] wrapper that holds
//! settings under a `tokio::sync::RwLock`.
//!
//! Existing `use crate::config::Settings;` (and similar) call
//! sites continue to compile via the wildcard re-export below.
use tokio::sync::RwLock;
pub use torrust_index_config::*;

/// The configuration service.
#[derive(Debug)]
pub struct Configuration {
    /// The state of the configuration.
    pub settings: RwLock<Settings>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            settings: RwLock::new(Settings::default()),
        }
    }
}

impl Configuration {
    /// Loads the configuration from the [`Info`] struct.
    ///
    /// # Errors
    ///
    /// Will return `Err` if the environment variable does not exist
    /// or has a bad configuration.
    pub fn load(info: &Info) -> Result<Self, Error> {
        let settings = load_settings(info)?;

        Ok(Self {
            settings: RwLock::new(settings),
        })
    }

    /// Loads the settings from the [`Info`] struct.
    ///
    /// Thin wrapper retained for backwards compatibility with call
    /// sites that previously called `Configuration::load_settings`.
    /// New code should call [`torrust_index_config::load_settings`]
    /// directly.
    ///
    /// # Errors
    ///
    /// Will return `Err` if the environment variable does not exist
    /// or has a bad configuration.
    pub fn load_settings(info: &Info) -> Result<Settings, Error> {
        load_settings(info)
    }

    pub async fn get_all(&self) -> Settings {
        let settings_lock = self.settings.read().await;

        settings_lock.clone()
    }

    pub async fn get_site_name(&self) -> String {
        let settings_lock = self.settings.read().await;

        settings_lock.website.name.clone()
    }

    pub async fn get_api_base_url(&self) -> Option<String> {
        let settings_lock = self.settings.read().await;
        settings_lock.net.base_url.as_ref().map(std::string::ToString::to_string)
    }
}
