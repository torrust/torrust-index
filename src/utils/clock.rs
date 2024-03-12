use chrono::Utc;

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

/// Returns the current time in database format.
///
/// For example: `2024-03-12 15:56:24`.
#[must_use]
pub fn datetime_now() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
