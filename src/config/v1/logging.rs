use std::fmt;

use serde::{Deserialize, Serialize};
use tracing::level_filters::LevelFilter;

/// Core configuration for the API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Logging {
    /// Logging level. Possible values are: `Off`, `Error`, `Warn`, `Info`, `Debug`, `Trace`.
    #[serde(default = "Logging::default_log_level")]
    pub log_level: LogLevel,
}

impl Default for Logging {
    fn default() -> Self {
        Self {
            log_level: Logging::default_log_level(),
        }
    }
}

impl Logging {
    fn default_log_level() -> LogLevel {
        LogLevel::Info
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// A level lower than all log levels.
    Off,
    /// Corresponds to the `Error` log level.
    Error,
    /// Corresponds to the `Warn` log level.
    Warn,
    /// Corresponds to the `Info` log level.
    Info,
    /// Corresponds to the `Debug` log level.
    Debug,
    /// Corresponds to the `Trace` log level.
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_str = match self {
            LogLevel::Off => "off",
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        };
        write!(f, "{display_str}")
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(log_level: LogLevel) -> Self {
        match log_level {
            LogLevel::Off => LevelFilter::OFF,
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Trace => LevelFilter::TRACE,
        }
    }
}
