//! Upgrade command.
//! It updates the application from version v1.0.0 to v2.0.0.
//! You can execute it with: `cargo run --bin upgrade ./data.db ./data_v2.db ./uploads`.
//!
//! ADR-T-010 classifies this as a side-effect command: stdout remains empty and
//! diagnostics are JSON records on stderr.
use std::process::ExitCode;

use clap::Parser;
use torrust_index::upgrades::from_v1_0_0_to_v2_0_0::upgrader::{Arguments, run};
use torrust_index_cli_common::{BaseArgs, install_json_panic_hook, parse_args_or_exit, run_no_stdout_command_async};

const COMMAND_NAME: &str = "upgrade";

#[derive(Parser)]
#[command(name = "upgrade", version, about = "Upgrade Torrust Index data from v1.0.0 to v2.0.0")]
struct Args {
    /// Source database file in version v1.0.0.
    source_database_file: String,

    /// Target database file for version v2.0.0 data.
    target_database_file: String,

    /// Directory where v1.0.0 torrent files are stored.
    upload_path: String,

    #[command(flatten)]
    base: BaseArgs,
}

#[tokio::main]
async fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);

    let args = parse_args_or_exit::<Args>();
    let debug = args.base.debug;
    let command_args = Arguments {
        source_database_file: args.source_database_file,
        target_database_file: args.target_database_file,
        upload_path: args.upload_path,
    };

    run_no_stdout_command_async(COMMAND_NAME, debug, tracing::Level::INFO, || run(command_args)).await
}
