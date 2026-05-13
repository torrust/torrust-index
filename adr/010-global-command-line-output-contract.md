# ADR-T-010: Global Command-Line Output Contract

**Status:** Decided
**Date:** 2026-05-13
**Supersedes:** The output-stream rules from ADR-T-009 P8/P9.
**Relates to:** [ADR-T-009](009-container-infrastructure-refactor.md) (helper-binary extraction and dependency rules)
**Implementation plan:** [Command-Line Output Conformance Plan](../docs/plans/adr-010-command-line-output-conformance-plan.md)

---

## Context

ADR-T-009 introduced a strict stdout/stderr contract for container helper binaries: JSON results on stdout, JSON diagnostics on stderr, and no stdout result data directly to a terminal. That decision was made inside the container-infrastructure refactor because the entry script needed reliable JSON from small Rust helpers.

The contract is not container-specific. The application has several first-party command-line entrypoints: the server binary, maintenance commands under `src/bin/`, container helpers under `packages/index-*/`, and future operator tools. If each command decides independently what stdout and stderr mean, shell integration becomes brittle and diagnostics can corrupt data streams.

## Decision

Adopt one repository-wide JSON-only command-line output contract for every first-party Torrust Index command-line entrypoint that is shipped, documented, or intended for operators.

This includes:

- the main `torrust-index` server binary;
- binaries under `src/bin/`;
- helper binaries under `packages/index-*/`;
- future entry, migration, maintenance, diagnostic, or operator tools.

Tests, examples, benches, and one-off developer fixtures are outside the normative scope unless they are documented as application commands.

### Streams

Stdout and stderr are both machine-readable streams. A command writes JSON records to them or leaves them empty. Plain human-readable text is not a valid application output format on either stream.

Stdout is reserved for command result data intended for a caller to consume. Diagnostics, logs, progress messages, warnings, help, usage, prompts, and status updates go to stderr as JSON control-plane records.

A command that has no stdout result data should leave stdout empty. A long-running server process normally has no stdout result data; its diagnostics are logs and therefore belong on stderr.

### JSON Output

When a command emits stdout result data, the default wire format is exactly one JSON object followed by one trailing newline.

On success (exit 0), a command that has stdout result data emits its JSON object on stdout and may emit JSON diagnostics on stderr.

On failure (exit not 0), stdout is empty. The exit code is the branch signal for callers, and JSON diagnostics go to stderr.

Commands that need a different stdout JSON shape, such as streaming output, must document that exception in the command's own contract and explain why the single-object JSON contract does not fit. When stdout result data is streaming, stdout uses NDJSON: one JSON object per line. Non-JSON stdout or stderr is outside this contract and requires a new ADR.

### TTY Refusal

A command that emits stdout result data refuses to run when stdout is attached to a terminal. It exits before producing stdout and reports the diagnostic as JSON on stderr.

The refusal is unconditional for commands with stdout result data. It does not depend on whether the payload is sensitive, and it is not caused by the JSON encoding. JSON is the only output encoding for both streams; the TTY refusal exists because stdout result data is intended for another process or file. Operators who want to inspect output interactively can pipe it to another program such as `jq`, `less`, or `cat`.

Commands that do not emit stdout result data do not refuse merely because stdout is attached to a terminal. They leave stdout empty and write any diagnostics to stderr as JSON.

Exit code 2 is reserved for command-line usage failures, including TTY refusal and argv parsing errors produced by `clap`.

### Diagnostics

Operator-facing and script-facing commands use `tracing` for diagnostics. The diagnostic writer is stderr, configured with JSON output.

Stderr is a JSON control and diagnostic stream. When stderr emits multiple records over time, it uses NDJSON: one JSON object per line. Diagnostic records should be `tracing` events so scripts can consume diagnostics without scraping text. Non-diagnostic control records, such as help and usage, also write JSON objects to stderr. Plain-text diagnostic formatting is not an output mode for first-party application binaries; operators can pipe JSON diagnostics to a viewer when they want a friendlier presentation.

### Help And Usage Output

Help and usage information is command output and follows the same JSON-only stream contract, but it is not stdout result data.

A help request writes a JSON control-plane record to stderr and exits with code 0. It does not trigger stdout TTY refusal, because it does not emit stdout result data.

A usage or argv-parse error writes a JSON diagnostic/control-plane record to stderr and exits with code 2.

Rust commands may still use `clap` for argv parsing, but raw `clap` help or error text is a legacy gap unless it is wrapped in the JSON contract.

Command-specific diagnostics should not use `println!` or `eprintln!` for progress, status, or errors; those would put raw text on stdout or stderr.

## Implementation Guidance

Use `torrust-index-cli-common` for command-line tools that emit a single JSON object on stdout. It provides the current shared scaffolding for the global contract: TTY refusal for commands with stdout result data, JSON tracing on stderr, JSON emission on stdout, and the common `--debug` flag.

Commands that do not emit stdout result data still follow the stream separation rule: diagnostics and logs go to stderr as JSON, preferably through `tracing`.

Existing commands that predate this ADR and print raw text to stdout or stderr are non-conforming legacy commands, not precedent. They should be migrated as follow-up work. Any functional change, operator-documentation change, or automation reuse of those commands must bring them under this ADR.

## Consequences

ADR-T-009 remains the historical record for why the helper binaries were extracted and why the first implementation exists. This ADR is the canonical application-wide output contract.

New command-line entrypoints must state whether they emit stdout result data. If they do, they must either use the default single-object JSON contract or document a justified JSON exception.

The main server and maintenance commands are governed by the same stdout/stderr separation as the helper binaries. The difference is only whether they have stdout result data.
