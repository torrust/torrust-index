//! Minimal health-check binary for Torrust Index containers.
//!
//! On success (exit 0), emits a JSON object to stdout per P9.
//! On failure (exit ≠ 0), stdout is empty; diagnostics go to stderr.

use std::process::ExitCode;

use clap::Parser;
use torrust_index_cli_common::{BaseArgs, emit, init_json_tracing, refuse_if_stdout_is_tty};
use torrust_index_health_check::do_health_check;
use tracing::error;

#[derive(Parser)]
#[command(
    name = "torrust-index-health-check",
    about = "Minimal health-check for Torrust Index containers"
)]
struct Args {
    /// URL to check, e.g. `http://localhost:3001/health_check`.
    url: String,

    #[command(flatten)]
    base: BaseArgs,
}

fn main() -> ExitCode {
    let args = Args::parse();
    init_json_tracing(if args.base.debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    });
    refuse_if_stdout_is_tty("torrust-index-health-check");

    match do_health_check(&args.url) {
        Ok(out) => match emit(&out) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                // Writing the JSON object to stdout failed (e.g. broken
                // pipe when the consumer has already exited). Surface
                // the failure through the documented exit-code contract
                // rather than letting Rust's panic handler set its own.
                error!(error = %e, "failed to write health-check output to stdout");
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            error!(error = %e, "health check failed");
            ExitCode::FAILURE
        }
    }
}
