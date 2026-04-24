//! Resolve the Torrust Index configuration and print the
//! container-relevant subset as a single JSON object on stdout.
//!
//! See ADR-T-009 §D3 and implementation plan §6.1.

use std::process::ExitCode;

use clap::Parser;
use torrust_index_cli_common::{BaseArgs, emit, init_json_tracing, refuse_if_stdout_is_tty};
use torrust_index_config::{DEFAULT_CONFIG_TOML_PATH, Info, load_settings};
use torrust_index_config_probe::{ProbeError, probe};
use tracing::error;

#[derive(Parser)]
#[command(
    name = "torrust-index-config-probe",
    about = "Emit the container-relevant subset of the resolved Torrust Index configuration as JSON"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,
}

fn main() -> ExitCode {
    install_panic_hook();

    let args = Args::parse();
    refuse_if_stdout_is_tty("torrust-index-config-probe");
    init_json_tracing(if args.base.debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    });

    // `Info::from_env` is the JSON-safe sibling of `Info::new`:
    // it reads the same env vars but skips the diagnostic
    // `println!`s that would corrupt our stdout-only contract.
    let info = Info::from_env(DEFAULT_CONFIG_TOML_PATH);

    let settings = match load_settings(&info) {
        Ok(s) => s,
        Err(e) => {
            error!(error = %e, "failed to load configuration");
            return ExitCode::from(3);
        }
    };

    match probe(&settings) {
        Ok(out) => {
            if let Err(e) = emit(&out) {
                error!(error = %e, "failed to write JSON to stdout");
                return ExitCode::from(1);
            }
            ExitCode::SUCCESS
        }
        Err(ProbeError::EmptyTrackerToken) => {
            error!("tracker.token is empty — refusing to start");
            ExitCode::from(4)
        }
        Err(ProbeError::UnsupportedScheme(s)) => {
            error!(scheme = %s, "unsupported scheme");
            ExitCode::from(5)
        }
    }
}

/// Map any unhandled panic to exit code 1 so the contract in
/// implementation plan §6.1 ("1 — Internal error (unhandled
/// panic, unexpected I/O)") is honoured. Without this hook a
/// panic would exit with Rust's default 101.
fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Preserve the default formatted backtrace on stderr,
        // then exit with the documented code.
        default(info);
        std::process::exit(1);
    }));
}
