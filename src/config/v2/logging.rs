use std::fmt;

use serde::{Deserialize, Serialize};
use tracing::level_filters::LevelFilter;

/// Core configuration for the API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Logging {
    /// Logging level. Possible values are: `Off`, `Error`, `Warn`, `Info`, `Debug`, `Trace`.
    #[serde(default = "Logging::default_threshold")]
    pub threshold: Threshold,
}

impl Default for Logging {
    fn default() -> Self {
        Self {
            threshold: Logging::default_threshold(),
        }
    }
}

impl Logging {
    fn default_threshold() -> Threshold {
        Threshold::Info
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Threshold {
    /// A level lower than all log security levels.
    Off,
    /// Corresponds to the `Error` log security level.
    Error,
    /// Corresponds to the `Warn` log security level.
    Warn,
    /// Corresponds to the `Info` log security level.
    Info,
    /// Corresponds to the `Debug` log security level.
    Debug,
    /// Corresponds to the `Trace` log security level.
    Trace,
}

impl Default for Threshold {
    fn default() -> Self {
        Self::Info
    }
}

impl fmt::Display for Threshold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_str = match self {
            Threshold::Off => "off",
            Threshold::Error => "error",
            Threshold::Warn => "warn",
            Threshold::Info => "info",
            Threshold::Debug => "debug",
            Threshold::Trace => "trace",
        };
        write!(f, "{display_str}")
    }
}

impl From<Threshold> for LevelFilter {
    fn from(threshold: Threshold) -> Self {
        match threshold {
            Threshold::Off => LevelFilter::OFF,
            Threshold::Error => LevelFilter::ERROR,
            Threshold::Warn => LevelFilter::WARN,
            Threshold::Info => LevelFilter::INFO,
            Threshold::Debug => LevelFilter::DEBUG,
            Threshold::Trace => LevelFilter::TRACE,
        }
    }
}
