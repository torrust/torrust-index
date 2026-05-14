//! Import Tracker Statistics command.
//!
//! It imports the number of seeders and leechers for all torrents from the linked tracker.
//!
//! You can execute it with:
//!
//! ```text
//! cargo run --quiet --bin import_tracker_statistics -- 2>import-tracker-statistics.ndjson
//! jq . import-tracker-statistics.ndjson
//! ```
//!
//! ADR-T-010 classifies this as a side-effect command: stdout remains empty and
//! diagnostics are JSON records on stderr. Scripts should branch on the process
//! exit code and parse stderr as NDJSON when they need diagnostics.
use std::process::ExitCode;

use clap::Parser;
use torrust_index::console::commands::tracker_statistics_importer::app::run;
use torrust_index_cli_common::{BaseArgs, install_json_panic_hook, parse_args_or_exit, run_no_stdout_command_async};

const COMMAND_NAME: &str = "import_tracker_statistics";

#[derive(Parser)]
#[command(
    name = "import_tracker_statistics",
    version,
    about = "Import torrent statistics from the linked tracker"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,
}

#[tokio::main]
async fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);

    let args = parse_args_or_exit::<Args>();

    run_no_stdout_command_async(COMMAND_NAME, args.base.debug, tracing::Level::INFO, run).await
}
