# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- ADR-T-008: Document rationale for roles and permissions refactor.
- ADR-T-006: Document rationale for error system refactor.
- 188 crate-level tests for the domain error system (`src/tests/errors/`):
  status-code mapping, display messages, `From` impl coverage, and
  `ApiError` delegation (ADR-T-006 §1–§4).
- Native `PermissionMatrix` replacing Casbin: compile-time checked `Role` and
  `Action` enums with an exhaustive default-deny policy table (ADR-T-008).
- `Permissions` trait abstraction for the authorization backend, consumed by
  the `RequirePermission<A>` extractor via `AppData.permissions`.
- `role: TEXT` column on `torrust_users` (migration for SQLite and MySQL);
  existing `administrator = true` rows migrated to `role = 'admin'`, others
  to `role = 'registered'`.
- `role: String` field on `TokenResponse`, `UserCompact`, `UserProfile`, and
  `UserFull` API response models.
- `RequirePermission<A>` Axum extractor enforcing role-based authorization at
  the HTTP boundary before the handler runs (ADR-T-008 Phase 2).
- `ActionMarker` trait and `action_markers!` macro mapping zero-sized types to
  `Action` enum variants for compile-time handler–permission binding.
- `Actor` struct yielded by `RequirePermission` carrying the resolved
  `user_id` and `Role` for downstream handler use.
- ADR-T-007: Document rationale for JWT system refactor.
- Centralised JWT module (`src/jwt.rs`) consolidating all `jsonwebtoken` usage:
  key loading, signing, verification, and algorithm configuration.
- `SessionClaims` with RFC 7519 registered claims (`sub`, `iss`, `aud`, `iat`,
  `exp`) plus advisory `role`, `username`, and revocation `gen` fields.
- `VerifyClaims` with `aud: "email-verification"` for purpose separation.
- RSA key pair configuration: `auth.private_key_path` / `auth.public_key_path`
  (or inline PEM via `auth.private_key_pem` / `auth.public_key_pem`).
- Ephemeral auto-generated RSA-2048 key pair when no keys are configured.
  Sessions do not survive server restarts with ephemeral keys. Deployers who
  want persistent sessions supply their own key pair via config.
- `torrust-generate-auth-keypair` CLI binary for generating RSA-2048 key pairs.
  Outputs both PEM blocks to stdout; refuses to run if stdout is a terminal.
- Container auto-generation of persistent auth keys on first boot. The entry
  script runs `torrust-generate-auth-keypair` and writes the PEM files to
  `/etc/torrust/index/auth/` on the volume. Sessions survive restarts with no
  manual setup.
- `kid` (Key ID) header in every JWT for future key rotation support.
- Configurable token lifetimes: `auth.session_token_lifetime_secs` (default:
  2 weeks) and `auth.email_verification_token_lifetime_secs` (default: ~10 years).
- `token_generation` column on `torrust_users` (migration for SQLite and MySQL).
- Token revocation: password changes, role changes (admin grant), and bans
  increment `token_generation`; tokens with an older `gen` claim are rejected.
- Consolidated session validation: `JsonWebToken::validate_session` is the
  sole entry point for verifying a session JWT, checking the token-generation
  counter, and rejecting banned users. All callers delegate here.
- `BearerToken` extractor rejects missing/malformed `Authorization` headers at
  the extraction boundary (`AuthError::TokenNotFound` / `AuthError::TokenInvalid`).
- `ExtractOptionalLoggedInUser` catches extraction rejection and returns `None`
  for anonymous requests.
- `AuthError::TokenRevoked` variant for revoked-token responses.
- Crate tests for the JWT module (session + email-verification round-trips,
  audience cross-contamination, tampered/garbage tokens).
- Crate tests for `parse_token` (valid extraction, whitespace trimming,
  empty bearer, missing prefix, non-ASCII rejection).

### Changed

- **BREAKING:** Raise MSRV from 1.85 to 1.88.
- **BREAKING:** `administrator: bool` replaced by `role: String` in API
  responses (`TokenResponse`, `UserCompact`, etc.). The legacy `admin: bool`
  field has been removed entirely (ADR-T-008).
- **BREAKING:** `administrator` column dropped from `torrust_users`; the
  `role: TEXT` column is now the sole authority. Migration
  `20260415000001_torrust_drop_administrator_column` handles both SQLite
  (table-rebuild) and MySQL (`DROP COLUMN`).
- **BREAKING:** `ACTION` enum renamed to `Action`; variants unchanged.
- All HTTP handlers that require authorization now use
  `RequirePermission<A>` extractors instead of calling
  `authorization::Service::authorize()` (ADR-T-008 Phase 2).
- Service methods no longer receive `maybe_user_id` for authorization
  purposes — they receive an already-authorized `Actor` or are called
  unconditionally.
- Unauthorized requests are rejected at the extractor boundary before
  reaching the service layer (fail-fast).
- **BREAKING:** JWT signing algorithm changed from HMAC-HS256 to RS256
  (RSA + SHA-256). Existing HS256 tokens are invalidated; users must re-login.
- **BREAKING:** JWT claims redesigned from `UserClaims { user, exp }` to
  `SessionClaims { sub, iss, aud, iat, exp, role, username, gen }`. Existing
  tokens without the new claims fail deserialization.
- **BREAKING:** Configuration keys changed — `auth.user_claim_token_pepper` /
  `auth.session_signing_key` / `auth.email_verification_signing_key` replaced
  by `auth.private_key_path` and `auth.public_key_path` (or inline PEM).
  Deployers must generate an RSA key pair.
- **BREAKING:** Replace `ServiceError` (41 variants) and `ServiceResult` with
  domain-scoped error enums: `AuthError`, `UserError`, `TorrentError`,
  `CategoryTagError`, and a thin `ApiError` wrapper (ADR-T-006).
- `Authentication::get_user_id_from_bearer_token` now takes `BearerToken`
  directly instead of `Option<BearerToken>`.
- `parse_token` returns `Result` instead of panicking on malformed headers.
- JWT `exp` validation relies solely on the `jsonwebtoken` library; redundant
  manual expiration check removed.
- Token signing uses `Result` propagation instead of `.unwrap()` / `.expect()`.
- `UserClaims` is now a type alias for `SessionClaims` (backward-compatible).
- `VerifyClaims` moved from `mailer` into the `jwt` module (re-exported for
  backward compatibility).
- Service functions now return domain-specific `Result<T, DomainError>` instead
  of `Result<T, ServiceError>`.
- Each domain error co-locates its HTTP status-code mapping via a
  `status_code()` method.
- Error `From` impls use `tracing::error!` instead of `eprintln!`.
- JWT session token `role` claim now carries the database `role` value
  directly (`"registered"`, `"admin"`) instead of the previous mapping
  (`"user"`, `"admin"`).
- v1→v2 upgrade path: `insert_imported_user` now writes the `role` column
  (`"admin"` / `"registered"`) instead of the removed `administrator` column.
- Standardise all error derives on `thiserror`.

### Removed

- `admin: bool` field from `TokenResponse`, `LoggedInUserData`, and
  `TokenRenewalData` — superseded by `role: String` (ADR-T-008).
- `UserCompact::is_admin()` convenience method — no longer needed after
  `admin: bool` removal.
- `administrator` column from `torrust_users` schema (migration for both
  SQLite and MySQL).
- `casbin` crate dependency and all Casbin-related code
  (`CasbinConfiguration`, `CasbinEnforcer`, the `ACTION` enum in
  SCREAMING_CASE) — replaced by the native `PermissionMatrix` (ADR-T-008).
- `unstable.auth.casbin` configuration section (`Unstable`, `Auth`, `Casbin`
  config structs in `src/config/v2/unstable.rs`).
- `bearer_token::Extract` wrapper struct (replaced by `BearerToken` directly).
- `get_optional_logged_in_user` free function (logic moved into extractors).
- `get_claims_from_bearer_token` private method on `Authentication` (inlined).
- `ClaimTokenPepper` / `JwtSigningSecret` / `user_claim_token_pepper` config
  keys (replaced by RSA key pair configuration).
- `ServiceError` enum and `ServiceResult` type alias from `src/errors.rs`.
- `http_status_code_for_service_error` and `map_database_error_to_service_error`
  helper functions.
- `IntoResponse` impl for `database::Error` (now handled by domain errors).
- `authorization::Service` struct — replaced by `RequirePermission<A>`
  extractors consulting `PermissionMatrix` directly (ADR-T-008 Phase 2).
- `ExtractLoggedInUser` (`user_id.rs`) and `ExtractOptionalLoggedInUser`
  (`optional_user_id.rs`) extractors — replaced by `RequirePermission<A>`
  (ADR-T-008 Phase 2).
- Dead `Action` variants `GetSettings` and `GetCanonicalInfoHash` (no
  corresponding handlers existed).

## [4.0.0] - 2026-03-23

### Added

- ADR-T-004: Document rationale for removing `located-error` package.
- ADR-T-005: Document rationale for Rust edition 2024 migration.

### Changed

- **BREAKING:** Raise MSRV from 1.83 to 1.85.
- **BREAKING:** Migrate workspace to Rust edition 2024.
- **BREAKING:** Bump workspace version from `3.1.0-develop` to `4.0.0-develop`.
- Upgrade `jsonwebtoken` from 9.3 to 10 (with `rust_crypto` feature).
- Upgrade `rand` from 0.9 to 0.10; rename `rand::Rng` to `rand::RngExt`.
- Promote `rust-2024-compatibility` lint group from `warn` to `deny`.
- Reformat imports across ~55 files to edition 2024 style.
- Simplify error types in `config` and `web::api::server` — replace
  `LocatedError<'static, dyn Error + Send + Sync>` with `Arc<dyn Error + Send + Sync>`.
- Emit `tracing::error!` events where `LocatedError` previously logged context.

### Removed

- **BREAKING:** Remove first-party `torrust-index-located-error` package
  (`packages/located-error/`). Use `tracing` for error-origin context instead.
