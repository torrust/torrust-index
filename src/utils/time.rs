pub fn current_time() -> u64 {
    chrono::prelude::Utc::now().timestamp() as u64
}
