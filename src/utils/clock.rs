use chrono::{DateTime, Duration, Utc};

pub const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// Returns the current timestamp in seconds.
///
/// # Panics
///
/// This function should never panic unless the current timestamp from the
/// time library is negative, which should never happen.
#[must_use]
pub fn now() -> u64 {
    u64::try_from(chrono::prelude::Utc::now().timestamp()).expect("timestamp should be positive")
}

/// Returns the datetime some seconds ago.
#[must_use]
pub fn seconds_ago_utc(seconds: i64) -> DateTime<chrono::Utc> {
    Utc::now() - Duration::seconds(seconds)
}

/// Returns the current time in database format.
///
/// For example: `2024-03-12 15:56:24`.
#[must_use]
pub fn datetime_now() -> String {
    Utc::now().format(DATETIME_FORMAT).to_string()
}
