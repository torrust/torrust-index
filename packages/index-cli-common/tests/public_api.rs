//! # `torrust-index-cli-common` — integration tests
//!
//! These tests exercise the crate's **public** surface
//! (`emit`, `BaseArgs`) the same way a downstream helper
//! binary does: through `pub` items only, with no
//! `pub(crate)` peeking. The unit tests under
//! `src/tests/mod.rs` cover the same functions plus a few
//! private branches; this file pins the externally-visible
//! contract that helper binaries rely on.
//!
//! | Test                                  | What it covers                                    |
//! |---------------------------------------|---------------------------------------------------|
//! | `emit_real_stdout_through_public_api` | `emit` is callable through the crate's public API |
//! | `base_args_flattens_into_clap_parser` | `BaseArgs` composes via `#[command(flatten)]`     |
//! | `base_args_long_flag_toggles_debug`   | The `--debug` long flag flips the field           |
//! | `base_args_rejects_unknown_short_flag` | clap rejects an unrelated flag at parse time     |

use clap::Parser;
use serde::Serialize;
use torrust_index_cli_common::{BaseArgs, emit};

/// Minimal helper-binary-shaped CLI: every helper composes
/// `BaseArgs` via `#[command(flatten)]`, so this fixture
/// mirrors the real call sites in
/// `torrust-index-config-probe`, `torrust-index-auth-keypair`,
/// and `torrust-index-health-check`.
#[derive(Parser)]
#[command(name = "fixture-helper")]
struct FixtureCli {
    #[command(flatten)]
    base: BaseArgs,
}

#[test]
fn emit_real_stdout_through_public_api() {
    // Under `cargo test`, stdout is captured by the harness,
    // so this is a happy-path smoke test — the documented
    // contract is "writes one JSON object + trailing newline".
    #[derive(Serialize)]
    struct Payload<'a> {
        schema: u32,
        name: &'a str,
    }

    emit(&Payload { schema: 1, name: "ok" }).expect("emit must succeed under captured stdout");
}

#[test]
fn base_args_flattens_into_clap_parser() {
    let cli = FixtureCli::try_parse_from(["fixture-helper"]).expect("no args is valid");
    assert!(!cli.base.debug, "default must be debug=false");
}

#[test]
fn base_args_long_flag_toggles_debug() {
    let cli = FixtureCli::try_parse_from(["fixture-helper", "--debug"]).expect("--debug is valid");
    assert!(cli.base.debug);
}

#[test]
fn base_args_rejects_unknown_short_flag() {
    // Pins clap's contract that unknown flags surface as a
    // parse error rather than being silently dropped — every
    // helper binary depends on this for the `unknown-flag →
    // exit 2` mapping documented in ADR-T-009 §6.1.
    let Err(err) = FixtureCli::try_parse_from(["fixture-helper", "--no-such-flag"]) else {
        panic!("unknown flag must be rejected by clap");
    };
    assert_eq!(err.exit_code(), 2, "clap argv-parse failure exit code is 2");
}
