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
use torrust_index_auth_keypair::generate_keypair;
use torrust_index_cli_common::{BaseArgs, emit, init_json_tracing, refuse_if_stdout_is_tty};
use tracing::{error, info};

#[derive(Parser)]
#[command(
    name = "torrust-index-auth-keypair",
    about = "Generate an RSA-2048 key pair for Torrust Index JWT authentication"
)]
struct Args {
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
    refuse_if_stdout_is_tty("torrust-index-auth-keypair");

    info!("Generating RSA-2048 key pair...");

    match generate_keypair() {
        Ok(out) => {
            info!("Key pair generated successfully.");
            match emit(&out) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    // Writing to stdout failed (e.g. broken pipe). Honour
                    // the helper's exit-code contract instead of panicking.
                    error!(error = %e, "failed to write key pair to stdout");
                    ExitCode::FAILURE
                }
            }
        }
        Err(e) => {
            error!(error = %e, "keypair generation failed");
            ExitCode::FAILURE
        }
    }
}
