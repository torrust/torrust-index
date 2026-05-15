//! Minimal health-check binary for Torrust Index containers.
//!
//! On success (exit 0), emits one JSON object with `schema`, `target`, `status`,
//! and `elapsed_ms` to stdout per ADR-T-010.
//! On failure (exit ≠ 0), stdout is empty; diagnostics go to stderr as JSON.
//! Direct terminal stdout is refused; pipe or redirect the result.

use std::process::ExitCode;

use clap::Parser;
use torrust_index_cli_common::{BaseArgs, install_json_panic_hook, parse_args_or_exit, run_stdout_json_command};
use torrust_index_health_check::{HealthCheckError, HealthCheckOutput, do_health_check};

const COMMAND_NAME: &str = "torrust-index-health-check";

#[derive(Parser)]
#[command(
    name = "torrust-index-health-check",
    version,
    about = "Minimal health-check for Torrust Index containers"
)]
struct Args {
    /// URL to check, e.g. `http://localhost:3001/health_check`.
    url: String,

    #[command(flatten)]
    base: BaseArgs,
}

fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);

    let args = parse_args_or_exit::<Args>();

    run_stdout_json_command::<HealthCheckOutput, HealthCheckError, _>(COMMAND_NAME, args.base.debug, tracing::Level::INFO, || {
        do_health_check(&args.url)
    })
}

#[cfg(test)]
mod tests {
    //! # Health-check binary CLI contract tests
    //!
    //! | Test                                  | What it covers                         |
    //! |---------------------------------------|----------------------------------------|
    //! | `help_is_json_control_record`         | `--help` is wrapped as JSON metadata   |
    //! | `version_is_json_control_record`      | `--version` is wrapped as JSON metadata|
    //! | `usage_error_is_json_control_record`  | argv errors become JSON usage records  |

    use torrust_index_cli_common::{CommandExit, ControlPlaneFields, ControlPlaneRecordKind, parse_args_from};

    use super::{Args, COMMAND_NAME};

    #[test]
    fn help_is_json_control_record() {
        let Err(exit) = parse_args_from::<Args, _, _>([COMMAND_NAME, "--help"]) else {
            panic!("help should stop parsing");
        };

        assert_eq!(exit.exit, CommandExit::Success);
        assert_eq!(exit.record.command, COMMAND_NAME);
        assert_eq!(exit.record.kind, ControlPlaneRecordKind::Help);

        let Some(ControlPlaneFields::Help { text }) = exit.record.fields else {
            panic!("help record should carry help text");
        };
        assert!(text.contains("Minimal health-check"));
        assert!(text.contains("--debug"));
    }

    #[test]
    fn version_is_json_control_record() {
        let Err(exit) = parse_args_from::<Args, _, _>([COMMAND_NAME, "--version"]) else {
            panic!("version should stop parsing");
        };

        assert_eq!(exit.exit, CommandExit::Success);
        assert_eq!(exit.record.command, COMMAND_NAME);
        assert_eq!(exit.record.kind, ControlPlaneRecordKind::Version);

        let Some(ControlPlaneFields::Version { version }) = exit.record.fields else {
            panic!("version record should carry version text");
        };
        assert_eq!(version, format!("{COMMAND_NAME} {}", env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn usage_error_is_json_control_record() {
        let Err(exit) = parse_args_from::<Args, _, _>([COMMAND_NAME, "--no-such-flag"]) else {
            panic!("unknown flags should stop parsing");
        };

        assert_eq!(exit.exit, CommandExit::Usage);
        assert_eq!(exit.record.command, COMMAND_NAME);
        assert_eq!(exit.record.kind, ControlPlaneRecordKind::UsageError);
        assert!(exit.record.message.contains("--no-such-flag"));

        let Some(ControlPlaneFields::UsageError {
            exit_code,
            clap_error_kind,
        }) = exit.record.fields
        else {
            panic!("usage record should carry usage fields");
        };
        assert_eq!(exit_code, CommandExit::Usage.code());
        assert_eq!(clap_error_kind, "unknown_argument");
    }
}
