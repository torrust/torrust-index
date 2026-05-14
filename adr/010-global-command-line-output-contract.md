# ADR-T-010: Global Command-Line Output Contract

**Status:** Decided and implemented
**Date decided:** 2026-05-13
**Date implemented:** 2026-05-13
**Supersedes:** The output-stream rules from ADR-T-009 P8/P9.
**Relates to:** [ADR-T-009](009-container-infrastructure-refactor.md) (helper-binary extraction and dependency rules)

---

## Context

ADR-T-009 introduced a strict stdout/stderr contract for container helper
binaries: JSON results on stdout, JSON diagnostics on stderr, and no stdout
result data directly to a terminal. That decision was made inside the
container-infrastructure refactor because the entry script needed reliable JSON
from small Rust helpers.

The contract is not container-specific. The application has several first-party
command-line entrypoints: the server binary, maintenance commands under
`src/bin/`, container helpers under `packages/index-*/`, and future operator
tools. If each command decides independently what stdout and stderr mean, shell
integration becomes brittle and diagnostics can corrupt data streams.

## Decision

Adopt one repository-wide JSON-only command-line output contract for every
first-party Torrust Index command-line entrypoint that is shipped, documented,
or intended for operators.

---

## Scope

### In scope

Every shipped, documented, or operator-facing first-party command-line
entrypoint:

- `src/main.rs` (`torrust-index` server binary).
- `src/bin/create_test_torrent.rs`.
- `src/bin/import_tracker_statistics.rs`.
- `src/bin/parse_torrent.rs`.
- `src/bin/seeder.rs`.
- `src/bin/upgrade.rs`.
- `packages/index-auth-keypair/src/bin/torrust-index-auth-keypair.rs`.
- `packages/index-config-probe/src/bin/torrust-index-config-probe.rs`.
- `packages/index-health-check/src/bin/torrust-index-health-check.rs`.
- `share/container/entry_script_sh` and `share/container/entry_script_lib_sh`.

Command-reachable library paths that emit operator-facing diagnostics:

- `src/bootstrap/logging.rs`.
- `src/console/commands/seeder/app.rs`.
- `src/console/commands/seeder/logging.rs`.
- `src/console/cronjobs/tracker_statistics_importer.rs`.
- `src/mailer.rs`.
- `src/tracker/statistics_importer.rs`.
- `src/upgrades/from_v1_0_0_to_v2_0_0/`.
- `src/utils/parse_torrent.rs`.
- `src/web/api/server/signals.rs`.

Future entry, migration, maintenance, diagnostic, or operator tools are
governed by this ADR from creation.

### Out of scope

Tests, examples, benches, one-off developer fixtures, and library packages with
no shipped binary entrypoint are outside the normative scope unless they are
documented as application commands:

- `build.rs` Cargo protocol output such as `cargo:rerun-if-changed=...`.
- Tests, benches, examples, and harnesses under `tests/`, `src/tests/`, and
  package test directories.
- Developer-only scripts under `contrib/dev-tools/`.
- Library packages with no shipped binary entrypoint, such as Mudlark and
  `render-text-as-image`.

---

## Contract

### Streams

Stdout and stderr are both machine-readable streams. A command writes JSON
records to them or leaves them empty. Plain human-readable text is not a valid
application output format on either stream.

Stdout is reserved for command result data intended for a caller to consume.
Diagnostics, logs, progress messages, warnings, help, usage, prompts, and
status updates go to stderr as JSON control-plane records.

A command that has no stdout result data leaves stdout empty. A long-running
server process normally has no stdout result data; its diagnostics are logs and
therefore belong on stderr.

### JSON output

When a command emits stdout result data, the default wire format is exactly one
JSON object followed by one trailing newline.

On success (exit 0), a command that has stdout result data emits its JSON object
on stdout and may emit JSON diagnostics on stderr.

On failure (exit not 0), stdout is empty. The exit code is the branch signal for
callers, and JSON diagnostics go to stderr.

Commands that need a different stdout JSON shape, such as streaming output, must
document that exception in the command's own contract and explain why the
single-object JSON contract does not fit. When stdout result data is streaming,
stdout uses NDJSON: one JSON object per line. Non-JSON stdout or stderr is
outside this contract and requires a new ADR.

### TTY refusal

A command that emits stdout result data refuses to run when stdout is attached
to a terminal. It exits before producing stdout and reports the diagnostic as
JSON on stderr.

The refusal is unconditional for commands with stdout result data. It does not
depend on whether the payload is sensitive, and it is not caused by the JSON
encoding. JSON is the only output encoding for both streams; the TTY refusal
exists because stdout result data is intended for another process or file.
Operators who want to inspect output interactively can pipe it to another
program such as `jq`, `less`, or `cat`.

Commands that do not emit stdout result data do not refuse merely because stdout
is attached to a terminal. They leave stdout empty and write any diagnostics to
stderr as JSON.

### Exit codes

The shared baseline exit-code classes are:

| Class   | Code | Meaning |
|---------|------|---------|
| success | 0    | Command succeeded. |
| failure | 1    | Runtime, startup, internal, or command execution failure. |
| usage   | 2    | Command-line usage failure: clap argv errors, TTY refusal, or invalid arguments. |

Exit code 2 is reserved for command-line usage failures, including TTY refusal
and argv parsing errors produced by `clap`.

Command-specific non-usage exit codes may still be documented by the owning
command contract. For example, `torrust-index-config-probe` keeps its existing
configuration and probe failure codes.

### Diagnostics

Operator-facing and script-facing commands use `tracing` for diagnostics. The
diagnostic writer is stderr, configured with JSON output.

Stderr is a JSON control and diagnostic stream. When stderr emits multiple
records over time, it uses NDJSON: one complete JSON object per line. Diagnostic
records are `tracing` events so scripts can consume diagnostics without
scraping text. Non-diagnostic control records, such as help and usage, also
write JSON objects to stderr. Plain-text diagnostic formatting is not an output
mode for first-party application binaries; operators can pipe JSON diagnostics
to a viewer when they want a friendlier presentation.

Command-specific diagnostics do not use `println!` or `eprintln!` for progress,
status, or errors; those would put raw text on stdout or stderr.

### Help and usage output

Help and usage information is command output and follows the same JSON-only
stream contract, but it is not stdout result data.

A help request writes a JSON control-plane record to stderr and exits with
code 0. It does not trigger stdout TTY refusal, because it does not emit stdout
result data.

A usage or argv-parse error writes a JSON diagnostic/control-plane record to
stderr and exits with code 2.

Rust commands use `clap` for argv parsing. Raw `clap` help or error text is
wrapped in the JSON contract by the shared CLI infrastructure.

---

## Shared Control-Plane Record Schema

Shared stderr control-plane records use this top-level JSON shape:

| Field     | Type   | Description |
|-----------|--------|-------------|
| `schema`  | number | Shared control-plane record schema version. Initial value: `1`. |
| `command` | string | Binary or entrypoint name. |
| `kind`    | string | One of `help`, `version`, `usage_error`, `tty_refusal`, `panic`, `status`, or `diagnostic`. |
| `message` | string | Short human-readable message carried inside the JSON record. |
| `fields`  | object | Optional kind-specific object tagged with `type`. |

### Structured field variants

| Kind          | Fields |
|---------------|--------|
| `help`        | `text` |
| `version`     | `version` |
| `usage_error` | `exit_code`, `clap_error_kind` |
| `tty_refusal` | `exit_code`, `stream` |
| `panic`       | `exit_code`, `thread`, `location` |

Panic payloads are not part of the shared record because they may contain
secrets.

---

## Command Output Classification

### Commands with stdout result data

These commands emit one JSON object on stdout on success, refuse when stdout is
attached to a terminal, and leave stdout empty on failure:

| Command                        | Stdout result schema |
|--------------------------------|----------------------|
| `torrust-index-auth-keypair`   | `schema`, `private_key_pem`, `public_key_pem` |
| `torrust-index-config-probe`   | `schema`, `database`, `auth` |
| `torrust-index-health-check`   | `schema`, `target`, `status`, `elapsed_ms` |
| `parse_torrent`                | `schema`, decoded torrent, original v1 info hash, input byte length |

All use the default single-object stdout contract. None use the documented
streaming NDJSON exception.

### Commands with no stdout result data

These commands leave stdout empty and write diagnostics to stderr as JSON:

| Command                        | Description |
|--------------------------------|-------------|
| `torrust-index`                | Long-running server; logs go to stderr. |
| `create_test_torrent`          | Side-effect command; writes a torrent file. |
| `import_tracker_statistics`    | Side-effect maintenance command. |
| `seeder`                       | Side-effect load/seeding command. |
| `upgrade`                      | Side-effect migration command. |
| `share/container/entry_script_sh` | Orchestration entrypoint; stdout from helpers is captured inside command substitutions. |

---

## Redaction Policy

JSON diagnostics make accidental secret exposure easier to automate. The
following redaction rules are applied before JSON stderr becomes the output
path:

- Never log raw database URLs that contain credentials. Log a redacted form
  with password, token, and query-secret components removed.
- Never log JWT secrets, private keys, admin tokens, session secrets, API keys,
  SMTP passwords, or mailer credentials.
- Avoid putting secrets in error `Display` strings. Prefer typed error fields
  that can be redacted before logging.
- Keep raw external utility stderr out of top-level diagnostic messages unless
  it has been reviewed or wrapped as a field that can be redacted.

Shared redaction helpers replace secret-like field names with `[redacted]` and
strip userinfo plus secret-bearing query parameters from database URLs before
they are logged.

---

## Shared Rust CLI Infrastructure

`packages/index-cli-common` (`torrust-index-cli-common`) provides the shared
scaffolding for the global contract so every Rust binary shares the same
implementation:

- **JSON clap handling:** `parse_args_or_exit::<T>()` wraps
  `clap::Parser::try_parse()`. Help and version requests emit JSON control
  records to stderr and exit 0; argv errors emit JSON diagnostic/control
  records to stderr and exit 2; stdout remains empty.
- **JSON stderr control-plane emission:** A direct record helper for
  control-plane output that does not depend on a tracing subscriber. Used for
  clap help, clap parse errors, early startup failures, and panic hooks.
- **JSON panic hook:** `install_json_panic_hook(command_name)` emits one JSON
  diagnostic record to stderr and terminates with exit code 1. It does not
  call Rust's default panic hook. It is safe for non-main-thread panics: emit
  a best-effort JSON diagnostic once, avoid waiting on other application
  threads, and terminate the process without returning to the default panic
  path.
- **JSON tracing on stderr:** Idempotent initialization (`try_init` or an
  equivalent guard) so early startup and later application setup cannot
  double-install a subscriber. Each tracing event is emitted as one complete
  JSON line on stderr using a non-interleaving locked writer per event, even
  when multiple tasks or threads log concurrently.
- **TTY refusal:** For commands with stdout result data. Exit code 2 with a
  JSON stderr diagnostic.
- **Stdout JSON emission:** `emit()` writes exactly one JSON object plus one
  trailing newline to stdout, called only after TTY refusal.
- **Command runners:** Small runner helpers for the two command classes:
  stdout-producing single-object commands and no-stdout side-effect commands.
- **Tracing filter precedence:** A non-empty `RUST_LOG` environment variable
  wins, otherwise `--debug` selects debug-level diagnostics, otherwise the
  command's default level is used.
- **`--debug` flag:** Shared across all commands.
- **`ExitCode` centralization:** Direct `std::process::exit` usage is
  centralized in this shared infrastructure. Binaries return `ExitCode` from
  their own `main` function.

---

## Implementation Summary

### Rust helper binaries (`packages/index-*`)

`torrust-index-auth-keypair`, `torrust-index-config-probe`, and
`torrust-index-health-check` use the shared JSON clap parser, install the
shared JSON panic hook, expose `--version` through clap metadata, keep their
stdout result schemas unchanged, and preserve TTY refusal for stdout result
data. `torrust-index-config-probe` no longer preserves Rust's default
plain-text panic output.

### Central application logging

Central application logging uses the shared JSON stderr tracing setup. The root
package depends on `torrust-index-cli-common`. Root binaries return explicit
`ExitCode` values at their `main` boundaries.

### `parse_torrent` and `create_test_torrent`

`parse_torrent` uses the shared JSON clap parser, JSON panic hook, and JSON
stderr tracing runner. It emits one JSON stdout result object and refuses
terminal stdout. `create_test_torrent` uses the same shared infrastructure and
remains a no-stdout side-effect command with JSON diagnostics on stderr.

### `import_tracker_statistics`, `seeder`, and `upgrade`

All three use the shared JSON clap parser, JSON panic hook, JSON stderr tracing
runner, empty stdout side-effect contract, structured tracing diagnostics, and
propagated command errors. The command-reachable tracker statistics and upgrade
modules no longer emit raw stream output or terminal color formatting.

### Command-reachable shared libraries

Server shutdown notices use structured tracing diagnostics. Mail template
initialization errors are returned to callers for JSON diagnostic reporting
instead of printing or exiting from the mailer library. `text-colorizer` is
removed from root runtime code. Color formatting is removed from
command-reachable modules. Library parsing helpers return errors and let
command callers decide how to report them.

### Container entry script

The shell entrypoint checks for `jq`, emits JSON diagnostics and debug phase
records on stderr, wraps validation failures in shared control-plane records,
captures expected utility stderr where the script controls the utility
invocation, and keeps helper stdout inside command substitutions. `set -x`
under `DEBUG=1` is replaced with explicit JSON debug records at phase
boundaries. Shell JSON diagnostics use `jq -cn --arg ...` for string escaping.
If `jq` is missing, one fixed, minimal JSON diagnostic is emitted to stderr
without interpolating untrusted values. File writes to `/etc/motd` and
`/etc/profile` remain plain text because they are not stream output.

### Documentation and changelog

Operator documentation and changelog entries are updated for the command-output
migrations. `README.md` command examples, `docs/containers.md`,
`upgrades/from_v1_0_0_to_v2_0_0/README.md`, and command module docs are
aligned with the migrated command behavior. `CHANGELOG.md` entries are marked
as breaking when command output changes can affect scripts that consumed
previous plain-text output.

---

## Tests And Guards

### Shared infrastructure tests (`packages/index-cli-common`)

- JSON help records, JSON version records, JSON argv-error records, exit code
  mapping, TTY-refusal records, stdout JSON emission, and the panic hook's JSON
  shape.
- Schema/version fields on control-plane records.
- Redaction of common secret-bearing fields.
- `RUST_LOG`/`--debug` precedence.
- Concurrent JSON logging: records emitted from multiple threads or tasks are
  each captured as one complete JSON object per stderr line.

### Helper binary contract tests

Success stdout shape, empty stdout on failure, JSON stderr diagnostics, clap
help JSON, clap version JSON, clap error JSON, and exit code 2 for usage
failures.

### Root binary contract tests

`parse_torrent` success/failure stdout shape. No-stdout commands keeping stdout
empty while logging JSON stderr.

### Container entry-script tests

`packages/index-entry-script` tests parse and assert JSON stderr records for
validation failures and status branches.

### Workspace Clippy guards

Configured through workspace lint levels in `Cargo.toml` (no separate
`clippy.toml`):

- `clippy::print_stdout` and `clippy::print_stderr` are denied. Local
  `#[allow]` annotations exist for Cargo build-script protocol output,
  developer examples, out-of-scope test diagnostics, and the shared CLI
  infrastructure's controlled stdout emitter.
- `clippy::exit` is denied. A local exception exists for the shared CLI
  infrastructure's `exit_with` helper.

### Binary-boundary regression test

The root `cli_contract` integration test scans all in-scope binaries and fails
if a `main` boundary stops returning `ExitCode` or regresses to `Result`
termination, because Rust's default `Result` termination writes raw
`Error: ...` text to stderr.

### TTY-refusal smoke tests

Stdout-producing commands are tested for TTY refusal using a pseudo-terminal
library or tool such as `rexpect` or `portable-pty`.

### Suggested verification commands

```sh
cargo fmt --all
cargo check --workspace --all-targets --all-features 2>&1 | tee /tmp/adr010-cargo-check.log
cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tee /tmp/adr010-cargo-clippy.log
cargo test --workspace --all-targets --all-features 2>&1 | tee /tmp/adr010-cargo-test.log
cargo test --workspace --all-targets --all-features --release 2>&1 | tee /tmp/adr010-cargo-test-release.log
cargo check --workspace --all-targets --no-default-features 2>&1 | tee /tmp/adr010-cargo-check-no-default-features.log
cargo test --workspace --all-targets --no-default-features 2>&1 | tee /tmp/adr010-cargo-test-no-default-features.log
cargo test --doc --workspace --all-features 2>&1 | tee /tmp/adr010-cargo-test-doc.log
cargo doc --workspace --all-features --no-deps 2>&1 | tee /tmp/adr010-cargo-doc.log
```

---

## Consequences

ADR-T-009 remains the historical record for why the helper binaries were
extracted and why the first implementation exists. This ADR is the canonical
application-wide output contract.

New command-line entrypoints must state whether they emit stdout result data. If
they do, they must either use the default single-object JSON contract or
document a justified JSON exception.

The main server and maintenance commands are governed by the same stdout/stderr
separation as the helper binaries. The difference is only whether they have
stdout result data.

All existing first-party command-line entrypoints now conform to this contract.
Commands that predate this ADR have been migrated; no non-conforming legacy
commands remain as precedent.

The exact exit-code taxonomy for root maintenance commands beyond the baseline
`success`, `failure`, and `usage` classes is left to future command-specific
contracts. Existing helper-specific exit codes remain stable unless a
command-specific contract says otherwise.