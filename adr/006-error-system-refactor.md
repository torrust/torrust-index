# ADR-T-006: Refactor the Error System

**Status:** Implemented (Phases 0–4 complete)
**Date:** 2026-04-15
**Relates to:** [ADR-T-004](004-remove-located-error.md)
(removal of `located-error` package — prerequisite cleanup)

## Context

The index's error system has grown organically over time and now
exhibits several structural problems that hinder maintainability,
debuggability, and API ergonomics. This ADR catalogues the issues
and presents options for a comprehensive refactor.

### Current Architecture

The error system is spread across roughly 15 domain-specific error
types plus several types that do not implement the `Error` trait at
all.

| Layer | Type | Crate | Trait |
|---|---|---|---|
| Service (central hub) | `ServiceError` (41 variants) | `derive_more` | `Error + Display` |
| Database | `database::Error` (12 variants) | manual | `Error + Display` |
| Tracker API | `TrackerAPIError` (10 variants) | `derive_more` | `Error + Display` |
| Torrent parsing | `DecodeTorrentFileError` (4 variants) | `derive_more` | `Error + Display` |
| Torrent metadata | `MetadataError` (2 variants) | `derive_more` | `Error + Display` |
| Configuration | `config::Error` (6 variants) | `thiserror` | `Error + Display` |
| Config validation | `ValidationError` (1 variant) | `thiserror` | `Error + Display` |
| Web server | `web::api::server::Error` (2 variants) | `thiserror` | `Error + Display` |
| HTTP handler | `torrent::handlers::Request` (8 variants) | `derive_more` | `Error + Display` |
| Cache (3 types) | `cache::{Error, cache::Error, image::manager::Error}` | none | **no `Error` impl** |
| Client | `web::api::client::Error` (1 variant) | manual | `From<reqwest::Error>` only |
| Model/parse | `UsernameParseError`, `UsersSortingParseError`, `UsersFiltersParseError` | manual | `Error + Display` |
| CLI | `seeder::Error`, `ImportError` | mixed | `Error + Display` |

Error flow: low-level errors (sqlx, argon2, io, serde_json,
domain) are converted into `ServiceError` via `From` impls in
`src/errors.rs`. `ServiceError` is then converted into an HTTP
response via `IntoResponse` in `src/web/api/server/v1/responses.rs`.

### Identified Problems

1. **`ServiceError` is a 41-variant god enum.** It conflates
   authentication, validation, database, tracker, torrent, and
   infrastructure failures into a single type. Every service
   function that returns `ServiceResult<T>` can, in theory, return
   any of the 41 variants — callers cannot narrow the set at the
   type level.

2. **Lossy error conversion.** Nine `From` impls convert diverse
   source errors into `ServiceError::InternalServerError`, erasing
   the original cause. For example, `argon2::Error`, `io::Error`,
   `Box<dyn Error>`, and `serde_json::Error` all collapse to the
   same variant. The original error is only dumped to `eprintln!`
   before being discarded.

3. **`eprintln!()` logging in `From` impls.** Every `From`
   implementation logs the source error via `eprintln!` instead of
   using `tracing`. This bypasses structured logging, span context,
   and subscriber configuration.

4. **Inconsistent error derivation.** Three different strategies
   coexist: `derive_more` (majority), `thiserror` (config/server),
   and hand-written `impl Display + Error` (database, model
   parsers). There is no convention for which to use where.

5. **Cache errors lack `Error` trait.** The three cache error enums
   (`cache::Error`, `cache::cache::Error`,
   `cache::image::manager::Error`) derive only `Debug`, making
   them incompatible with `?` propagation and error combinators.

6. **Unused `anyhow` dependency.** `anyhow = "1"` is declared in
   `Cargo.toml` but never imported or used anywhere in the crate.

7. **String-typed error data.** Several variants carry opaque
   `String` payloads instead of structured source errors:
   `TrackerAPIError::TrackerOffline { error: String }`,
   `FailedToParseTrackerResponse { body: String }`,
   `database::Error::ErrorWithText(String)`,
   `UsernameParseError(String)`.

8. **Duplicated semantics across layers.** `database::Error` and
   `ServiceError` each define their own `UserNotFound`,
   `TorrentNotFound`, `TagAlreadyExists`, etc. The mapping between
   them is a manual const function in `src/errors.rs`.

9. **HTTP status codes are stringly-typed.** Error-to-status
   mapping lives in a large `match` block that must be updated in
   lock-step with every new `ServiceError` variant.

10. **No `#[source]` / `Error::source()` chain.** Because `From`
    impls discard the original error, `ServiceError` never carries
    a `source()` reference. Backtraces and error chains are lost.

11. **Some `From` impls contain business logic.** `From<sqlx::Error>`
    inspects raw SQLite error code `2067` and string-matches on
    `"torrust_torrents.info_hash"` to decide the variant. This
    couples error conversion to database schema details.

## Options Considered

### Option A: Incremental Cleanup (Minimal Scope)

Keep the existing architecture but address the worst pain points
without restructuring error types.

**Changes:**
- Replace all `eprintln!()` in `From` impls with `tracing::error!()`.
- Standardise on a single derive crate (`derive_more` or
  `thiserror`) across all error types.
- Add `#[derive(Debug, Display, Error)]` to the cache error types.
- Remove `ErrorWithText(String)` and `Error` from
  `database::Error`; replace with specific variants.
- Remove unused `anyhow` dependency.
- Preserve `ServiceError` as the central enum.

**Pros:**
- Smallest diff; lowest risk.
- No API-response changes; no migration needed.
- Can be done incrementally, one file at a time.

**Cons:**
- The 41-variant god enum remains.
- Error source chains are still lost.
- Does not fix the structural coupling between layers.

---

### Option B: Domain-Scoped Error Enums with a Thin HTTP Mapping Layer

Split `ServiceError` into separate enums per domain (auth, torrent,
user, category/tag, tracker, infrastructure) and introduce a thin
`ApiError` wrapper that implements `IntoResponse`.

**Changes:**
- Define per-domain error enums: `AuthError`, `TorrentError`,
  `UserError`, `CategoryTagError`, `TrackerError`, etc.
- Each domain error carries `#[source]` to preserve the cause chain.
- Service functions return domain-specific `Result<T, DomainError>`.
- A top-level `ApiError` enum wraps the domain errors and
  implements `IntoResponse` with status-code mapping.
- HTTP handlers convert domain errors into `ApiError` at the
  handler boundary (via `From` or `.map_err()`).
- `database::Error` is refined with specific variants and
  implements `Error::source()`.
- Status codes are derived from a trait method
  (`fn status_code(&self) -> StatusCode`) on each domain error,
  removing the monolithic match block.
- All errors use a single derive crate.

**Pros:**
- Service functions advertise exactly which errors they can return.
- Error chains are preserved end-to-end.
- Smaller, focused enums are easier to review and maintain.
- Adding a new domain error does not touch unrelated modules.
- Status-code mapping is co-located with the error definition.

**Cons:**
- Large refactor touching most service and handler files.
- Risk of over-segmentation — some shared variants (e.g.
  `InternalServerError`) would be duplicated or require a shared
  "common" error type.
- Handlers that call multiple services need to unify heterogeneous
  error types.

---

### Option C: Adopt `error-stack` for Context-Rich Error Reporting

Replace the hand-rolled conversion system with the
[`error-stack`](https://docs.rs/error-stack) crate, which provides
typed context attachments and full report trees.

**Changes:**
- Service functions return `error_stack::Result<T, DomainError>`.
- Low-level errors are attached as context rather than converted:
  `report.attach_printable("while uploading torrent")`.
- `IntoResponse` is implemented for `error_stack::Report<E>` via a
  wrapper, extracting the top-level error for the response and
  logging the full report via `tracing`.
- Status codes are derived from a trait on the top-level error.
- Domain error enums are kept small (per Option B) or even reduced
  to simple marker types where the real detail lives in attachments.

**Pros:**
- Full context tree: every intermediate step can attach metadata
  without losing the source.
- Backtraces and span traces are captured automatically.
- Report debug output is extremely detailed — excellent for logs.
- Encourages "attach context, don't convert" philosophy, which
  avoids lossy `From` impls entirely.

**Cons:**
- Adds a new dependency (`error-stack` + its transitive deps).
- Learning curve for contributors unfamiliar with the crate.
- `error-stack::Report` is not `PartialEq`, breaking existing test
  assertions that compare `ServiceError` variants directly.
- Report formatting may leak internal details unless carefully
  filtered before serialising HTTP responses.

---

### Option D: Adopt `anyhow`/`eyre` for Internal Errors, Typed Errors at Boundaries

Use `anyhow::Error` (or `eyre::Report`) as the internal error type
for service logic, and convert to typed errors only at the HTTP
boundary.

**Changes:**
- Service functions return `anyhow::Result<T>`.
- Internal conversions use `.context("…")` for human-readable
  chaining.
- At the handler level, `anyhow::Error` is inspected (via
  `downcast_ref`) or mapped to an `ApiError` enum that implements
  `IntoResponse`.
- `anyhow` is already in `Cargo.toml` (currently unused), so no
  new dependency is introduced.

**Pros:**
- Minimal boilerplate in service code: any `?` just works.
- Rich context strings for debugging.
- Uses the already-declared `anyhow` dependency.
- Very fast to implement — most `From` impls can be deleted.

**Cons:**
- Service-layer errors become opaque: callers cannot pattern-match
  without `downcast`, losing compile-time exhaustiveness checks.
- `downcast_ref` at the HTTP boundary is fragile and easy to get
  wrong.
- `anyhow::Error` is not `PartialEq` or `Eq`, breaking test
  assertions.
- Encourages stringly-typed context; domain errors become second
  class.

---

### Option E: `thiserror` Everywhere + `#[source]` Chains

Standardise the entire codebase on `thiserror` with mandatory
`#[source]` on every variant that wraps a lower-level error, and
tighten up `ServiceError` without splitting it.

**Changes:**
- Migrate all `derive_more`-based and hand-written error types to
  `thiserror`.
- Add `#[source]` (or `#[from]`) attributes to variants that
  currently discard their cause.
- Replace `ServiceError::InternalServerError` with specific
  variants (e.g. `PasswordHashingFailed(#[source] argon2::Error)`,
  `IoError(#[source] std::io::Error)`, etc.) so causes are
  preserved.
- Replace `eprintln!()` logging with `tracing::error!()` at the
  `IntoResponse` boundary (single logging site).
- Move the SQL-error-code inspection out of `From<sqlx::Error>` and
  into a named function in the database layer.
- Implement `Error` for cache error types.
- Remove `anyhow` dependency.

**Pros:**
- Single, well-known derive macro (`thiserror` is the Rust
  ecosystem standard for library errors).
- Full `Error::source()` chain without a new framework.
- `ServiceError` keeps its role as the central enum — smaller diff
  than Option B.
- `PartialEq` / `Eq` can still be derived on variants that don't
  carry non-Eq sources (or can be dropped from tests that don't
  need it).

**Cons:**
- `ServiceError` remains large (though more informative).
- Functions still return a broad error type; no compile-time
  narrowing.
- `thiserror` and `derive_more` serve overlapping purposes;
  removing `derive_more::Error` may conflict with other
  `derive_more` uses (Display, etc.).

## Decision

**Option B: Domain-Scoped Error Enums with a Thin HTTP Mapping Layer.**

### Rationale

The 41-variant `ServiceError` god enum is the root of the most
painful problems listed above: every service function can
theoretically return any of the 41 variants, nine `From` impls
erase the original cause into `InternalServerError`, and the
monolithic status-code match block must be updated in lock-step with
every new variant. Option B directly addresses all of these by
splitting error types along domain boundaries and co-locating
status-code mapping with the error definition.

**Why not the other options:**

- **Option A (Incremental Cleanup)** fixes symptoms (logging,
  derive consistency, cache traits) but leaves the structural
  coupling intact. The god enum remains, callers still cannot narrow
  error sets, and source chains are still lost. It is a necessary
  subset of any option — but not sufficient on its own.

- **Option C (`error-stack`)** provides excellent diagnostics but
  introduces a framework-level dependency. `error-stack::Report` is
  not `PartialEq`, its debug output may leak internal details into
  HTTP responses if not carefully filtered, and the learning curve
  for contributors is non-trivial. The benefits (context trees,
  automatic backtraces) can largely be achieved with `#[source]`
  chains and `tracing` spans without the extra dependency.

- **Option D (`anyhow`/`eyre` internally)** trades compile-time
  exhaustiveness for convenience. Service-layer errors become
  opaque, `downcast_ref` at the HTTP boundary is fragile, and the
  approach actively discourages domain-specific error types — the
  opposite direction from what the codebase needs.

- **Option E (`thiserror` everywhere, keep `ServiceError`)** is
  close to Option B but retains the large central enum. Functions
  still return a broad error type with no compile-time narrowing.
  The smaller diff is attractive, but it preserves the structural
  problem rather than solving it.

**Key advantages of Option B for this codebase:**

1. **Compile-time narrowing.** A function returning
   `Result<T, AuthError>` cannot produce a `TorrentNotFound`. The
   type system communicates which failures are possible.

2. **Source chains.** Option B supports carrying `#[source]`
   on domain errors so the full cause chain can be preserved for
   logging. The current implementation is still partly lossy:
   some `From` impls erase `argon2::Error`, `io::Error`, etc.
   into higher-level variants such as `InternalServerError`
   instead of retaining them as sources.

3. **Co-located status codes.** Each domain error implements a
   `fn status_code(&self) -> StatusCode` method (or the mapping
   lives in the `From<DomainError> for ApiError` impl), removing
   the monolithic 41-arm match in `http_status_code_for_service_error`.

4. **Incremental migration.** `ServiceError` can shrink one domain
   at a time. For example, extract `AuthError` first, update
   auth-related services and handlers, then move on to
   `TorrentError`. Each step is self-contained and reviewable.

5. **No new framework dependency.** Only `thiserror` (or
   `derive_more`) is needed — both are already in the dependency
   tree.

### Anticipated Domain Error Enums

A rough grouping based on the current `ServiceError` variants:

| Domain Error         | Current `ServiceError` variants covered |
|----------------------|----------------------------------------|
| `AuthError`          | `WrongPasswordOrUsername`, `InvalidPassword`, `UsernameNotFound`, `TokenNotFound`, `TokenExpired`, `TokenInvalid`, `UnauthorizedAction`, `UnauthorizedActionForGuests`, `LoggedInUserNotFound` |
| `UserError`          | `ClosedForRegistration`, `EmailMissing`, `EmailInvalid`, `EmailTaken`, `EmailNotVerified`, `FailedToSendVerificationEmail`, `UserNotFound`, `AccountNotFound`, `UsernameTaken`, `UsernameInvalid`, `ProfanityError`, `BlacklistError`, `UsernameCaseMappedError`, `PasswordTooShort`, `PasswordTooLong`, `PasswordsDontMatch`, `InvalidUserListing` |
| `TorrentError`       | `InvalidTorrentFile`, `InvalidTorrentPiecesLength`, `InvalidFileType`, `InvalidTorrentTitleLength`, `MissingMandatoryMetadataFields`, `InfoHashAlreadyExists`, `CanonicalInfoHashAlreadyExists`, `OriginalInfoHashAlreadyExists`, `TorrentTitleAlreadyExists`, `TorrentNotFound`, `WhitelistingError` |
| `CategoryTagError`   | `InvalidCategory`, `InvalidTag`, `CategoryAlreadyExists`, `CategoryNameEmpty`, `CategoryNotFound`, `TagAlreadyExists`, `TagNameEmpty`, `TagNotFound` |
| `TrackerError`       | `TrackerOffline`, `TrackerResponseError`, `TrackerUnknownResponse`, `TorrentNotFoundInTracker`, `InvalidTrackerToken` |
| `InfraError`         | `InternalServerError`, `DatabaseError`, `NotAUrl` |

This grouping is indicative; the exact split will be refined during
implementation.

### Migration Strategy

1. **Phase 0 — Prerequisite cleanup:** ✅ Done
   - Complete ADR-T-004 (remove `located-error`). ✅
   - Replace all `eprintln!()` in `From` impls with `tracing::error!()`. ✅
   - Add `Error` trait to cache error types. ✅
   - Remove unused `anyhow` dependency. ✅
   - Standardise on `thiserror` for all error derives. ✅

2. **Phase 1 — Extract domain errors:** ✅ Done
   - Extracted `AuthError` (13 variants: JWT, password, authorization).
   - Extracted `UserError` (22 variants: registration, profile, banning).
   - Extracted `TorrentError` (22 variants: upload, listing, tracker).
   - Retained `CategoryTagError` (already extracted prior to this ADR).
   - Each domain error has a co-located `status_code()` method and
     typed `From` impls for lower-level errors (`database::Error`,
     `argon2`, `sqlx`, `TrackerAPIError`, `MetadataError`, etc.).
   - All service functions updated to return domain-specific types.
   - All handlers and extractors updated.

3. **Phase 2 — Introduce `ApiError` wrapper:** ✅ Done
   - `ApiError` wraps `AuthError`, `UserError`, `TorrentError`, and
     `CategoryTagError` via `#[from]`.
   - `ApiError` implements `IntoResponse` by delegating to the
     wrapped domain error's `status_code()` and `Display` message.
   - Handlers that span multiple domains can use `ApiError` as
     their error type.

4. **Phase 3 — Cleanup:** ✅ Done
   - Removed `ServiceError` (41 variants) and `ServiceResult`.
   - Removed `http_status_code_for_service_error` and
     `map_database_error_to_service_error`.
   - Removed `IntoResponse for database::Error`.
   - All doc comments updated to reference domain error types.

5. **Phase 4 — Crate-level test suite:** ✅ Done
   - 188 crate tests in `src/tests/errors/` covering §1–§4 of the
     Testing Acceptance Guide.
   - `auth_error.rs` — 29 tests (status codes, display, `From` impls).
   - `user_error.rs` — 54 tests (status codes, display, `From` impls).
   - `torrent_error.rs` — 72 tests (status codes, display, `From` impls).
   - `category_tag_error.rs` — 25 tests (status codes, display, `From` impls).
   - `api_error.rs` — 8 tests (`status_code()` and `Display` delegation).
   - §5 (tracing capture) and §6 (integration `IntoResponse`) remain
     as future work.

### Handling Cross-Domain Errors in Handlers

Handlers that call multiple services (e.g. auth + torrent) will
unify errors via `From` impls into `ApiError`:

```rust
// Handler returns Result<_, ApiError>
let user = auth_service.verify(&token).await?;   // AuthError -> ApiError
let torrent = torrent_service.get(id).await?;     // TorrentError -> ApiError
```

A shared `InfraError` type will cover truly cross-cutting concerns
(database failures, I/O errors) and can be embedded in domain
errors via `#[source]` when needed.

## Consequences

- **`ServiceError` has been removed.** All code now uses
  domain-specific error types (`AuthError`, `UserError`,
  `TorrentError`, `CategoryTagError`) or `ApiError`.
- **Test assertions** work on the smaller domain enums — all four
  derive `PartialEq` + `Eq`.
- **HTTP response format** (`{"error": "<string>"}`) remains
  stable. `ApiError` and the domain error `IntoResponse` impls
  produce the same JSON shape as the old `ServiceError` mapping.
- **ADR-T-004 migration** (remove `located-error`, adopt `tracing`)
  was completed as a prerequisite (Phase 0).
- **Logging** uses `tracing` exclusively in error `From` impls;
  no `eprintln!` calls remain in error conversion code.
- **Cross-cutting variants** (`UnauthorizedAction`,
  `UnauthorizedActionForGuests`, `DatabaseError`,
  `InternalServerError`) are currently duplicated across domain
  enums with a `// see ADR-T-006` comment. A future refinement
  could extract a shared `InfraError` type to reduce this
  duplication.

## Testing Acceptance Guide

This section describes the testing requirements for the error system.
New error variants, `From` impls, and status-code mappings introduced
after this ADR **must** satisfy the criteria below before merging.

### 1. Status-Code Mapping (Crate Tests) ✅

Every variant of every domain error enum (`AuthError`, `UserError`,
`TorrentError`, `CategoryTagError`) **must** have a crate-level test
that asserts the expected `StatusCode` returned by `status_code()`.

```rust
// src/tests/errors.rs  (crate-level, §T-006 status-code tests)

use hyper::StatusCode;
use crate::errors::AuthError;

#[test]
fn auth_error_wrong_password_returns_forbidden() {
    assert_eq!(AuthError::WrongPasswordOrUsername.status_code(), StatusCode::FORBIDDEN);
}
```

Group the tests by domain error enum. One test per variant is the
baseline; variants that share a status code may be combined into a
single parametric test if preferred, but every variant must appear.

### 2. Display Messages (Crate Tests) ✅

Each variant's `Display` output is the string returned to API
consumers in the `{"error": "…"}` JSON body. A crate test **must**
assert the exact message string for every variant to prevent
accidental regressions.

```rust
#[test]
fn auth_error_token_expired_display_message() {
    assert_eq!(
        AuthError::TokenExpired.to_string(),
        "Token expired. Please sign in again.",
    );
}
```

### 3. `From` Impl Coverage (Crate Tests) ✅

Every `From<SourceError> for DomainError` impl **must** have at
least one test per mapping arm. In particular:

- **Specific mappings** (e.g. `database::Error::UserNotFound` →
  `AuthError::UserNotFound`) must each be asserted.
- **Catch-all / fallback arms** (e.g. `_ => Self::DatabaseError`)
  must be tested with at least one representative source variant
  that exercises the fallback.
- **Lossy conversions** (source error → `InternalServerError`) must
  verify the correct variant *and* confirm that the source error is
  logged (see §5 below).

```rust
#[test]
fn auth_error_from_database_user_not_found() {
    let err: AuthError = database::Error::UserNotFound.into();
    assert_eq!(err, AuthError::UserNotFound);
}

#[test]
fn auth_error_from_database_fallback() {
    let err: AuthError = database::Error::CategoryNotFound.into();
    assert_eq!(err, AuthError::DatabaseError);
}
```

### 4. `ApiError` Delegation (Crate Tests) ✅

`ApiError` must transparently delegate `status_code()` and
`Display` to the wrapped domain error. For each `ApiError` variant,
a test should assert:

- `ApiError::from(domain_error).status_code()` equals the inner
  error's `status_code()`.
- `ApiError::from(domain_error).to_string()` equals the inner
  error's `to_string()`.

```rust
#[test]
fn api_error_delegates_status_code_to_auth() {
    let inner = AuthError::TokenNotFound;
    let api = ApiError::from(inner);
    // TokenNotFound → UNAUTHORIZED
    assert_eq!(api.status_code(), StatusCode::UNAUTHORIZED);
}
```

### 5. Tracing Output on Lossy Conversions (Crate Tests) — Future

`From` impls that discard the source error (converting to
`InternalServerError`, `DatabaseError`, etc.) **must** log the
original error via `tracing::error!`. Tests should use a
[`tracing_subscriber::fmt::TestWriter`] or the `tracing-test` crate
to capture log output and assert the source error message appears.

At minimum, one test per `From` impl that performs a lossy
conversion should confirm the log line is emitted.

### 6. `IntoResponse` HTTP Shape (Integration Tests) — Future

Integration tests (`tests/e2e/`) should verify the end-to-end HTTP
response for representative error scenarios:

- **Status code** matches the domain error's `status_code()`.
- **Content-Type** is `application/json`.
- **Body** is `{"error": "<Display message>"}`.

These tests exercise the full Axum handler → domain error →
`IntoResponse` pipeline. At least one happy-path error per domain
should be covered (e.g. login with wrong password → 403, upload
duplicate torrent → 400, delete non-existent category → 404).

### 7. Exhaustiveness Guard

To prevent a new variant from being added without a corresponding
test, each domain error's status-code test module should include a
compile-time or runtime exhaustiveness check. The simplest approach
is a test that matches all variants in a `match` expression with no
wildcard arm — the compiler will error if a new variant is added
without updating the test:

```rust
#[test]
fn auth_error_status_code_is_exhaustive() {
    // This match must list every variant. Adding a new variant
    // to AuthError without updating this test will cause a
    // compiler error.
    let variants: Vec<AuthError> = vec![
        AuthError::WrongPasswordOrUsername,
        AuthError::InvalidPassword,
        AuthError::UsernameNotFound,
        AuthError::TokenNotFound,
        AuthError::TokenExpired,
        AuthError::TokenInvalid,
        AuthError::UnauthorizedAction,
        AuthError::UnauthorizedActionForGuests,
        AuthError::LoggedInUserNotFound,
        AuthError::EmailNotVerified,
        AuthError::InternalServerError,
        AuthError::DatabaseError,
        AuthError::UserNotFound,
    ];
    for variant in variants {
        // Just ensure status_code() doesn't panic.
        let _ = variant.status_code();
    }
}
```

### 8. Cross-Cutting Variant Consistency

The duplicated cross-cutting variants (`UnauthorizedAction`,
`UnauthorizedActionForGuests`, `DatabaseError`,
`InternalServerError`) **must** map to the same `StatusCode` in
every domain enum that contains them. A dedicated test should assert
this invariant:

```rust
#[test]
fn cross_cutting_variants_have_consistent_status_codes() {
    // UnauthorizedAction → FORBIDDEN everywhere
    assert_eq!(AuthError::UnauthorizedAction.status_code(), StatusCode::FORBIDDEN);
    assert_eq!(UserError::UnauthorizedAction.status_code(), StatusCode::FORBIDDEN);
    assert_eq!(TorrentError::UnauthorizedAction.status_code(), StatusCode::FORBIDDEN);
    assert_eq!(CategoryTagError::UnauthorizedAction.status_code(), StatusCode::FORBIDDEN);
}
```

### Summary Checklist

| # | What | Where | Blocking? |
|---|------|-------|-----------|
| 1 | `status_code()` per variant | `src/tests/` | Yes |
| 2 | `Display` message per variant | `src/tests/` | Yes |
| 3 | Every `From` mapping arm | `src/tests/` | Yes |
| 4 | `ApiError` delegation | `src/tests/` | Yes |
| 5 | Tracing on lossy conversions | `src/tests/` | Recommended |
| 6 | HTTP response shape (e2e) | `tests/e2e/` | Recommended |
| 7 | Exhaustiveness guard | `src/tests/` | Yes |
| 8 | Cross-cutting consistency | `src/tests/` | Yes |
