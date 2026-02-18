use std::fmt;

use serde::{Deserialize, Serialize};
use tracing::level_filters::LevelFilter;

/// Core configuration for the API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Logging {
    /// Logging level. Possible values are: `Off`, `Error`, `Warn`, `Info`, `Debug`, `Trace`.
    #[serde(default = "Logging::default_threshold")]
    pub threshold: Threshold,
}

impl Default for Logging {
    fn default() -> Self {
        Self {
            threshold: Self::default_threshold(),
        }
    }
}

impl Logging {
    const fn default_threshold() -> Threshold {
        Threshold::Info
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum Threshold {
    /// A level lower than all log security levels.
    Off,
    /// Corresponds to the `Error` log security level.
    Error,
    /// Corresponds to the `Warn` log security level.
    Warn,
    /// Corresponds to the `Info` log security level.
    #[default]
    Info,
    /// Corresponds to the `Debug` log security level.
    Debug,
    /// Corresponds to the `Trace` log security level.
    Trace,
}

impl fmt::Display for Threshold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_str = match self {
            Self::Off => "off",
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        };
        write!(f, "{display_str}")
    }
}

impl From<Threshold> for LevelFilter {
    fn from(threshold: Threshold) -> Self {
        match threshold {
            Threshold::Off => Self::OFF,
            Threshold::Error => Self::ERROR,
            Threshold::Warn => Self::WARN,
            Threshold::Info => Self::INFO,
            Threshold::Debug => Self::DEBUG,
            Threshold::Trace => Self::TRACE,
        }
    }
}
