//! Command line tool to parse a torrent file and emit the decoded torrent.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use serde::Serialize;
use thiserror::Error;
use torrust_index::models::torrent_file::Torrent;
use torrust_index::utils::parse_torrent::{DecodeTorrentFileError, decode_and_validate_torrent_file};
use torrust_index_cli_common::{BaseArgs, install_json_panic_hook, parse_args_or_exit, run_stdout_json_command};

const COMMAND_NAME: &str = "parse_torrent";
const OUTPUT_SCHEMA: u32 = 1;

#[derive(Parser)]
#[command(
    name = "parse_torrent",
    version,
    about = "Parse a torrent file and emit decoded torrent metadata as JSON"
)]
struct Args {
    /// Path to the torrent file to parse.
    torrent_file: PathBuf,

    #[command(flatten)]
    base: BaseArgs,
}

#[derive(Debug, Serialize)]
struct Output {
    schema: u32,
    torrent: Torrent,
    original_v1_info_hash: String,
    input_byte_length: usize,
}

#[derive(Debug, Error)]
enum CommandError {
    #[error("failed to read torrent file: {source}")]
    ReadInput { source: std::io::Error },

    #[error("failed to decode torrent file: {source}")]
    DecodeInput { source: DecodeTorrentFileError },
}

fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);

    let args = parse_args_or_exit::<Args>();

    run_stdout_json_command::<Output, CommandError, _>(COMMAND_NAME, args.base.debug, tracing::Level::INFO, || {
        parse_file(&args.torrent_file)
    })
}

fn parse_file(path: &Path) -> Result<Output, CommandError> {
    let bytes = std::fs::read(path).map_err(|source| CommandError::ReadInput { source })?;
    let input_byte_length = bytes.len();
    let (torrent, original_info_hash) =
        decode_and_validate_torrent_file(&bytes).map_err(|source| CommandError::DecodeInput { source })?;

    Ok(Output {
        schema: OUTPUT_SCHEMA,
        torrent,
        original_v1_info_hash: original_info_hash.to_hex_string(),
        input_byte_length,
    })
}

#[cfg(test)]
mod tests {
    //! # Parse-torrent binary CLI contract tests
    //!
    //! | Test                                  | What it covers                         |
    //! |---------------------------------------|----------------------------------------|
    //! | `help_is_json_control_record`         | `--help` is wrapped as JSON metadata   |
    //! | `version_is_json_control_record`      | `--version` is wrapped as JSON metadata|
    //! | `usage_error_is_json_control_record`  | argv errors become JSON usage records  |
    //! | `valid_torrent_yields_json_result`    | stdout result schema and metadata      |
    //! | `invalid_torrent_yields_error`        | invalid input fails before stdout data |

    use std::io::Write;

    use serde_json::Value;
    use tempfile::NamedTempFile;
    use torrust_index_cli_common::{CommandExit, ControlPlaneFields, ControlPlaneRecordKind, parse_args_from};

    use super::{Args, COMMAND_NAME, OUTPUT_SCHEMA, parse_file};

    const VALID_TORRENT_BYTES: &[u8] = include_bytes!(
        "../../tests/fixtures/torrents/6c690018c5786dbbb00161f62b0712d69296df97_with_custom_info_dict_key.torrent"
    );

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
        assert!(text.contains("Parse a torrent file"));
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

    #[test]
    fn valid_torrent_yields_json_result() {
        let mut file = NamedTempFile::new().expect("temporary file must be created");
        file.write_all(VALID_TORRENT_BYTES).expect("fixture torrent must be writable");

        let output = parse_file(file.path()).expect("fixture torrent must parse");
        let json = serde_json::to_string(&output).expect("parse output must serialize");
        let value: Value = serde_json::from_str(&json).expect("parse output must be valid JSON");

        assert_eq!(value["schema"], OUTPUT_SCHEMA);
        assert_eq!(value["original_v1_info_hash"], "6c690018c5786dbbb00161f62b0712d69296df97");
        assert!(value["input_byte_length"].as_u64().is_some_and(|length| length > 0));
        assert!(value["torrent"].is_object());
        assert!(value.get("path").is_none(), "stdout schema must not expose the input path");
    }

    #[test]
    fn invalid_torrent_yields_error() {
        let mut file = NamedTempFile::new().expect("temporary file must be created");
        file.write_all(b"not bencoded torrent data")
            .expect("temporary file must be writable");

        let error = parse_file(file.path()).expect_err("invalid torrent data must fail");

        assert!(error.to_string().contains("failed to decode torrent file"));
    }
}
