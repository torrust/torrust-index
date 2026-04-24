# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- ADR-T-009: Container infrastructure hardening (Phases 1, 2, 3, 4 & 5).
- `torrust-index-config` workspace crate (`packages/index-config/`)
  containing the parsing surface of the configuration system: schema
  modules, validator, `load_settings`, `Info`, `Error`, the
  `CONFIG_OVERRIDE_*` / `ENV_VAR_CONFIG_TOML*` constants, and the
  permission value types (`Role`, `Action`, `Effect`,
  `PermissionOverride`). Leaf crate \u2014 no `tokio`, `reqwest`,
  `sqlx`, `hyper`, `rustls`, `native-tls`, or `openssl` in its dep
  closure (ADR-T-009 Phase 3).
- `EXPOSE ${IMPORTER_API_PORT}/tcp` in Containerfile; port 3002 mapped in
  compose.
- `restart: unless-stopped` on index and tracker compose services.
- `DEBUG=1` env-var gate for entry-script shell tracing (`set -x`).
- Runtime image notes in `docs/containers.md`: healthcheck behaviour
  on both targets, the curated busybox applet subset, the Podman
  `--format docker` requirement for `HEALTHCHECK`, and entry-script
  debugging.
- Container runtime base split into two parallel stages
  (`runtime_release` and `runtime_debug`) layered onto a shared
  base-agnostic `runtime_assets` bundle, with `busybox_donor`,
  `busybox_preflight`, `etc_seed`, `adduser_preflight`, and a
  `preflight_gate` aggregator stage that wires donor-validation
  into the build graph for both variants (ADR-T-009 Phase 4, D4).
- Curated busybox applet subset in the release runtime base: a
  single root-only `/bin/busybox` (mode `0700 root:root`) plus
  symlinks for `sh`, `adduser`, `addgroup`, `install`, `mkdir`,
  `dirname`, `chown`, `chmod`, `tr`, `mktemp`, `cat`, `printf`,
  `rm`, `echo`, `grep`. The unprivileged `torrust` user gets
  `EACCES` on the busybox binary (and therefore on every applet
  symlink) after privilege drop (ADR-T-009 Phase 4, D4).
- `HEALTHCHECK` directive on the `debug` build target (was
  previously omitted), plus `torrust-index-health-check` in the
  debug image so the directive resolves. The debug `CMD` is now
  `["/usr/bin/torrust-index"]` so the debug image is a drop-in
  replacement for release (ADR-T-009 Phase 4).
- DEV-ONLY credential comments in `compose.yaml`.
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
- `Actor::try_user_id()` non-panicking accessor returning `Option<UserId>`,
  safe for handlers that may serve guests (ADR-T-008).
- `Actor::is_authenticated()` convenience predicate (ADR-T-008).
- Compile-time `action_markers!` ↔ `Action::ALL` sync assertion: adding an
  `Action` variant without a matching marker (or vice versa) is a compile
  error (ADR-T-008).
- E2E tests for non-owner update and delete denial (`and_non_owners` module
  in `tests/e2e/web/api/v1/contexts/torrent/contract.rs`) (ADR-T-008 Phase 4).
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
- `torrust-index-auth-keypair` CLI binary (initially shipped as
  `torrust-generate-auth-keypair`) for generating RSA-2048 key pairs.
  Emits a JSON object `{"private_key_pem": "...", "public_key_pem": "..."}`
  on stdout; refuses to run if stdout is a terminal.
- Container auto-generation of persistent auth keys on first boot. The entry
  script runs `torrust-index-auth-keypair`, splits the JSON output with `jq`,
  and writes the PEM files to `/etc/torrust/index/auth/` on the volume.
  Sessions survive restarts with no manual setup.
- `packages/index-cli-common/` library crate providing the shared P9
  scaffolding (`refuse_if_stdout_is_tty`, `init_json_tracing`, `emit`,
  `BaseArgs`) used by every Torrust Index helper binary (ADR-T-009 Phase 2).
- `packages/index-health-check/` workspace crate hosting the
  `torrust-index-health-check` binary, rewritten on top of `std::net::TcpStream`
  with no `reqwest`/`tokio`/TLS dependencies and Happy Eyeballs
  IPv6/IPv4 fallback (ADR-T-009 Phase 2).
- `packages/index-auth-keypair/` workspace crate hosting the
  `torrust-index-auth-keypair` binary; the previous `[[bin]]` entry on the
  root crate was removed so the helper no longer inherits the application's
  HTTP/TLS dep closure (ADR-T-009 Phase 2).
- `jq_donor` build stage in `Containerfile` providing `jq` to the runtime
  image so the entry script can extract PEM keys from the keypair helper's
  JSON output (ADR-T-009 Phase 2).
- `#[doc(hidden)] pub mod test_helpers` in `torrust-index-config` exposing
  `PLACEHOLDER_TOML` (the canonical "minimal but legal" TOML) and
  `placeholder_settings()` (loads it via `load_settings`, panicking on
  failure). Single source of truth for the ~40 tests across both crates
  that previously relied on the now-removed ambient `Settings::default()`
  fixture (ADR-T-009 Phase 5).
- `Configuration::for_tests` (test-only, `pub(crate)`) on the root crate's
  runtime `Configuration` wrapper, replacing the deleted
  `impl Default for Configuration` and seeding from `placeholder_settings()`
  (ADR-T-009 Phase 5).
- `clear_inherited_config_env()` test helper in `src/tests/config/` that
  strips `TORRUST_INDEX_CONFIG_OVERRIDE_*` and
  `TORRUST_INDEX_CONFIG_TOML[_PATH]` inside a `figment::Jail` closure so
  default-configuration assertions stay deterministic when the suite is
  re-run after an e2e session (ADR-T-009 Phase 5).
- New loader tests `missing_database_connect_url_is_rejected` and
  `missing_database_section_is_rejected` covering the two new mandatory
  failure paths (ADR-T-009 Phase 5).
- Inverted shipped-sample test suite
  (`packages/index-config/tests/shipped_samples.rs`):
  `every_shipped_index_toml_omits_credentials` asserts no shipped sample
  carries `connect_url`, `token =`, `[mail.smtp]`, `private_key_path`, or
  `public_key_path`; `every_shipped_index_toml_demands_runtime_secrets`
  asserts the schema rejects each sample with a missing-field error
  mentioning `token` or `connect_url` (ADR-T-009 Phase 5, §D2).
- Default `TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL` in
  `compose.yaml` matching the SQLite path the entry script materialises,
  alongside the existing `TRACKER__TOKEN` default (ADR-T-009 Phase 5).
- `TRACKER__TOKEN` and `DATABASE__CONNECT_URL` exports in the mysql and
  sqlite e2e runner scripts so the host-side `cargo test` process sees
  the same overrides the container receives (ADR-T-009 Phase 5).
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

- `.containerignore` now excludes `/adr/` and `/docs/` from the build
  context (ADR-T-009 Phase 1).
- Container `HEALTHCHECK` now invokes `torrust-index-health-check` (was
  `health_check`); the binary is rewritten in stdlib-only Rust with no
  `reqwest`/`tokio`/TLS in its dep closure (ADR-T-009 Phase 2).
- Container entry script now invokes `torrust-index-auth-keypair` (was
  `torrust-generate-auth-keypair`) and consumes its JSON output via
  `jq -r .private_key_pem` / `jq -r .public_key_pem` instead of `sed`
  PEM-block extraction (ADR-T-009 Phase 2).
- Helper-binary TTY-refusal exit code unified on 2 (was 1 for the
  keypair helper) via the shared `refuse_if_stdout_is_tty` in
  `torrust-index-cli-common` (ADR-T-009 Phase 2).
- **BREAKING:** TLS configuration renamed from `[net.tsl]` to `[net.tls]`
  in operator TOMLs and from `"tsl"` to `"tls"` in the settings JSON
  API response. The original spelling was a typo; corrected as a clean
  break (no compatibility alias) alongside the Phase 3 config-crate
  extraction (ADR-T-009 Phase 3).
- Configuration parsing surface moved from `src/config/` into the new
  `torrust-index-config` workspace crate. `src/config/mod.rs` is now a
  thin re-export shim plus the runtime `Configuration` wrapper holding
  `RwLock<Settings>`; existing `use crate::config::*;` call sites
  continue to compile unchanged (ADR-T-009 Phase 3).
- Permission value types (`Role`, `Action`, `Effect`,
  `PermissionOverride`, `RoleParseError`) moved to
  `torrust_index_config::permissions` and re-exported from
  `crate::services::authorization` for backwards compatibility. The
  `Permissions` trait and `PermissionMatrix` runtime policy stay in
  the root crate (ADR-T-009 Phase 3).
- Container entry script now uses the busybox short-option form
  for `adduser` (`adduser -D -s /bin/sh -u "$USER_ID" torrust`) so
  the same invocation works on both runtime bases. Distroless
  `cc-debian13` ships `/etc/passwd` and `/etc/group` but not
  `/etc/shadow`; `-D` honours that (ADR-T-009 Phase 4).
- Helper binaries (`torrust-index-health-check`,
  `torrust-index-auth-keypair`) tightened from world-executable
  to `0500 root:root` in both `release` and `debug` images. The
  application binary (`torrust-index`) keeps `0755`. The
  `HEALTHCHECK` directive runs as root, so the tightened mode is
  sufficient (ADR-T-009 Phase 4, D4).
- `PATH` is now pinned in both runtime bases
  (`/usr/local/bin:/bin:/usr/bin:/sbin` for release;
  `/usr/local/bin:/busybox:/bin:/usr/bin:/sbin` for debug) so the
  entry script's bare-name lookups resolve deterministically
  regardless of future base-image changes (ADR-T-009 Phase 4).
- **BREAKING:** Container `USER_ID` validation rule changed from
  `USER_ID >= 1000` to "non-negative integer, not `0`" (D7). The
  previous rule rejected legitimate configurations (rootless
  Podman with subuid remapping, low-UID CI runners, BSD-derived
  hosts) without stating its intent. The property the entry
  script actually enforces is "do not run as root"; that is now
  what it checks (ADR-T-009 Phase 4, D7).
- **BREAKING:** `database.connect_url` and `tracker.token` are now
  mandatory in the parsed configuration; the schema-level
  `#[serde(default = "...")]` attributes and the
  `impl Default for Database` / `impl Default for Tracker` blocks
  have been removed. Omitting either field (or its enclosing
  `[database]` / `[tracker]` section) now fails configuration
  loading with a precise serde `missing field` error pointing at
  the exact section. Operators must supply both via env-var
  override (`TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL`,
  `..._TRACKER__TOKEN`) or a side-loaded TOML; zero-config startup
  is intentionally rejected (ADR-T-009 Phase 5, §D2).
- **BREAKING:** All shipped sample TOMLs under
  `share/default/config/` no longer carry `connect_url`, `token`,
  `[mail.smtp]` values, or `[auth]` key paths. The
  container-oriented samples (`index.*.container.*.toml`,
  `index.private.e2e.container.sqlite3.toml`,
  `index.public.e2e.container.*.toml`) and the bare-metal
  `index.development.sqlite3.toml` template are all affected;
  the two `tracker.*.e2e.container.sqlite3.toml` files lose
  their `[tracker].token` value as well. Bare-metal developers
  who copy `index.development.sqlite3.toml` verbatim must now
  supply `connect_url` and `token` themselves (ADR-T-009
  Phase 5, §D2).
- `load_settings` no longer terminates with
  `figment.join(Serialized::defaults(Settings::default()))`.
  Optional sub-sections still default through their per-field
  `#[serde(default = "...")]` attributes; mandatory fields no
  longer have anywhere to silently come from (ADR-T-009 Phase 5).
- `check_mandatory_options` no longer covers `tracker.token`; its
  absence now surfaces through serde rather than the bespoke
  pre-flight check, giving a single consistent error shape for
  every missing mandatory field (ADR-T-009 Phase 5).
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
- First-user auto-admin grant in `RegistrationService::register` now logs
  a `warn!` on failure instead of silently discarding the `Result` via
  `drop()`.
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
- Container base images upgraded from Debian bookworm to trixie
  (`rust:bookworm` → `rust:trixie`, `cc-debian12` → `cc-debian13`).
- `cargo-binstall` bootstrap pinned to tag `v1.18.1` (was `main` branch).
- MySQL compose image pinned to `8.0.45`; auth flag changed from
  `--default-authentication-plugin` to `--authentication-policy`.
- MySQL healthcheck uses `$$MYSQL_ROOT_PASSWORD` instead of broken
  `/run/secrets/db-password` reference.
- Dev-only compose ports (tracker, MySQL, mailcatcher) bound to `127.0.0.1`.
- Entry script `set -x` gated behind `DEBUG=1` to avoid leaking env vars.
- Entry script `USER_ID` guard: `&&` → `||` (was always-false when unset).
- `.dockerignore` renamed to `.containerignore` for Podman compatibility.
- Removed redundant `--tests --benches --examples` from Containerfile (covered
  by `--all-targets`).
- `docs/containers.md`: fixed `USER_UID` → `USER_ID` typo; removed
  "(i.e. the Dockerfile)" phrasing.
- Removed stale `TORRUST_TRACKER_USER_UID` export from E2E container scripts.

### Fixed

- MySQL compose healthcheck: was referencing a non-existent Docker secret
  (`/run/secrets/db-password`), now uses `$$MYSQL_ROOT_PASSWORD`.
- Entry script `USER_ID` validation: `-z "$USER_ID" && "$USER_ID" -lt 1000`
  always short-circuited to an error when `USER_ID` was unset; corrected to
  `||`.
- Containerfile release `HEALTHCHECK` trailing whitespace removed.

### Security

- Dev-only ports (MySQL 3306, tracker 6969/7070/1212, mailcatcher 1025/1080)
  no longer bind to `0.0.0.0`; bound to `127.0.0.1`.
- Entry script no longer unconditionally traces commands containing credentials.
- Compose credentials annotated as DEV-ONLY with TODO for Docker secrets
  migration (ADR-T-009 §S1).

### Removed

- Build-time `ARG API_PORT` / `ARG IMPORTER_API_PORT` from `Containerfile`
  (ADR-T-009 Phase 1, D6). The runtime `ENV API_PORT=3001` /
  `ENV IMPORTER_API_PORT=3002` defaults are retained so the listener and
  `HEALTHCHECK` resolve correctly; runtime `--env` overrides continue to
  work, but image metadata (`docker inspect`) now reflects the defaults
  unconditionally.
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
- `contrib/dev-tools/container/build.sh` — stale; passed wrong build-arg and
  assumed a `Dockerfile` that no longer exists.
- `contrib/dev-tools/container/run.sh` — stale; mounted wrong paths and read a
  removed config file name.
- Monolithic `runtime` Containerfile stage and its ad-hoc
  `cp -sp` busybox-applet copy (`sh`, `cat`, `ls`, `env`),
  superseded by the curated symlink loop and the
  `runtime_release` / `runtime_debug` split (ADR-T-009 Phase 4).
- `RUN env` and `CMD ["sh"]` lines from the previous debug
  target — debug now ships the same `ENTRYPOINT` /
  `CMD ["/usr/bin/torrust-index"]` / `HEALTHCHECK` block as
  release; operators reach a shell with `docker run … sh`
  (ADR-T-009 Phase 4).
- `impl Default for Settings`, `impl Default for Tracker`,
  `impl Default for Database`, and the matching
  `#[serde(default = "...")]` attributes on `Settings::tracker`,
  `Settings::database`, `Tracker::token`, and
  `Database::connect_url`. Also removed: `Tracker::default_token()`
  and `Settings::default_tracker()` (now dead code) (ADR-T-009
  Phase 5).
- `impl Default for Configuration` on the runtime wrapper —
  superseded by the test-only `Configuration::for_tests`
  (ADR-T-009 Phase 5).

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
