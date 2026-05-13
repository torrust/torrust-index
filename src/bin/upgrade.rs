//! Upgrade command.
//! It updates the application from version v1.0.0 to v2.0.0.
//! You can execute it with: `cargo run --bin upgrade ./data.db ./data_v2.db ./uploads`.
//!
//! ADR-T-010 classifies this as a side-effect command: the target contract is
//! empty stdout and JSON diagnostics on stderr. The current implementation is a
//! legacy output gap until migration.

use torrust_index::upgrades::from_v1_0_0_to_v2_0_0::upgrader::run;

#[tokio::main]
async fn main() {
    run().await;
}
