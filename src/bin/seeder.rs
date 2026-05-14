//! Program to upload random torrents to a live Index API.
//!
//! ADR-T-010 classifies this as a side-effect command: stdout remains empty and
//! diagnostics are JSON records on stderr. Scripts should branch on the process
//! exit code and parse stderr as NDJSON when they need diagnostics.
use std::process::ExitCode;

use torrust_index::console::commands::seeder::app::{self, Args};
use torrust_index_cli_common::{install_json_panic_hook, parse_args_or_exit, run_no_stdout_command_async};

const COMMAND_NAME: &str = "seeder";

#[tokio::main]
async fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);

    let args = parse_args_or_exit::<Args>();
    let debug = args.base.debug;

    run_no_stdout_command_async(COMMAND_NAME, debug, tracing::Level::INFO, || app::run(args)).await
}
