//! Program to upload random torrents to a live Index API.
//!
//! ADR-T-010 classifies this as a side-effect command: the target contract is
//! empty stdout and JSON diagnostics on stderr. The current implementation is a
//! legacy output gap until migration.
use std::process::ExitCode;

use torrust_index::console::commands::seeder::app;
use torrust_index_cli_common::{CommandExit, install_json_panic_hook};
use tracing::error;

const COMMAND_NAME: &str = "seeder";

#[tokio::main]
async fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);

    match app::run().await {
        Ok(()) => CommandExit::Success.exit_code(),
        Err(error) => {
            error!(%error, "command failed");
            CommandExit::Failure.exit_code()
        }
    }
}
