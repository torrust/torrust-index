//! # `Info` env-var resolution tests
//!
//! `Info::new` and `Info::from_env` read process-global env
//! vars, so these tests must serialise access to those vars.
//! A single mutex guards every test in this file.
//!
//! | Test                                              | What it covers                                  |
//! |---------------------------------------------------|-------------------------------------------------|
//! | `from_env_uses_default_path_when_unset`           | Both env vars unset \u2192 fallback to default      |
//! | `from_env_overrides_path_when_set`                | `..._TOML_PATH` wins when non-empty             |
//! | `from_env_treats_empty_path_as_unset`             | Empty `..._TOML_PATH` falls back to default     |
//! | `from_env_loads_inline_toml`                      | `..._TOML` body is forwarded                    |
//! | `from_env_treats_empty_inline_toml_as_unset`      | Empty inline TOML is dropped (compose quirk)    |
//! | `new_returns_info_with_inline_toml`               | `Info::new` happy path                          |
//! | `new_with_only_path_var_set`                      | `Info::new` log branch \u2014 explicit path env var |
//! | `from_toml_round_trip`                            | `Info::from_toml` constructor shape             |
//! | `tls_default_is_all_none`                         | `Tls::default()` covers the derived `Default`   |
//!
//! Tests use a shared mutex to avoid races on
//! `TORRUST_INDEX_CONFIG_TOML*` between threads.

#![allow(
    unsafe_code,
    clippy::undocumented_unsafe_blocks,
    clippy::significant_drop_tightening,
    clippy::unused_self
)]

use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::{ENV_VAR_CONFIG_TOML, ENV_VAR_CONFIG_TOML_PATH, Info, Tls};

/// Serialise env-var manipulation across this file's tests.
fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Snapshot the two relevant env vars and restore them on drop.
struct EnvGuard {
    toml: Option<String>,
    path: Option<String>,
    _lock: MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn new() -> Self {
        let lock = env_lock();
        let toml = std::env::var(ENV_VAR_CONFIG_TOML).ok();
        let path = std::env::var(ENV_VAR_CONFIG_TOML_PATH).ok();
        // Clear so each test starts from a known state.
        // SAFETY: `set_var`/`remove_var` are sound here because
        // every test that touches these vars goes through
        // `env_lock`, which serialises mutation across the
        // whole test binary.
        // SAFETY: see comment above.
        unsafe {
            std::env::remove_var(ENV_VAR_CONFIG_TOML);
            std::env::remove_var(ENV_VAR_CONFIG_TOML_PATH);
        }
        Self { toml, path, _lock: lock }
    }

    fn set_toml(&self, value: &str) {
        // SAFETY: see `EnvGuard::new`.
        unsafe { std::env::set_var(ENV_VAR_CONFIG_TOML, value) };
    }

    fn set_path(&self, value: &str) {
        // SAFETY: see `EnvGuard::new`.
        unsafe { std::env::set_var(ENV_VAR_CONFIG_TOML_PATH, value) };
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        // SAFETY: see `EnvGuard::new`.
        unsafe {
            match &self.toml {
                Some(v) => std::env::set_var(ENV_VAR_CONFIG_TOML, v),
                None => std::env::remove_var(ENV_VAR_CONFIG_TOML),
            }
            match &self.path {
                Some(v) => std::env::set_var(ENV_VAR_CONFIG_TOML_PATH, v),
                None => std::env::remove_var(ENV_VAR_CONFIG_TOML_PATH),
            }
        }
    }
}

#[test]
fn from_env_uses_default_path_when_unset() {
    let _g = EnvGuard::new();
    let info = Info::from_env("/default/path.toml");
    assert!(info.config_toml.is_none());
    assert_eq!(info.config_toml_path, "/default/path.toml");
}

#[test]
fn from_env_overrides_path_when_set() {
    let g = EnvGuard::new();
    g.set_path("/explicit/path.toml");
    let info = Info::from_env("/default/path.toml");
    assert!(info.config_toml.is_none());
    assert_eq!(info.config_toml_path, "/explicit/path.toml");
}

#[test]
fn from_env_treats_empty_path_as_unset() {
    let g = EnvGuard::new();
    g.set_path("");
    let info = Info::from_env("/default/path.toml");
    assert_eq!(
        info.config_toml_path, "/default/path.toml",
        "an empty env var must collapse to the default (compose quirk)"
    );
}

#[test]
fn from_env_loads_inline_toml() {
    let g = EnvGuard::new();
    g.set_toml("[metadata]\nschema_version = \"2.0.0\"\n");
    let info = Info::from_env("/default/path.toml");
    assert_eq!(info.config_toml.as_deref(), Some("[metadata]\nschema_version = \"2.0.0\"\n"));
}

#[test]
fn from_env_treats_empty_inline_toml_as_unset() {
    let g = EnvGuard::new();
    g.set_toml("");
    let info = Info::from_env("/default/path.toml");
    assert!(
        info.config_toml.is_none(),
        "empty inline TOML must collapse to None so a bare `${{VAR}}` in compose does not clobber the file"
    );
}

#[test]
fn new_returns_info_with_inline_toml() {
    let g = EnvGuard::new();
    g.set_toml("[metadata]\nschema_version = \"2.0.0\"\n");
    let info = Info::new("/default/path.toml".to_string()).expect("Info::new is currently infallible");
    assert!(info.config_toml.is_some());
}

#[test]
fn new_with_only_path_var_set() {
    let g = EnvGuard::new();
    g.set_path("/explicit/path.toml");
    let info = Info::new("/default/path.toml".to_string()).expect("Info::new is currently infallible");
    assert!(info.config_toml.is_none());
    assert_eq!(info.config_toml_path, "/explicit/path.toml");
}

#[test]
fn new_with_no_env_vars_uses_default_path() {
    let _g = EnvGuard::new();
    let info = Info::new("/default/path.toml".to_string()).expect("Info::new is currently infallible");
    assert!(info.config_toml.is_none());
    assert_eq!(info.config_toml_path, "/default/path.toml");
}

#[test]
fn from_toml_round_trip() {
    // Doesn't touch the env at all \u2014 included here for
    // completeness alongside the other constructors.
    let info = Info::from_toml("body");
    assert_eq!(info.config_toml.as_deref(), Some("body"));
    assert_eq!(info.config_toml_path, "");
}

#[test]
fn tls_default_is_all_none() {
    let tls = Tls::default();
    assert!(tls.ssl_cert_path.is_none());
    assert!(tls.ssl_key_path.is_none());
}
