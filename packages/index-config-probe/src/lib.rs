//! Resolves the Torrust Index configuration and emits the
//! container-relevant subset as JSON.
//!
//! The probe is a small bridge between the entry script (POSIX
//! shell, can't parse TOML or env-var overrides) and the
//! application's actual configuration loader. The shell pipes
//! the probe's stdout into `jq`; the future Rust entry binary
//! will deserialise the same JSON via `serde_json`.
//!
//! ADR-T-009 §D3 (config probe helper).

use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use torrust_index_config::{Auth, Settings, Tracker};
use url::Url;

#[cfg(test)]
mod tests;

/// Schema version of the JSON output. Incremented on breaking
/// changes (per ADR-T-009 §D3).
pub const SCHEMA: u32 = 1;

/// Container-relevant resolved configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Probe {
    pub schema: u32,
    pub database: DatabaseProbe,
    pub auth: AuthProbe,
}

/// Resolved database settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatabaseProbe {
    /// URL scheme extracted from `connect_url`. Not the
    /// Containerfile's `TORRUST_INDEX_DATABASE_DRIVER` env var
    /// (which uses `sqlite3`/`mysql`).
    pub driver: Driver,
    /// File path for `sqlite` URLs (absolute, relative, or
    /// `:memory:`); `null` for non-`sqlite` schemes.
    pub path: Option<String>,
}

/// The set of database URL schemes the entry script knows.
///
/// Mirrors the schemes recognised by the application's own
/// `databases::database::get_driver` (`sqlite`, `mysql`).
/// Anything outside this set produces
/// [`ProbeError::UnsupportedScheme`] before this enum is ever
/// constructed, so the dispatch table is enforced at the type
/// level rather than via stringly-typed `match`es.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Driver {
    Sqlite,
    Mysql,
}

/// Resolved auth-key settings for both the private and public
/// halves of the JWT keypair.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthProbe {
    pub private_key: AuthKeyProbe,
    pub public_key: AuthKeyProbe,
}

/// Resolved settings for a single auth key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthKeyProbe {
    /// Raw presence (non-empty after resolution) of the inline
    /// PEM field, before PEM-overrides-PATH precedence.
    pub pem_set: bool,
    /// Raw presence of the path field.
    pub path_set: bool,
    /// Winner after precedence: `pem`, `path`, or `none`.
    pub source: AuthKeySource,
    /// Resolved path if `source` is `path`; `null` otherwise.
    pub path: Option<String>,
}

/// Which delivery mechanism wins the per-key precedence.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuthKeySource {
    Pem,
    Path,
    None,
}

/// Errors raised by the probe after the application's loader has
/// already returned a `Settings` value.
#[derive(Debug)]
pub enum ProbeError {
    /// `tracker.token` deserialised to an empty string. The
    /// application's `ApiToken::new` panics on empty input but
    /// `#[derive(Deserialize)]` bypasses that guard, so we reject
    /// at the container boundary instead.
    EmptyTrackerToken,
    /// `database.connect_url` uses a scheme the entry script does
    /// not know how to dispatch on (anything other than `sqlite`
    /// or `mysql`).
    UnsupportedScheme(String),
}

impl std::fmt::Display for ProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyTrackerToken => f.write_str("tracker.token is empty"),
            Self::UnsupportedScheme(s) => write!(f, "unsupported scheme: {s}"),
        }
    }
}

impl std::error::Error for ProbeError {}

/// Resolve the container-relevant subset of `settings` into the
/// JSON-shaped [`Probe`] value.
///
/// # Errors
///
/// Returns [`ProbeError::EmptyTrackerToken`] if `tracker.token`
/// is empty after deserialisation, or
/// [`ProbeError::UnsupportedScheme`] if `database.connect_url`'s
/// scheme is not one of `sqlite` or `mysql`.
pub fn probe(settings: &Settings) -> Result<Probe, ProbeError> {
    check_tracker_token(&settings.tracker)?;

    Ok(Probe {
        schema: SCHEMA,
        database: probe_database(&settings.database.connect_url)?,
        auth: probe_auth(&settings.auth),
    })
}

const fn check_tracker_token(tracker: &Tracker) -> Result<(), ProbeError> {
    if tracker.token.is_empty() {
        return Err(ProbeError::EmptyTrackerToken);
    }
    Ok(())
}

fn probe_database(connect_url: &Url) -> Result<DatabaseProbe, ProbeError> {
    let scheme = connect_url.scheme().to_ascii_lowercase();

    let (driver, path) = match scheme.as_str() {
        "sqlite" => (Driver::Sqlite, Some(extract_sqlite_path(connect_url))),
        "mysql" => (Driver::Mysql, None),
        other => return Err(ProbeError::UnsupportedScheme(other.to_string())),
    };

    Ok(DatabaseProbe { driver, path })
}

/// Extract the file path from a `sqlite://` (or `sqlite::memory:`)
/// URL. The shapes are spelled out in ADR-T-009 §D3.
fn extract_sqlite_path(url: &Url) -> String {
    // Opaque form (e.g. `sqlite::memory:`) — `cannot_be_a_base`
    // returns true and `path()` is the opaque body.
    if url.cannot_be_a_base() {
        return url.path().to_string();
    }

    // Authority form: `sqlite://data.db?mode=rwc` puts the
    // relative file in the host slot (`url`'s parser does not
    // recognise `sqlite` as a special scheme, so it accepts this
    // shape but exposes `data.db` as the host string).
    if let Some(host) = url.host_str()
        && !host.is_empty()
    {
        return host.to_string();
    }

    // Hierarchical form: `sqlite:///var/lib/...` — empty
    // authority, real path. Percent-decode so callers see
    // `My Data` and not `My%20Data`.
    //
    // POSIX paths are byte strings, not text. We assume sqlite
    // file paths in container deployments are valid UTF-8 (the
    // common case for declarative compose stacks); a non-UTF-8
    // byte sequence would be replaced with U+FFFD here. If a
    // future deployment needs raw-bytes fidelity, switch the
    // wire format to a base64-encoded byte string and surface a
    // schema bump.
    let raw_path = url.path();
    percent_decode_str(raw_path).decode_utf8_lossy().into_owned()
}

fn probe_auth(auth: &Auth) -> AuthProbe {
    AuthProbe {
        private_key: probe_auth_key(auth.private_key_pem.as_deref(), auth.private_key_path.as_deref()),
        public_key: probe_auth_key(auth.public_key_pem.as_deref(), auth.public_key_path.as_deref()),
    }
}

/// Resolve a single auth-key pair into the wire-format
/// [`AuthKeyProbe`].
///
/// **Empty-string-equals-absent semantics are defined here**,
/// not in the config crate's deserialiser: `Auth` stores the
/// fields as `Option<String>` and accepts the empty string at
/// the type level. The probe collapses both `None` *and*
/// `Some("")` into `*_set = false` because a bare `${VAR}` in a
/// compose file that substitutes to an empty value is
/// indistinguishable from "unset" at the container boundary.
fn probe_auth_key(pem: Option<&str>, path: Option<&str>) -> AuthKeyProbe {
    let pem_set = pem.is_some_and(|s| !s.is_empty());
    let path_set = path.is_some_and(|s| !s.is_empty());

    let (source, resolved_path) = if pem_set {
        (AuthKeySource::Pem, None)
    } else if path_set {
        (AuthKeySource::Path, path.map(str::to_string))
    } else {
        (AuthKeySource::None, None)
    };

    AuthKeyProbe {
        pem_set,
        path_set,
        source,
        path: resolved_path,
    }
}
