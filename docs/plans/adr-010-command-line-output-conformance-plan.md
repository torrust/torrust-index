# Command-Line Output Conformance Plan

**Status:** Draft plan
**Date:** 2026-05-13
**Implements:** [ADR-T-010](../../adr/010-global-command-line-output-contract.md)
**Related:** [ADR-T-009](../../adr/009-container-infrastructure-refactor.md)

This is an implementation plan for ADR-T-010, not a separate ADR. Its job is to
turn the decided repository-wide command-line output contract into concrete
code, documentation, and regression tests.

## Current Implementation Status

Stages 1 through 6 have landed for the shared Rust helper path and root command
migrations:

- Stage 1 fixed the shared control-plane record shape, baseline exit classes,
  helper stdout schemas, and redaction helpers.
- Stage 2 expanded `torrust-index-cli-common` with JSON `clap` handling, direct
  JSON stderr control-plane emission, the JSON panic hook, idempotent JSON
  stderr tracing with `RUST_LOG` / `--debug` precedence, a non-interleaving
  stderr writer, and command runners.
- Stage 3 wired `torrust-index-auth-keypair`, `torrust-index-config-probe`, and
  `torrust-index-health-check` to the expanded shared infrastructure. Their
  help, version, argv errors, TTY refusal, and panic diagnostics are now JSON
  control-plane records on stderr.
- Stage 4 switched central application logging to the shared JSON stderr
  tracing setup, added the shared CLI contract crate to the root package, and
  made root binaries return explicit `ExitCode` values at their `main`
  boundaries. At that point, maintenance-command internals still remained
  legacy output gaps pending their per-command migration stages.
- Stage 5 migrated `parse_torrent` and `create_test_torrent` to the shared JSON
  clap parser, JSON panic hook, JSON stderr tracing runners, and focused CLI
  contract tests. `parse_torrent` now emits one JSON stdout result object and
  refuses terminal stdout; `create_test_torrent` remains a no-stdout side-effect
  command.
- Stage 6 migrated `import_tracker_statistics`, `seeder`, and `upgrade` to the
  shared JSON clap parser, JSON panic hook, JSON stderr tracing runner, empty
  stdout side-effect contract, structured tracing diagnostics, and propagated
  command errors. The command-reachable tracker statistics and upgrade modules
  no longer emit raw stream output or terminal color formatting.

The container entry script, remaining shared-library cleanup, and regression
guards remain future rollout stages unless their sections below say otherwise.

## Goal

Bring every shipped, documented, or operator-facing first-party command-line
entrypoint into conformance with ADR-T-010. After this work, command stdout and
stderr are machine-readable streams: stdout is empty unless the command emits
result data, result data is JSON, diagnostics are JSON records on stderr, and
stdout-producing commands refuse to write result data directly to a terminal.

## Scope

In scope:

- `src/main.rs` (`torrust-index` server binary).
- `src/bin/create_test_torrent.rs`.
- `src/bin/import_tracker_statistics.rs`.
- `src/bin/parse_torrent.rs`.
- `src/bin/seeder.rs`.
- `src/bin/upgrade.rs`.
- `packages/index-auth-keypair/src/bin/torrust-index-auth-keypair.rs`.
- `packages/index-config-probe/src/bin/torrust-index-config-probe.rs`.
- `packages/index-health-check/src/bin/torrust-index-health-check.rs`.
- `share/container/entry_script_sh` and `share/container/entry_script_lib_sh`,
  because the container entry script is a shipped first-party command
  entrypoint.

Command-reachable library paths that must also be cleaned up when they emit
operator-facing diagnostics:

- `src/bootstrap/logging.rs`.
- `src/console/commands/seeder/app.rs`.
- `src/console/commands/seeder/logging.rs`.
- `src/console/commands/tracker_statistics_importer/app.rs`.
- `src/console/cronjobs/tracker_statistics_importer.rs`.
- `src/mailer.rs`.
- `src/tracker/statistics_importer.rs`.
- `src/upgrades/from_v1_0_0_to_v2_0_0/`.
- `src/utils/parse_torrent.rs`.
- `src/web/api/server/signals.rs`.

Out of scope unless they become documented operator commands:

- `build.rs` Cargo protocol output such as `cargo:rerun-if-changed=...`.
- Tests, benches, examples, and harnesses under `tests/`, `src/tests/`, and
  package test directories.
- Developer-only scripts under `contrib/dev-tools/`.
- Library packages with no shipped binary entrypoint, such as Mudlark and
  `render-text-as-image`.

## Contract Baseline

All migrated commands must share these baseline behaviours:

- Exit code 0 means success.
- Exit code 1 means a runtime, startup, internal, or command execution failure
  unless a command-specific contract documents a narrower non-usage code.
- Exit code 2 means command-line usage failure, including clap argv errors and
  stdout TTY refusal.
- On failure, stdout is empty.
- Stderr is always JSON or empty. There is no TTY exemption for stderr; JSON
  diagnostics remain JSON even when stderr is attached to a terminal.
- Commands with stdout result data refuse when stdout is attached to a terminal.
  Commands with no stdout result data do not perform stdout TTY refusal.
- The in-scope stdout-producing commands use ADR-T-010's default single-object
  stdout contract. None of them use the documented streaming NDJSON exception
  unless a later command-specific contract explicitly says so.
- Help and version requests are JSON control-plane records on stderr. They do
  not emit stdout result data and do not trigger stdout TTY refusal.
- Stderr records are NDJSON: one complete JSON object per line. Rust and shell
  emitters must avoid partial writes that can interleave bytes from concurrent
  records.
- Shared control-plane records, including help, version, usage errors, TTY
  refusal, and panic diagnostics, include a schema/version field so scripts can
  distinguish future contract revisions.
- Command-specific stdout result schemas should either include their own version
  field or be documented as stable command contracts.

## Stage 1 Contract Decisions

The first implementation stage fixes the shared contract details that later
rollout stages wire into each binary.

Shared stderr control-plane records use this top-level JSON shape:

- `schema`: numeric shared control-plane record schema. The initial value is `1`.
- `command`: binary or entrypoint name.
- `kind`: one of `help`, `version`, `usage_error`, `tty_refusal`, `panic`,
  `status`, or `diagnostic`.
- `message`: short human-readable message carried inside the JSON record.
- `fields`: optional kind-specific object tagged with `type`.

The initial structured field variants are:

- `help`: `text`.
- `version`: `version`.
- `usage_error`: `exit_code` and `clap_error_kind`.
- `tty_refusal`: `exit_code` and `stream`.
- `panic`: `exit_code`, `thread`, and `location`. Panic payloads are not part of
  the shared record because they may contain secrets.

The shared baseline exit-code classes are:

- `success`: process status `0`.
- `failure`: process status `1`.
- `usage`: process status `2`.

Command-specific non-usage exit codes may still be documented by the owning
command contract. For example, `torrust-index-config-probe` keeps its existing
configuration and probe failure codes until a command-specific contract changes
them.

Stdout-producing command result schemas use a numeric top-level `schema` field.
The first-stage helper outputs are:

- `torrust-index-auth-keypair`: `schema`, `private_key_pem`, and
  `public_key_pem`.
- `torrust-index-config-probe`: `schema`, `database`, and `auth`.
- `torrust-index-health-check`: `schema`, `target`, `status`, and `elapsed_ms`.

Shared redaction helpers apply the initial redaction policy for diagnostics:
secret-like field names are replaced with `[redacted]`, and database URLs have
userinfo plus secret-bearing query parameters removed before they are logged.

## Redaction Policy

JSON diagnostics are easier for operators and scripts to consume, but they also
make accidental secret exposure easier to automate. The migration must define
and apply a redaction policy before JSON stderr becomes the default path.

Required redaction rules:

- Never log raw database URLs that contain credentials. Either omit them or log
  a redacted form with password, token, and query-secret components removed.
- Never log JWT secrets, private keys, admin tokens, session secrets, API keys,
  SMTP passwords, or mailer credentials.
- Avoid putting secrets in error `Display` strings. Prefer typed error fields
  that can be redacted before logging.
- Keep raw external utility stderr out of top-level diagnostic messages unless
  it has been reviewed or wrapped as a field that can be redacted.
- Add focused tests or review guards for the most likely secret-bearing fields
  before the rollout switches operator commands to JSON stderr by default.

## Command Output Classification

Commands with stdout result data:

- `torrust-index-auth-keypair`: emits one JSON object containing the generated
  key pair.
- `torrust-index-config-probe`: emits one JSON object containing the resolved
  container-relevant configuration subset.
- `torrust-index-health-check`: emits one JSON object containing the health
  result.
- `parse_torrent`: emits one JSON object containing the result schema version,
  decoded torrent, original v1 info hash, and stable parse metadata such as
  input byte length. Do not include raw filesystem paths in the stable result
  schema unless the command contract also defines explicit path encoding rules.

Commands with no stdout result data:

- `torrust-index`: long-running server; stdout remains empty and logs go to
  stderr as JSON.
- `create_test_torrent`: side-effect command; write the torrent file and emit
  status diagnostics on stderr as JSON. If a future caller needs the generated
  path as data, promote that to stdout result data and add TTY refusal at that
  time. For this migration, it remains a no-stdout command.
- `import_tracker_statistics`: side-effect maintenance command; stdout remains
  empty.
- `seeder`: side-effect load/seeding command; stdout remains empty.
- `upgrade`: side-effect migration command; stdout remains empty.
- `share/container/entry_script_sh`: orchestration entrypoint; stdout remains
  empty except for stdout captured from helper binaries inside command
  substitutions.

## Shared Rust CLI Infrastructure

Update `packages/index-cli-common` so every Rust binary can share the same
contract implementation instead of open-coding it.

Stage 2 status: implemented. The shared crate now owns the control-plane record
writer, `parse_args_or_exit::<T>()`, the JSON panic hook, `RUST_LOG` / `--debug`
tracing precedence, locked stderr JSON tracing, and stdout/no-stdout command
runners. The helper binaries use these entrypoints; root binaries will migrate
in later stages.

Required changes:

- Define the shared JSON control-plane record shape, including a schema/version
  field, command name, record kind, message, and structured fields for usage
  errors, TTY refusal, and panic diagnostics.
- Add a JSON stderr record helper for control-plane output that does not depend
  on a tracing subscriber already being installed. This is needed for clap help,
  clap parse errors, early startup failures, and panic hooks.
- Add a `parse_args_or_exit::<T>()` helper around `clap::Parser::try_parse()`:
  help and version requests emit JSON control records to stderr and exit 0;
  argv errors emit JSON diagnostic/control records to stderr and exit 2;
  stdout remains empty.
- Replace all raw clap help/error paths in binaries with the shared parse
  helper.
- Keep `emit()` as the single-object stdout JSON writer, but ensure all callers
  use it only after TTY refusal.
- Keep TTY refusal exit code 2 for stdout-producing commands, and make the
  refusal diagnostic a JSON stderr record.
- Add an `install_json_panic_hook(command_name)` helper. The hook must not call
  Rust's default panic hook, because the default hook writes plain text to
  stderr. It should emit one JSON diagnostic record to stderr and terminate with
  exit code 1.
- Make the panic hook safe for non-main-thread panics: emit a best-effort JSON
  diagnostic once, avoid waiting on other application threads, and terminate the
  process without returning to the default panic path.
- Make JSON tracing setup write to stderr explicitly and use an idempotent
  initialization path (`try_init` or an equivalent guard) so early startup and
  later application setup cannot double-install a subscriber.
- Ensure each tracing event is emitted as one complete JSON line on stderr, even
  when multiple tasks or threads log concurrently. Use a writer strategy that
  serializes each completed record, such as a locked writer per event or an
  equivalent non-interleaving writer, rather than relying on ad-hoc writes to a
  shared stream.
- Add small runner helpers for the two command classes: stdout-producing
  single-object commands, and no-stdout side-effect commands.
- Centralize direct `std::process::exit` usage in this shared infrastructure
  where practical, so binaries return `ExitCode` from their own `main` function.

## Root Crate Wiring

Update the root package so the application binaries can use the shared CLI
contract crate.

Required changes:

- Add `torrust-index-cli-common` as a root dependency in `Cargo.toml`.
- Remove `text-colorizer` from root runtime code once terminal color output is
  gone. Keep it only if a test-only or non-command path still needs it.
- Remove color formatting from command-reachable modules, including
  `src/console/cronjobs/tracker_statistics_importer.rs`,
  `src/tracker/statistics_importer.rs`, and
  `src/upgrades/from_v1_0_0_to_v2_0_0/upgrader.rs`.
- Ensure root binaries return `std::process::ExitCode` rather than `Result` from
  `main`, so Rust's `Termination` implementation cannot print raw `Error: ...`
  text to stderr.
- Convert startup functions that currently panic or unwrap at the binary
  boundary into `Result`-returning functions whose errors are logged as JSON and
  mapped to process exit codes.

## Application Logging

Update `src/bootstrap/logging.rs` and any command-specific logging modules so
all operator-facing diagnostics use JSON tracing on stderr.

Required changes:

- Replace the default, pretty, and compact command-line logging styles with JSON
  stderr logging for application binaries. If human-formatted test logs are
  still useful, keep them behind test-only helpers outside the operator command
  path.
- Make `src/console/commands/seeder/logging.rs` delegate to the central JSON
  stderr setup or remove the module.
- Initialize logging before any code path can emit diagnostics.
- Preserve the current operator intent of `--debug` and add `RUST_LOG` as the
  more expressive override: when `RUST_LOG` is set and non-empty, use it as the
  filter directive; otherwise, `--debug` raises the command's default filter to
  debug; otherwise, use the command or server configuration default. Document
  this precedence and make all paths produce JSON stderr records.
- Avoid emitting ANSI color escape sequences inside log messages. Prefer
  structured fields such as `torrent_id`, `tracker_url`, `limit`, and
  `elapsed_ms`.
- Apply the redaction policy to tracing fields and error messages before they
  are serialized.

## Main Server Binary

Update `src/main.rs` and the startup path used by `torrust-index`.

Required changes:

- Install the JSON panic hook at process start.
- Initialize JSON stderr diagnostics before configuration loading can fail.
- Add a non-panicking configuration loader, for example
  `try_initialize_configuration()`, and have `main` log loader failures as JSON
  before returning a non-zero exit code.
- Change `app::run` to return `Result<Running, StartupError>` or otherwise
  convert startup failures into JSON diagnostics rather than `expect`/`unwrap`
  panics.
- Replace `assert!` and `expect` at the binary boundary with JSON diagnostics
  and explicit exit codes.
- Replace `println!` in `src/web/api/server/signals.rs` with a tracing event
  that records the received shutdown signal or phase on JSON stderr.
- Replace raw output and process exits in `src/mailer.rs` with structured errors
  or tracing errors that flow to the binary boundary.
- Remove raw output from `src/utils/parse_torrent.rs`; library parsing helpers
  should return errors and let command callers decide how to report them.

## Helper Binaries

Update the helper binaries under `packages/index-*`.

Stage 3 status: implemented for the three container helpers. They use the shared
JSON clap parser, install the shared JSON panic hook, expose `--version` through
clap metadata, keep their stdout result schemas unchanged, and preserve TTY
refusal for stdout result data. `torrust-index-config-probe` no longer preserves
Rust's default plain-text panic output.

Required changes:

- Use the shared JSON clap parser in:
  `packages/index-auth-keypair/src/bin/torrust-index-auth-keypair.rs`,
  `packages/index-config-probe/src/bin/torrust-index-config-probe.rs`, and
  `packages/index-health-check/src/bin/torrust-index-health-check.rs`.
- Install the shared JSON panic hook in each helper.
- Replace `torrust-index-config-probe`'s current panic hook, which preserves the
  default raw panic output, with the shared JSON hook.
- Keep stdout success output as exactly one JSON object plus one trailing
  newline.
- Keep failure stdout empty.
- Keep TTY refusal for all three helpers because they emit stdout result data.

## Root `src/bin` Binaries

Update every root maintenance and diagnostic binary.

Stage 5 status: implemented for `src/bin/parse_torrent.rs` and
`src/bin/create_test_torrent.rs`.

Stage 6 status: implemented for `src/bin/import_tracker_statistics.rs`,
`src/bin/seeder.rs`, `src/bin/upgrade.rs`, and their command-reachable tracker
statistics, seeder, and upgrade modules.

Required changes for `src/bin/parse_torrent.rs`:

- Replace hand-rolled `std::env::args()` parsing with clap plus the shared JSON
  clap wrapper.
- Install JSON tracing and the JSON panic hook.
- Refuse stdout TTY because this command emits stdout result data.
- Remove progress `println!` calls.
- Emit one JSON object on stdout on success, containing the result schema
  version, decoded torrent, original v1 info hash, and input byte length.
- On invalid bencode, invalid torrent data, I/O errors, or serialization errors,
  leave stdout empty, emit JSON diagnostics on stderr, and exit non-zero.

Required changes for `src/bin/create_test_torrent.rs`:

- Replace hand-rolled argv parsing with clap plus the shared JSON clap wrapper.
- Install JSON tracing and the JSON panic hook.
- Keep stdout empty.
- Replace usage `eprintln!`, `panic!`, and any future status output with JSON
  diagnostics on stderr.
- Return explicit exit codes for invalid arguments, encode errors, file creation
  errors, and write errors.
- Log the generated torrent path as a JSON stderr status record if operators
  need confirmation.

Required changes for `src/bin/import_tracker_statistics.rs` and its reachable
modules:

- Add clap parsing with the shared JSON clap wrapper, even though the command
  currently takes no arguments, so `--help` and unknown flags are JSON.
- Install JSON tracing and the JSON panic hook in the binary entrypoint.
- Keep stdout empty.
- Replace `println!`, `eprintln!`, colored strings, `expect`, and raw parse
  failures in these paths with tracing events and `Result` propagation:
  `src/console/commands/tracker_statistics_importer/app.rs`,
  `src/console/cronjobs/tracker_statistics_importer.rs`, and
  `src/tracker/statistics_importer.rs`.
- Convert database connection and import failures into JSON diagnostics and
  explicit exit codes.

Required changes for `src/bin/seeder.rs` and
`src/console/commands/seeder/app.rs`:

- Install JSON tracing and the JSON panic hook in the binary entrypoint.
- Use the shared JSON clap wrapper for help and argv errors.
- Keep stdout empty.
- Replace the remaining `print!` error path with a structured tracing error event.
- Remove terminal color formatting from log messages.
- Replace `expect`, `unwrap`, and `panic!` in the command path with propagated
  errors that the binary logs as JSON.
- Return `ExitCode` from `main` instead of `Result`.

Required changes for `src/bin/upgrade.rs` and the v1-to-v2 upgrade modules:

- Replace hand-rolled argv parsing with clap plus the shared JSON clap wrapper.
- Install JSON tracing and the JSON panic hook in the binary entrypoint.
- Keep stdout empty.
- Replace all `println!` and `eprintln!` calls under
  `src/upgrades/from_v1_0_0_to_v2_0_0/` with tracing events.
- Remove terminal color formatting from diagnostics.
- Convert database open, migration, truncation, transfer, and file-read failures
  into propagated errors that the binary logs as JSON.
- Replace `unwrap`, `expect`, and assertion failures in the command path with
  typed errors where practical. For invariant violations that remain panics, the
  JSON panic hook must still prevent raw stderr output.

## Container Entry Script

Update `share/container/entry_script_sh` and `share/container/entry_script_lib_sh`.

Required changes:

- Add POSIX-shell JSON diagnostic helpers, for example `json_log` and
  `json_error_exit`, that write one JSON object per line to stderr. Because the
  runtime image already ships `jq`, prefer `jq -cn --arg ...` for string escaping
  rather than hand-built JSON.
- Check for `jq` before any helper depends on it. If `jq` is missing or cannot
  run, emit one fixed, minimal JSON diagnostic to stderr without interpolating
  untrusted values, then exit non-zero.
- Include the shared schema/version field in shell control-plane records.
- Replace every `echo ... >&2` diagnostic with the JSON helper.
- Replace informational `echo`/`printf` diagnostics in helper functions with JSON
  stderr records. File writes to `/etc/motd` and `/etc/profile` are not stream
  output and can remain plain text.
- Remove `set -x` under `DEBUG=1`. Replace it with explicit JSON debug records
  at phase boundaries that are useful to operators.
- Add failure handling around external utilities that may emit raw stderr on
  expected operator errors (`jq`, `addgroup`, `adduser`, `install`, `chown`,
  `chmod`, `mkdir`, `rm`, and `su-exec`). Capture their stderr where practical,
  redact it when needed, and re-emit it as JSON fields.
- Add a trap for unexpected shell failures that emits a JSON stderr diagnostic
  with the failing line or phase before exiting non-zero.
- Keep helper stdout captured only in command substitutions. Do not forward
  helper stdout directly to the terminal.
- Where pipeline status matters, split commands or capture statuses explicitly
  instead of depending on non-POSIX shell features.
- Update `packages/index-entry-script` tests so validation failures assert JSON
  stderr records rather than plain text.

## Documentation Updates

Update operator documentation after the behavior changes.

Current documentation status: the shared contract shape, helper stdout result
schemas, expanded Rust CLI infrastructure, helper-binary wiring state,
stage-four server logging / root `ExitCode` boundary state, the stage-five
`parse_torrent` / `create_test_torrent` migration, and the stage-six root
maintenance command migration have been documented. The container entry script
is still a legacy output gap until its rollout stage lands; its documentation
should describe the ADR-T-010 target contract without promising behaviour it
does not yet implement.

Documentation maintenance requirements:

- Keep `README.md` command examples aligned with each command's current
  stdout/stderr class.
- Keep `docs/containers.md` aligned with JSON stderr diagnostics, stdout result
  data, helper TTY refusal, and recommended inspection patterns such as piping
  stdout result data to `jq`.
- Keep `upgrades/from_v1_0_0_to_v2_0_0/README.md` aligned with `upgrade`'s JSON
  stderr diagnostics and empty stdout contract.
- Keep command module docs aligned with the migrated command behavior,
  especially tracker statistics importer and upgrade docs.
- Keep `CHANGELOG.md` entries marked as breaking when command output changes can
  affect scripts that consumed previous plain-text output.

## Tests And Guards

Add focused conformance tests near the command code and one broad guard to catch
future regressions.

Required tests:

- `packages/index-cli-common` tests for JSON help records, JSON version records,
  JSON argv-error records, exit code mapping, TTY-refusal records, stdout JSON
  emission, and the panic hook's JSON shape.
- Shared infrastructure tests for schema/version fields on control-plane
  records, redaction of common secret-bearing fields, `RUST_LOG`/`--debug`
  precedence, and concurrent JSON logging. The concurrency test should emit
  records from multiple threads or tasks and assert that every captured stderr
  line round-trips as one complete JSON object.
- Helper binary contract tests for success stdout shape, empty stdout on
  failure, JSON stderr diagnostics, clap help JSON, clap version JSON, clap error
  JSON, and exit code 2 for usage failures.
- Root binary contract tests for `parse_torrent` success/failure stdout shape,
  and for no-stdout commands keeping stdout empty while logging JSON stderr.
- Container entry-script tests in `packages/index-entry-script` for JSON stderr
  on each validation failure branch.
- Workspace clippy guards for raw stream output in shipped command paths. Prefer
  `clippy::print_stdout`, `clippy::print_stderr`, and `clippy.toml`
  `disallowed-macros` entries for `println!`, `eprintln!`, `print!`, and
  `eprint!`, with explicit allow-list entries or local `#[allow]` annotations
  for Cargo build-script protocol output and tests.
- A regression test for `main() -> Result` in in-scope binaries, because Rust's
  default `Result` termination writes raw text on failure.
- A lint-backed guard for `std::process::exit` outside shared CLI infrastructure
  and shell entry scripts. Prefer `clippy::exit` or a `clippy.toml`
  `disallowed-methods` entry when supported, with explicit allow-list entries
  for the shared CLI infrastructure and shell entry scripts.
- TTY-refusal smoke tests for stdout-producing commands. Use a pseudo-terminal
  library or tool such as `rexpect` or `portable-pty` if in-process Rust tests
  cannot reliably allocate a TTY.

Suggested verification commands:

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

After each command completes, grep the temp log for failures or warnings before
summarizing results, following the repository test-running convention.

## Rollout Order

Current status: steps 1 through 6 have landed. Documentation and changelog
entries for the shared-helper stages, the stage-four root logging /
binary-boundary rollout, the stage-five root binary migration, and the stage-six
root maintenance command migration have been updated. Later operator-visible
migrations still need their own documentation and changelog updates when they
land.

1. Finalize the shared control-plane record shape, command-specific result
    schema details, exit-code mapping, and redaction rules.
2. Extend `torrust-index-cli-common` with JSON clap handling, JSON panic hooks,
   idempotent JSON stderr tracing, redaction helpers, and command runners.
3. Migrate the three helper binaries to the expanded shared infrastructure.
4. Switch central application logging to JSON stderr and make root binaries use
   `ExitCode` boundaries.
5. Migrate `parse_torrent` and `create_test_torrent`.
6. Migrate `import_tracker_statistics`, `seeder`, and `upgrade`, including their
   command-reachable modules.
7. Remove raw output from shared libraries reached by command paths.
8. Migrate the container entry script and its tests.
9. Update documentation and changelog.
10. Add regression guards and run the verification suite.

## Open Decisions

- The exact exit-code taxonomy for root maintenance commands beyond the baseline
  `success`, `failure`, and `usage` classes. Existing helper-specific exit codes
  should remain stable unless a command-specific contract says otherwise.
- How strict the container entry script can be with external utility stderr. Full
  conformance requires expected failures to be captured and re-emitted as JSON;
  unexpected process crashes may still need a pragmatic trap-based fallback.
