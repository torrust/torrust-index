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
///
/// `Configuration` no longer implements `Default`. After ADR-T-009
/// §D2, `tracker.token` and `database.connect_url` are mandatory at
/// the schema level; there is no `Settings::default()` to build a
/// runtime wrapper from. Construct one via [`Configuration::load`]
/// with an [`Info`], or use `Configuration::for_tests` (test-only)
/// in unit tests.
#[derive(Debug)]
pub struct Configuration {
    /// The state of the configuration.
    pub settings: RwLock<Settings>,
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

#[cfg(test)]
impl Configuration {
    /// Build a `Configuration` from the shared placeholder TOML for
    /// use in tests. Replaces the previous `Configuration::default()`
    /// — `Settings` no longer carries an `impl Default` (ADR-T-009 §D2).
    ///
    /// The TOML literal lives in
    /// [`torrust_index_config::test_helpers::PLACEHOLDER_TOML`] so
    /// every crate-boundary test fixture stays in sync.
    ///
    /// # Hermeticity caveat
    ///
    /// This is **not** hermetic with respect to the ambient process
    /// environment. `Configuration::load` ultimately calls
    /// [`torrust_index_config::load_settings`], which always merges
    /// any `TORRUST_INDEX_CONFIG_OVERRIDE_*` (and `TORRUST_INDEX_CONFIG_TOML[_PATH]`)
    /// variables on top of the placeholder TOML via figment's `Env`
    /// provider. E2E runner scripts and developer shells routinely
    /// export these (notably
    /// `TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL`), so a
    /// caller that asserts on placeholder values must scrub the
    /// environment first — typically by wrapping the test in
    /// `figment::Jail::expect_with(|_| { clear_inherited_config_env();
    /// … })`, as `src/tests/config/mod.rs` does. Without that guard,
    /// `for_tests()` silently picks up whatever overrides happen to be
    /// in scope.
    #[must_use]
    pub(crate) fn for_tests() -> Self {
        Self::load(&Info::from_toml(torrust_index_config::test_helpers::PLACEHOLDER_TOML)).expect("placeholder TOML must load")
    }
}
