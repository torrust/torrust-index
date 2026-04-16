//! Generate an RSA-2048 key pair for JWT authentication.
//!
//! Outputs both PEM blocks (private key first, then public key) to
//! **stdout**. Diagnostic messages go to **stderr** via `tracing`.
//!
//! The tool refuses to run if stdout is a terminal to prevent
//! accidental display of key material on screen.
//!
//! # Usage
//!
//! ```sh
//! tmpfile=$(mktemp /tmp/auth_keys.XXXXXX)
//! chmod 0600 "$tmpfile"
//! torrust-generate-auth-keypair > "$tmpfile"
//! sed -n '/BEGIN PRIVATE KEY/,/END PRIVATE KEY/p' "$tmpfile" > private.pem
//! sed -n '/BEGIN PUBLIC KEY/,/END PUBLIC KEY/p'   "$tmpfile" > public.pem
//! rm -f "$tmpfile"
//! ```
//!
//! See ADR-T-007 Phase 6 for full context.

use std::io::{self, IsTerminal, Write};
use std::process;

use clap::Parser;
use rsa::RsaPrivateKey;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use tracing::{error, info};

#[derive(Parser)]
#[command(
    name = "torrust-generate-auth-keypair",
    about = "Generate an RSA-2048 key pair for Torrust Index JWT authentication"
)]
struct Args {
    /// Enable debug-level logging output on stderr.
    #[arg(long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();

    // Initialise tracing subscriber with structured JSON output on stderr.
    let level = if args.debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    tracing_subscriber::fmt()
        .json()
        .with_max_level(level)
        .with_writer(io::stderr)
        .init();

    // Refuse to run if stdout is a terminal.
    if io::stdout().is_terminal() {
        error!(
            hint = "pipe the output to a file to avoid displaying key material on screen",
            example = concat!(
                "tmpfile=$(mktemp /tmp/auth_keys.XXXXXX) && ",
                "chmod 0600 \"$tmpfile\" && ",
                "torrust-generate-auth-keypair > \"$tmpfile\" && ",
                "sed -n '/BEGIN PRIVATE KEY/,/END PRIVATE KEY/p' \"$tmpfile\" > private.pem && ",
                "sed -n '/BEGIN PUBLIC KEY/,/END PUBLIC KEY/p' \"$tmpfile\" > public.pem && ",
                "rm -f \"$tmpfile\"",
            ),
            "stdout is a terminal"
        );
        process::exit(1);
    }

    info!("Generating RSA-2048 key pair...");

    let mut rng = rsa::rand_core::OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap_or_else(|e| {
        error!(error = %e, "RSA key generation failed");
        process::exit(1);
    });

    let private_pem = private_key.to_pkcs8_pem(LineEnding::LF).unwrap_or_else(|e| {
        error!(error = %e, "private key PEM export failed");
        process::exit(1);
    });

    let public_pem = private_key
        .to_public_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap_or_else(|e| {
            error!(error = %e, "public key PEM export failed");
            process::exit(1);
        });

    info!("Key pair generated successfully.");

    let stdout = io::stdout();
    let mut out = stdout.lock();
    out.write_all(private_pem.as_bytes()).unwrap_or_else(|e| {
        error!(error = %e, "failed to write private key");
        process::exit(1);
    });
    out.write_all(public_pem.as_bytes()).unwrap_or_else(|e| {
        error!(error = %e, "failed to write public key");
        process::exit(1);
    });
    out.flush().unwrap_or_else(|e| {
        error!(error = %e, "failed to flush stdout");
        process::exit(1);
    });
}
