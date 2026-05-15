//! Generate an RSA-2048 key pair for JWT authentication.
//!
//! Emits one JSON object with `schema`, `private_key_pem`, and `public_key_pem`
//! on stdout. Direct terminal stdout is refused; pipe or redirect the result.
//!
//! # Usage
//!
//! ```sh
//! torrust-index-auth-keypair | jq -r .private_key_pem > private.pem
//! torrust-index-auth-keypair | jq -r .public_key_pem  > public.pem
//! ```

use std::process::ExitCode;

use clap::Parser;
use torrust_index_auth_keypair::{KeypairOutput, generate_keypair};
use torrust_index_cli_common::{BaseArgs, install_json_panic_hook, parse_args_or_exit, run_stdout_json_command};
use tracing::info;

const COMMAND_NAME: &str = "torrust-index-auth-keypair";

#[derive(Parser)]
#[command(
    name = "torrust-index-auth-keypair",
    version,
    about = "Generate an RSA-2048 key pair for Torrust Index JWT authentication"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,
}

fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);

    let args = parse_args_or_exit::<Args>();

    run_stdout_json_command::<KeypairOutput, String, _>(COMMAND_NAME, args.base.debug, tracing::Level::INFO, || {
        info!("generating RSA-2048 key pair");
        generate_keypair()
    })
}

#[cfg(test)]
mod tests {
    //! # Auth-keypair binary CLI contract tests
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
        assert!(text.contains("Generate an RSA-2048 key pair"));
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
