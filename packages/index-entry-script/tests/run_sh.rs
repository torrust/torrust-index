//! # `run_sh` — host-side smoke tests
//!
//! [`run_sh`](torrust_index_entry_script::run_sh) is the
//! no-positional-args sibling of `run_sh_with_args`. The other
//! integration files (`seed_sqlite`, `validate_auth_keys`)
//! exclusively use the args variant, so without this file
//! `run_sh` would carry zero coverage even though the helper
//! is part of the crate's public surface (re-exported under
//! `#[doc(hidden)]` for use by future test files).
//!
//! | Test                              | What it covers                                    |
//! |-----------------------------------|---------------------------------------------------|
//! | `inline_snippet_succeeds`         | A trivial snippet runs under `set -eu` (exit 0).  |
//! | `library_is_sourced_before_snippet` | `validate_auth_keys` is callable from the snippet. |
//! | `set_eu_surfaces_unbound_variable` | `set -u` is in effect: bare `$x` aborts.          |
//! | `lib_path_is_stable_within_process` | `lib_path()` returns the same tempfile twice.   |

use torrust_index_entry_script::{lib_path, run_sh};

#[test]
fn inline_snippet_succeeds() {
    let out = run_sh(":");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn library_is_sourced_before_snippet() {
    // If the library wasn't sourced this would exit non-zero
    // with "command not found". The auth-keys helper accepts
    // the all-`none` invocation as a happy path (no config).
    let out = run_sh("validate_auth_keys false false none false false none");
    assert!(
        out.status.success(),
        "expected helpers to be sourced; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn set_eu_surfaces_unbound_variable() {
    // `set -u` is part of the discipline `run_sh` enforces.
    // Reading an undeclared variable must abort the snippet
    // with a non-zero exit.
    let out = run_sh("printf '%s' \"$does_not_exist\"");
    assert!(!out.status.success(), "set -u must reject unbound variables");
}

#[test]
fn lib_path_is_stable_within_process() {
    // The OnceLock guarantees the materialised tempfile path
    // is reused, not re-written on every call. This is what
    // makes the helpers safe to source from concurrent test
    // threads.
    let a = lib_path();
    let b = lib_path();
    assert_eq!(a, b);
    assert!(a.exists(), "materialised library file must exist");
}
