# ADR-T-004: Remove `located-error` Package, Use `tracing` for Error Context

**Status:** Decided
**Date:** 2026-03-23

## Context

The workspace ships a first-party package, `torrust-index-located-error`,
whose sole purpose is to decorate errors with `std::panic::Location`
(file, line, column) so that error origins can be traced back to source
code. It does this through a `LocatedError<E>` wrapper that captures
`#[track_caller]` locations and formats them alongside the original
error message.

The package has a small surface — two public types (`Located`,
`LocatedError`) and a pair of `Into` impls — but it introduces several
costs:

1. **Maintenance overhead.** A dedicated workspace member with its own
   `Cargo.toml`, `README.md`, and test suite must be kept in sync with
   the rest of the crate.
2. **Leaky abstraction.** `LocatedError` wraps every source error in an
   `Arc`, which is unnecessary for errors that are never shared across
   threads. Callers must deal with a lifetime parameter (`'a`) and the
   `Arc` indirection.
3. **Incomplete picture.** `Location` captures a single call-site, not
   a causal chain. In async code the real origin may be several
   `.await` points away, making the single location misleading.
4. **Redundant with `tracing`.** The package already depends on
   `tracing` and emits a `tracing::debug!` event inside the `Into`
   impl. Modern Rust convention is to use structured `tracing` spans
   and events for exactly this kind of contextual metadata — the
   subscriber (e.g. `tracing-subscriber`) can be configured to capture
   file/line automatically via `with_file(true)` and
   `with_line_number(true)`, with richer context (span trees, timing,
   fields) than a bare `Location`.

The package is consumed in only two modules:

- `src/config/mod.rs` — three error variants wrap a `LocatedError`.
- `src/web/api/server/mod.rs` — one error variant wraps a
  `LocatedError`.

## Options Considered

| Option | Approach | Pros | Cons |
|---|---|---|---|
| A. Keep `located-error` | No change | Zero effort | Maintenance cost continues; incomplete context vs. tracing |
| B. Replace with `#[track_caller]` in-line | Remove package, use `Location::caller()` directly at each call-site | No extra dependency | Still a single location; manual plumbing everywhere |
| C. Replace with `tracing` spans/events | Remove package, log errors with `tracing::error!` or `tracing::warn!` at the point of creation; rely on subscriber for file/line | Full causal chain via span tree; zero custom error wrappers; idiomatic Rust | Requires subscriber configuration; slightly different log output |
| D. Replace with `error-stack` or `eyre` | Use a community error-context crate | Rich backtraces and attachments | Adds a new dependency; heavier than needed for this use case |

## Decision

**Option C — Remove `torrust-index-located-error` and rely on `tracing`
for error-origin context.**

### Rationale

- The Rust ecosystem has converged on `tracing` as the standard
  approach for structured diagnostics. Capturing file and line via the
  subscriber is zero-cost when disabled and richer than a manual
  `Location` when enabled.
- Removing the wrapper simplifies the error types in `config` and
  `web::api::server` — they can use plain `Arc<dyn Error + Send + Sync>`
  or concrete error types, eliminating the lifetime parameter.
- The `tracing::debug!` call already present inside `LocatedError`'s
  `Into` impl proves the intent was always to log; the wrapper is
  just an intermediate step that can now be dropped.

### Migration Path

1. In each call-site that currently converts into a `LocatedError`,
   emit a `tracing::error!` (or `tracing::warn!`) event with the
   source error as a field.
2. Replace the `LocatedError<'static, dyn Error + Send + Sync>` fields
   in error enums with `Arc<dyn Error + Send + Sync>` (or a concrete
   type where possible).
3. Remove the `torrust-index-located-error` dependency from the root
   `Cargo.toml` workspace members and the `[dependencies]` table.
4. Delete the `packages/located-error/` directory.
5. Ensure the `tracing-subscriber` configuration includes file/line
   metadata (already the case in the existing subscriber setup, or
   trivially enabled).

## Consequences

- **Simpler error types.** No lifetime parameter, no `Arc` wrapping
  every error.
- **Better diagnostics.** `tracing` spans provide a full causal tree,
  not just a single source location.
- **Fewer workspace members.** One less package to maintain, version,
  and test.
- **Log format change.** Error locations will appear in tracing output
  rather than in the `Display` impl of the error itself. Consumers
  that parsed `LocatedError`'s string format will need to adapt.
