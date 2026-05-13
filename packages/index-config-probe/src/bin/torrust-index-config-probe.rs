//! Resolve the Torrust Index configuration and print the
//! container-relevant subset as a single JSON object on stdout.
//! The JSON object contains `schema`, `database`, and `auth`.
//! Direct terminal stdout is refused; pipe or redirect the result.
//!
//! See ADR-T-009 §D3.

use std::process::ExitCode;

use clap::Parser;
use torrust_index_cli_common::{
    BaseArgs, emit, init_json_tracing_with_debug, install_json_panic_hook, parse_args_or_exit, refuse_if_stdout_is_tty,
};
use torrust_index_config::{DEFAULT_CONFIG_TOML_PATH, Info, load_settings};
use torrust_index_config_probe::{ProbeError, probe};
use tracing::error;

const COMMAND_NAME: &str = "torrust-index-config-probe";

#[derive(Parser)]
#[command(
    name = "torrust-index-config-probe",
    version,
    about = "Emit the container-relevant subset of the resolved Torrust Index configuration as JSON"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,
}

fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);

    let args = parse_args_or_exit::<Args>();
    init_json_tracing_with_debug(args.base.debug, tracing::Level::INFO);
    refuse_if_stdout_is_tty(COMMAND_NAME);

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

#[cfg(test)]
mod tests {
    //! # Config-probe binary CLI contract tests
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
        assert!(text.contains("Emit the container-relevant subset"));
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
