#[must_use]
pub fn now() -> u64 {
    u64::try_from(chrono::prelude::Utc::now().timestamp()).expect("timestamp should be positive")
}
