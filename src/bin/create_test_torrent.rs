//! Command line tool to create a test torrent file.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use thiserror::Error;
use torrust_index::models::torrent_file::{Torrent, TorrentFile, TorrentInfoDictionary};
use torrust_index::services::hasher::sha1; // DevSkim: ignore DS126858
use torrust_index::utils::parse_torrent;
use torrust_index_cli_common::{BaseArgs, install_json_panic_hook, parse_args_or_exit, run_no_stdout_command};
use tracing::info;
use uuid::Uuid;

const COMMAND_NAME: &str = "create_test_torrent";

#[derive(Parser)]
#[command(name = "create_test_torrent", version, about = "Create a test torrent file")]
struct Args {
    /// Directory where the generated torrent file will be written.
    destination_folder: PathBuf,

    #[command(flatten)]
    base: BaseArgs,
}

#[derive(Debug, Error)]
enum CommandError {
    #[error("generated file content is too large to fit in a torrent length field")]
    ContentTooLarge,

    #[error("failed to encode torrent: {source}")]
    EncodeTorrent { source: serde_bencode::Error },

    #[error("failed to create torrent file: {source}")]
    CreateFile { source: std::io::Error },

    #[error("failed to write torrent file: {source}")]
    WriteFile { source: std::io::Error },
}

fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);

    let args = parse_args_or_exit::<Args>();

    run_no_stdout_command::<CommandError, _>(COMMAND_NAME, args.base.debug, tracing::Level::INFO, || {
        create_test_torrent(&args.destination_folder)
    })
}

fn create_test_torrent(destination_folder: &Path) -> Result<(), CommandError> {
    let id = Uuid::new_v4();

    // Content of the file from which the torrent will be generated.
    // We use the UUID as the content of the file.
    let file_contents = format!("{id}\n");
    let file_name = format!("file-{id}.txt");
    let file_length = i64::try_from(file_contents.len()).map_err(|_error| CommandError::ContentTooLarge)?;

    let torrent = Torrent {
        info: TorrentInfoDictionary::with(
            &file_name,
            16384,
            None,
            0,
            &sha1(&file_contents), // DevSkim: ignore DS126858
            &[TorrentFile {
                path: vec![file_name.clone()], // Adjusted to include the actual file name
                length: file_length,
                md5sum: None, // DevSkim: ignore DS126858
            }],
        ),
        announce: Some("https://tracker.torrust-demo.com/announce".to_string()),
        nodes: Some(vec![("99.236.6.144".to_string(), 6881), ("91.109.195.156".to_string(), 1996)]),
        encoding: None,
        httpseeds: Some(vec!["https://seeder.torrust-demo.com/seed".to_string()]),
        announce_list: Some(vec![vec!["https://tracker.torrust-demo.com/announce".to_string()]]),
        creation_date: None,
        comment: None,
        created_by: None,
    };

    let bytes = parse_torrent::encode_torrent(&torrent).map_err(|source| CommandError::EncodeTorrent { source })?;
    let torrent_file_path = destination_folder.join(format!("{file_name}.torrent"));
    let mut output_file = File::create(&torrent_file_path).map_err(|source| CommandError::CreateFile { source })?;
    output_file
        .write_all(&bytes)
        .map_err(|source| CommandError::WriteFile { source })?;

    info!(path = %torrent_file_path.display(), "created test torrent file");

    Ok(())
}

#[cfg(test)]
mod tests {
    //! # Create-test-torrent binary CLI contract tests
    //!
    //! | Test                                  | What it covers                         |
    //! |---------------------------------------|----------------------------------------|
    //! | `help_is_json_control_record`         | `--help` is wrapped as JSON metadata   |
    //! | `version_is_json_control_record`      | `--version` is wrapped as JSON metadata|
    //! | `usage_error_is_json_control_record`  | argv errors become JSON usage records  |
    //! | `command_writes_valid_torrent_file`   | side effect succeeds without stdout data|
    //! | `missing_directory_yields_error`      | file creation errors are propagated    |

    use tempfile::TempDir;
    use torrust_index::utils::parse_torrent::decode_and_validate_torrent_file;
    use torrust_index_cli_common::{CommandExit, ControlPlaneFields, ControlPlaneRecordKind, parse_args_from};

    use super::{Args, COMMAND_NAME, create_test_torrent};

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
        assert!(text.contains("Create a test torrent file"));
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
    fn command_writes_valid_torrent_file() {
        let directory = TempDir::new().expect("temporary directory must be created");

        create_test_torrent(directory.path()).expect("torrent creation must succeed");

        let mut entries = std::fs::read_dir(directory.path()).expect("temporary directory must be readable");
        let entry = entries
            .next()
            .expect("one torrent file must be written")
            .expect("directory entry must be readable");
        assert!(entries.next().is_none(), "only one torrent file should be written");
        assert_eq!(entry.path().extension().and_then(std::ffi::OsStr::to_str), Some("torrent"));

        let bytes = std::fs::read(entry.path()).expect("torrent file must be readable");
        let (_torrent, original_info_hash) = decode_and_validate_torrent_file(&bytes).expect("written torrent must decode");
        assert_eq!(original_info_hash.to_hex_string().len(), 40);
    }

    #[test]
    fn missing_directory_yields_error() {
        let directory = TempDir::new().expect("temporary directory must be created");
        let missing_directory = directory.path().join("missing");

        let error = create_test_torrent(&missing_directory).expect_err("missing output directory must fail");

        assert!(error.to_string().contains("failed to create torrent file"));
    }
}
