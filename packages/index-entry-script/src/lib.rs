//! # `torrust-index-entry-script`
//!
//! Integration-test harness for the container entry script's
//! pure shell helpers in
//! [`share/container/entry_script_lib_sh`](../../../share/container/entry_script_lib_sh).
//!
//! The crate ships **no runtime code** of its own — it exists
//! purely as a home for `tests/` that invoke `sh` as a
//! subprocess against the shell library and assert exit
//! codes / stderr contents. This keeps the tests inside
//! `cargo test --workspace` (so CI runs them automatically)
//! while the helpers themselves remain POSIX `sh`, since they
//! must run inside the distroless busybox runtime where Rust
//! is absent.
//!
//! ## What is covered here
//!
//! Anything in `entry_script_lib_sh` that can be exercised
//! host-side without root and without writing to the
//! container's managed volumes
//! (`/etc/torrust/index/`, `/var/lib/torrust/index/`,
//! `/var/log/torrust/index/`):
//!
//! * `validate_auth_keys` — every branch of ADR-T-009 §7.1's
//!   three invariants (mutual exclusion, pair completeness,
//!   cross-pair source consistency).
//! * `seed_sqlite` — every branch of ADR-T-009 §7.2's five
//!   seeding outcomes that does not require chowning into a
//!   managed volume.
//!
//! ## What is **not** covered here
//!
//! * The "missing-under-volume seeded" outcome of
//!   `seed_sqlite` (writes to `/var/lib/torrust/index/…`).
//! * The end-to-end §7.1 "case-3 export" of
//!   `TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__*_PATH` plus key
//!   materialisation against the real generator.
//!
//! Both belong in the container e2e suite (Phase 8 / 9).

use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

/// Contents of the shell library, embedded at compile time.
///
/// Embedding sidesteps the cargo-nextest archive →
/// `--extract-to` → `--target-dir-remap` workflow used by the
/// container `test` stage: a path computed from
/// `CARGO_MANIFEST_DIR` is baked at compile time and no longer
/// exists at run time, so the library would fail to source.
const LIB_CONTENTS: &str = include_str!("../../../share/container/entry_script_lib_sh");

/// Path to a materialised copy of the shell library.
///
/// The library is written once into the OS tempdir on first
/// use and reused for the lifetime of the test binary. Tests
/// only read it (via `sh -c '. "$lib"; …'`), so a single
/// writer is fine.
#[doc(hidden)]
#[must_use]
pub fn lib_path() -> PathBuf {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        // Use a per-process file name so concurrent test
        // binaries (cargo nextest runs them in parallel) do
        // not race on a shared path: one process's `fs::write`
        // would truncate the file while another sourced it,
        // producing spurious `command not found` failures.
        let path = std::env::temp_dir().join(format!("torrust-index-entry-script-lib.{}.sh", std::process::id()));
        std::fs::write(&path, LIB_CONTENTS).expect("failed to materialise entry_script_lib_sh into tempdir");
        path
    })
    .clone()
}

/// Run a snippet of POSIX shell with `entry_script_lib_sh`
/// already sourced.
///
/// The snippet is passed to `sh -c`. The library is sourced
/// **before** the snippet so callers can invoke any function
/// it defines (`validate_auth_keys`, `seed_sqlite`, …)
/// directly. Stdin is closed so a runaway helper cannot block
/// the test.
///
/// Returns the captured `Output`. The caller asserts on
/// `status` and `stderr` as appropriate.
#[doc(hidden)]
#[must_use]
pub fn run_sh(snippet: &str) -> Output {
    let path = lib_path();
    let lib = path
        .to_str()
        .expect("entry_script_lib_sh tempfile path must be UTF-8 for sh -c");

    // The leading `set -eu` mirrors the discipline the entry
    // script itself runs under (see `share/container/entry_script_sh`,
    // which executes its body under `set -eu`). Running the test
    // snippet under the same flags ensures host-side coverage cannot
    // mask failure modes — e.g. unbound variables or unchecked command
    // failures — that the real entry script would surface. `. "$lib"`
    // sources the helpers; the user-supplied snippet runs after.
    let script = format!("set -eu\n. \"{lib}\"\n{snippet}");

    Command::new("sh")
        .arg("-c")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to spawn sh; is /bin/sh available?")
}

/// Same as [`run_sh`], but forwards each argument to the snippet
/// as a positional parameter (`$1`, `$2`, …).
///
/// This is the natural shape for testing functions that take
/// positional arguments (e.g. `validate_auth_keys`):
///
/// ```ignore
/// run_sh_with_args(
///     "validate_auth_keys \"$@\"",
///     &["false", "false", "none", "false", "false", "none"],
/// );
/// ```
///
/// Arguments are **not** spliced into the script text — they are
/// passed as additional `argv` entries to `sh -c`, after a `"sh"`
/// placeholder for `$0`. `sh` then exposes them to the snippet as
/// `$1`, `$2`, … (and through `"$@"`). Because nothing is
/// interpolated into the script string, no quoting or
/// metacharacter escaping is required: arbitrary byte sequences
/// (single quotes, backslashes, spaces, …) are forwarded verbatim
/// by the kernel's `execve` boundary.
#[doc(hidden)]
#[must_use]
pub fn run_sh_with_args(snippet: &str, args: &[&str]) -> Output {
    let path = lib_path();
    let lib = path
        .to_str()
        .expect("entry_script_lib_sh tempfile path must be UTF-8 for sh -c");

    // Mirror `run_sh`: run under `set -eu` so the args variant
    // shares the same discipline as the entry script itself and
    // cannot mask unbound variables or unchecked failures.
    let script = format!("set -eu\n. \"{lib}\"\n{snippet}");

    let mut cmd = Command::new("sh");
    cmd.arg("-c")
        .arg(script)
        .arg("sh") // $0 placeholder
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for a in args {
        cmd.arg(a);
    }
    cmd.output().expect("failed to spawn sh; is /bin/sh available?")
}
