# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

**Highlights:** global command-line output contract (ADR-T-010), container infrastructure refactor (ADR-T-009), native role-based authorization replacing Casbin (ADR-T-008), RSA-signed JWTs with revocation support (ADR-T-007), domain-scoped error system (ADR-T-006), MSRV policy and raise to 1.89 (ADR-T-011), independent package versioning (ADR-T-012).

### Breaking changes

- Every workspace crate now declares its own `version`, and `[workspace.package]` no longer carries one (ADR-T-012). `torrust-index` keeps `4.0.0-develop`; the six member crates that inherited the workspace version are now at `0.1.0`, down from `4.0.0-develop`. `torrust-mudlark` keeps `1.0.0` and `torrust-index-render-text-as-image` keeps `0.1.0`, the versions they had already declared for themselves rather than inherited. None of the crates that moved has ever been on crates.io, so no dependency requirement and no lock file ever resolved the numbers they carried, and `0.x` is the honest statement of an API that has not been released. Sibling dependencies now pin the version each crate declares beside its `path`, which is the requirement `cargo publish` writes into a published manifest; a `4.0.0-develop` requirement named nothing the registry could resolve. Crates are published one at a time from a `releases/pkg/<crate-name>/v<semver>` branch, and the application release publishes only `torrust-index`.
- MSRV raised from 1.85 to 1.89. ADR-T-011 replaces the dependency-driven floor with a computed one: `rust-version` is the newest stable Rust released at least one year before the day the pin is computed, recomputed at every release and in every maintenance pass. Computed on 2026-09-05 that is 1.89.
- First-party command-line entrypoints are now governed by ADR-T-010's
  JSON-only output contract. Stdout is reserved for machine-readable result
  data, stderr is reserved for machine-readable diagnostics/control records, and
  stdout-producing commands refuse direct terminal stdout. `parse_torrent` now
  emits JSON result data on stdout. `create_test_torrent`,
  `import_tracker_statistics`, `seeder`, and `upgrade` now keep stdout empty
  while reporting status and diagnostics as JSON on stderr. The container entry
  script also reports validation failures, status records, utility failures, and
  debug phase records as JSON/NDJSON on stderr instead of plain text or shell
  trace output. Scripts that scraped previous plain-text command or startup
  output must switch to exit codes and JSON/NDJSON stderr parsing.
- The `torrust-index` server's application logs now use JSON records on stderr
  instead of the previous human-formatted tracing output. Log consumers should
  parse stderr as NDJSON or pipe it through a JSON viewer.
- `torrust-index-auth-keypair` and `torrust-index-health-check` stdout JSON now
  includes a top-level `schema` field. Scripts that expected an exact object
  shape must tolerate or consume the schema field.
- `database.connect_url` and `tracker.token` are now mandatory
  schema fields with no defaults. Supply them via env-var override
  (`TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL`,
  `..._TRACKER__TOKEN`) or a side-loaded `index.toml`. Missing values
  fail at parse time with a precise `missing field` error
  (ADR-T-009 §D2).
- TLS configuration renamed from `[net.tsl]` to `[net.tls]` in
  operator TOMLs and from `"tsl"` to `"tls"` in the settings JSON
  API. Clean break, no compatibility alias (ADR-T-009 §D3).
- `TORRUST_INDEX_DATABASE_DRIVER` no longer dispatches the
  application's runtime driver — that is derived from the URL
  scheme of `database.connect_url` (ADR-T-009 §D2). It is retained
  as an input-validation gate at container start (`sqlite3` /
  `mysql`); both values seed the same driver-agnostic
  `index.container.toml` template into `/etc/torrust/index/` on
  first boot.
- `TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__*_PEM` and `..._PATH` are
  mutually exclusive within a single key, both keys must use the
  same delivery mechanism, and the pair must either both be set or
  both be absent (ADR-T-009 §D3).
- Container `USER_ID` validation changed from `>= 1000` to
  "non-negative integer, not `0`". The property the entry script
  actually enforces is "do not run as root" (ADR-T-009 §D7).
- JWT signing changed from HMAC-HS256 to RS256. Existing tokens are
  invalidated; users must re-login (ADR-T-007).
- JWT claims redesigned from `UserClaims { user, exp }` to
  `SessionClaims { sub, iss, aud, iat, exp, role, username, gen }`
  (ADR-T-007).
- Auth config keys `auth.user_claim_token_pepper`,
  `auth.session_signing_key`, and `auth.email_verification_signing_key`
  replaced by `auth.private_key_path` / `auth.public_key_path` (or
  inline PEM). Deployers must generate an RSA key pair (ADR-T-007).
- `administrator: bool` replaced by `role: String` across the API
  (`TokenResponse`, `UserCompact`, etc.); the legacy `admin: bool`
  field is removed entirely (ADR-T-008).
- `administrator` column dropped from `torrust_users`; `role: TEXT`
  is now sole authority (ADR-T-008).
- `ACTION` enum renamed to `Action` (variants unchanged) (ADR-T-008).
- `ServiceError` (41 variants) and `ServiceResult` replaced by
  domain-scoped enums: `AuthError`, `UserError`, `TorrentError`,
  `CategoryTagError`, with a thin `ApiError` wrapper (ADR-T-006).

### ADR-T-010 — Global command-line output contract

#### Added

- ADR-T-010 establishes a repository-wide JSON-only output contract for
  first-party command-line entrypoints. Stdout is reserved for result data;
  stderr carries diagnostics and control records; commands that emit stdout
  result data refuse direct terminal stdout.
- `torrust-index-cli-common` provides the shared implementation for that
  contract: JSON `clap` help/version/usage handling, JSON panic diagnostics,
  JSON stderr tracing, TTY refusal, stdout JSON emission, command runners,
  baseline exit-code classes, control-plane record types, and redaction
  helpers.
- Regression coverage now protects the contract with CLI behavior tests,
  binary-boundary checks, and workspace lint rules denying accidental raw
  stream output or direct process exits outside the shared CLI boundary.

#### Changed

- Helper binaries (`torrust-index-auth-keypair`,
  `torrust-index-config-probe`, and `torrust-index-health-check`) share the
  JSON CLI boundary. Their successful stdout payloads remain single JSON
  objects, now explicitly versioned with a top-level `schema` field, while
  help, version, argv errors, TTY refusal, panic diagnostics, and tracing are
  emitted as JSON records on stderr.
- The `torrust-index` server and root Rust binaries return explicit
  `ExitCode` values at their `main` boundaries and install the shared JSON
  panic hook. Central application logging uses JSON tracing on stderr, with a
  non-empty `RUST_LOG` taking precedence over the configured default filter.
- `parse_torrent` is a stdout-result command. It emits one JSON object with
  `schema`, `torrent`, `original_v1_info_hash`, and `input_byte_length`, leaves
  stdout empty on failure, and refuses direct terminal stdout with a JSON
  diagnostic record.
- `create_test_torrent`, `import_tracker_statistics`, `seeder`, and `upgrade`
  are no-stdout side-effect commands. They keep stdout empty, report status and
  diagnostics as JSON/NDJSON on stderr, and propagate command failures instead
  of printing plain text or relying on panic output.
- Command-reachable shared libraries use the command diagnostic path instead of
  raw stream output. Shutdown notices are structured tracing records, mail
  template failures are returned to callers, terminal color formatting is
  removed from command paths, and parsing helpers leave reporting decisions to
  their command callers.
- The container entry script follows the JSON stderr contract during startup:
  it captures helper stdout internally, keeps its own stdout empty before
  `su-exec`, checks for `jq` before JSON-dependent helpers run, emits explicit
  `DEBUG=1` phase records instead of `set -x`, and wraps controlled utility
  failures with captured stderr fields.
- Operator documentation and command examples describe the completed contract
  across the README, container guide, upgrade notes, and command module docs.

### ADR-T-009 — Container infrastructure refactor

#### Added

- ADR-T-009 itself.
- `torrust-index-config` workspace crate
  (`packages/index-config/`) containing the parsing surface of the
  configuration system. Leaf crate — no `tokio`, `reqwest`, `sqlx`,
  `hyper`, `rustls`, `native-tls`, or `openssl` in its dep closure.
- Helper-binary crates split into leaves with no HTTP/TLS in their
  dep closure: `torrust-index-cli-common` (shared ADR-T-010 scaffolding —
  `refuse_if_stdout_is_tty`, `init_json_tracing`, `emit`, `BaseArgs`),
  `torrust-index-health-check` (stdlib-only, Happy Eyeballs IPv6/IPv4
  fallback), `torrust-index-auth-keypair` (RSA-2048 key generator),
  `torrust-index-config-probe` (resolved-config JSON emitter), and
  `torrust-index-entry-script` (test-only host-side driver).
- Sourced POSIX `sh` library at `share/container/entry_script_lib_sh`
  containing the entry script's pure helpers (`inst`,
  `key_configured`, `validate_auth_keys`, `seed_sqlite`). Shipped as
  `0444 root:root` and sourced — not exec'd — by both the entry
  script and the host-side test crate (§D3).
- Top-level `Makefile` with `make up-dev` (plain `docker compose up`)
  and `make up-prod` (validates required credential env vars, runs
  with the override excluded) (§D1).
- `compose.override.yaml` auto-loaded by Compose v2, re-introducing
  the `mailcatcher` sidecar, `tty: true`, and permissive
  `${VAR:-default}` substitutions on top of the production-shaped
  baseline (§D1).
- `contrib/dev-tools/su-exec/AUDIT.md` recording provenance and a
  SHA-256-anchored append-only audit log for the vendored
  `su-exec.c`. CI fails the build when the file changes without a
  matching audit entry (§D8).
- `jq_donor` Containerfile stage providing `jq` (and required shared
  libs) from a pristine `rust:slim-trixie` base; both runtime images
  copy `/usr/bin/jq` as `0500 root:root`. Used during the
  pre-`su-exec` phase to parse the config probe's and keypair
  helper's JSON output. An `ldd`-based allow-list catches future
  transitive-dep changes at build time (§D5).
- Container runtime split into parallel `runtime_release` and
  `runtime_debug` stages over a shared base-agnostic `runtime_assets`
  bundle, with `busybox_donor`, `busybox_preflight`, `etc_seed`,
  `adduser_preflight`, and a `preflight_gate` aggregator (§D4).
- Curated busybox applet subset in the release runtime: a single
  root-only `/bin/busybox` (`0700 root:root`) plus symlinks for `sh`,
  `adduser`, `addgroup`, `install`, `mkdir`, `dirname`, `chown`,
  `chmod`, `tr`, `mktemp`, `cat`, `printf`, `rm`, `echo`, `grep`.
  The unprivileged `torrust` user gets `EACCES` on busybox after
  privilege drop (§D4).
- Container auto-generation of persistent auth keys on first boot:
  the entry script runs `torrust-index-auth-keypair`, splits the
  JSON output with `jq`, and writes PEMs to
  `/etc/torrust/index/auth/` on the volume.
- `HEALTHCHECK` directive on the `debug` build target (was
  previously omitted); debug `CMD` is now
  `["/usr/bin/torrust-index"]` so debug is a drop-in replacement for
  release (Phase 4).
- `Info::from_env` constructor on `torrust-index-config` — the
  JSON-safe sibling of `Info::new` that skips diagnostic `println!`s
  and filters empty-string env vars.
- `pub const DEFAULT_CONFIG_TOML_PATH` shared between the
  application, helper binaries, and integration tests.
- `ApiToken::is_empty()` accessor so the config probe can reject
  `tracker.token = ""` at the container boundary.
- `# ENTRY_ENV_VARS:` / `# END_ENTRY_ENV_VARS` canonical manifest
  block in the entry script; CI verifies every name is documented in
  `docs/containers.md`.
- `EXPOSE ${IMPORTER_API_PORT}/tcp` in Containerfile; port 3002
  mapped in compose.
- `restart: unless-stopped` on index and tracker compose services.
- `DEBUG=1` env-var gate for entry-script JSON phase diagnostics.
- `#[doc(hidden)] pub mod test_helpers` in `torrust-index-config`
  exposing `PLACEHOLDER_TOML` and `placeholder_settings()` — single
  source of truth for the ~40 tests across both crates that
  previously relied on the removed `Settings::default()` fixture
  (Phase 5).
- `Configuration::for_tests` (test-only, `pub(crate)`) replacing the
  deleted `impl Default for Configuration` (Phase 5).
- `clear_inherited_config_env()` test helper that strips
  `TORRUST_INDEX_CONFIG_OVERRIDE_*` and
  `TORRUST_INDEX_CONFIG_TOML[_PATH]` inside a `figment::Jail` so
  default-configuration assertions stay deterministic (Phase 5).
- Inverted shipped-sample test suite asserting no shipped TOML
  carries `connect_url`, `token`, `[mail.smtp]`, or auth key paths,
  and that the schema rejects each sample with a missing-field error
  (Phase 5, §D2).
- New loader tests `missing_database_connect_url_is_rejected` and
  `missing_database_section_is_rejected` (Phase 5).
- Default `TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL` in
  `compose.yaml`, alongside the existing `TRACKER__TOKEN` default;
  matching exports in the mysql and sqlite e2e runner scripts.

#### Changed

- Configuration parsing surface moved from `src/config/` into the new
  `torrust-index-config` workspace crate. `src/config/mod.rs` is now
  a thin re-export shim plus the runtime `Configuration` wrapper;
  existing `use crate::config::*;` call sites compile unchanged.
- Permission value types (`Role`, `Action`, `Effect`,
  `PermissionOverride`, `RoleParseError`) moved to
  `torrust_index_config::permissions` and re-exported from
  `crate::services::authorization`. The `Permissions` trait and
  `PermissionMatrix` runtime policy stay in the root crate.
- `load_settings` no longer ends with
  `figment.join(Serialized::defaults(Settings::default()))`. Optional
  sub-sections still default through their per-field `#[serde(default)]`
  attributes; mandatory fields no longer have a silent fallback.
- `check_mandatory_options` no longer covers `tracker.token`; its
  absence now surfaces through serde for a single consistent error
  shape across all missing mandatory fields.
- `Info::new` routes its "loading extra configuration from …"
  diagnostics through `tracing` (stderr) instead of `println!`
  (stdout).
- Container `HEALTHCHECK` invokes `torrust-index-health-check` (was
  `health_check`); the binary is rewritten in stdlib-only Rust.
- Entry script invokes `torrust-index-auth-keypair` (was
  `torrust-generate-auth-keypair`) and consumes its JSON output via
  `jq -r .private_key_pem` instead of `sed` PEM-block extraction.
- Entry script uses busybox short-option form for `adduser` so the
  same invocation works on both runtime bases (distroless
  `cc-debian13` ships `/etc/passwd` and `/etc/group` but not
  `/etc/shadow`).
- Helper binaries (`torrust-index-health-check`,
  `torrust-index-auth-keypair`) tightened from world-executable to
  `0500 root:root`. The application binary keeps `0755`.
- `PATH` pinned in both runtime bases
  (`/usr/local/bin:/bin:/usr/bin:/sbin` for release;
  `/usr/local/bin:/busybox:/bin:/usr/bin:/sbin` for debug) so the
  entry script's bare-name lookups resolve deterministically.
- Helper-binary TTY-refusal exit code unified on `2` (was `1` for
  the keypair helper) via the shared `refuse_if_stdout_is_tty`.
- `.containerignore` now excludes `/adr/` and `/docs/` from the
  build context.
- Container base images upgraded from Debian bookworm to trixie
  (`rust:bookworm` → `rust:trixie`, `cc-debian12` → `cc-debian13`).
- `cargo-binstall` bootstrap pinned to tag `v1.18.1` (was `main`).
- MySQL compose image pinned to `8.0.45`; auth flag changed from
  `--default-authentication-plugin` to `--authentication-policy`.
- Dev-only compose ports (tracker, MySQL, mailcatcher) bound to
  `127.0.0.1`.
- `.dockerignore` renamed to `.containerignore` for Podman
  compatibility.
- Removed redundant `--tests --benches --examples` from
  Containerfile (covered by `--all-targets`).
- All shipped sample TOMLs under `share/default/config/` no longer
  carry `connect_url`, `token`, `[mail.smtp]` values, or `[auth]`
  key paths. Bare-metal developers who copy
  `index.development.sqlite3.toml` verbatim must now supply
  `connect_url` and `token` themselves (Phase 5, §D2).
- DEV-ONLY credential comments in `compose.yaml`.
- `docs/containers.md`: documented healthcheck behaviour, busybox
  applet subset, Podman `--format docker` requirement for
  `HEALTHCHECK`, entry-script debugging; fixed `USER_UID` →
  `USER_ID` typo.

#### Removed

- Build-time `ARG API_PORT` / `ARG IMPORTER_API_PORT`. The runtime
  `ENV` defaults are retained so the listener and `HEALTHCHECK`
  resolve correctly; runtime `--env` overrides continue to work
  (§D6).
- Monolithic `runtime` Containerfile stage and its ad-hoc `cp -sp`
  busybox-applet copy, superseded by the curated symlink loop and
  the release/debug split.
- `RUN env` and `CMD ["sh"]` from the previous debug target — debug
  now ships the same `ENTRYPOINT` / `CMD` / `HEALTHCHECK` as
  release; operators reach a shell with `docker run … sh`.
- `impl Default for Settings`, `impl Default for Tracker`,
  `impl Default for Database`, the matching `#[serde(default = …)]`
  attributes, and the now-dead `Tracker::default_token()` /
  `Settings::default_tracker()` (Phase 5).
- `impl Default for Configuration` — superseded by
  `Configuration::for_tests` (Phase 5).
- `contrib/dev-tools/container/build.sh` and `run.sh` — both stale,
  passed wrong build-args and assumed a `Dockerfile` that no longer
  exists.
- Stale `TORRUST_TRACKER_USER_UID` export from E2E container
  scripts.

### ADR-T-008 — Roles & permissions

#### Added

- ADR-T-008 itself.
- Native `PermissionMatrix` replacing Casbin: compile-time checked
  `Role` and `Action` enums with an exhaustive default-deny policy
  table.
- `Permissions` trait abstraction for the authorization backend,
  consumed by the `RequirePermission<A>` extractor via
  `AppData.permissions`.
- `role: TEXT` column on `torrust_users` (migration for SQLite and
  MySQL); existing `administrator = true` rows migrated to
  `role = 'admin'`, others to `role = 'registered'`.
- `role: String` field on `TokenResponse`, `UserCompact`,
  `UserProfile`, and `UserFull` API response models.
- `RequirePermission<A>` Axum extractor enforcing role-based
  authorization at the HTTP boundary before the handler runs.
- `ActionMarker` trait and `action_markers!` macro mapping
  zero-sized types to `Action` variants for compile-time
  handler–permission binding. A compile-time sync assertion catches
  divergence between `action_markers!` and `Action::ALL`.
- `Actor` struct yielded by `RequirePermission` carrying the
  resolved `user_id` and `Role`, with `try_user_id()` and
  `is_authenticated()` helpers.
- E2E tests for non-owner update and delete denial
  (`and_non_owners` module under torrent contract tests).

#### Changed

- All HTTP handlers requiring authorization use
  `RequirePermission<A>` extractors instead of calling
  `authorization::Service::authorize()`.
- Service methods no longer receive `maybe_user_id` for
  authorization — they receive an already-authorized `Actor` or are
  called unconditionally. Unauthorized requests are rejected at the
  extractor boundary (fail-fast).
- First-user auto-admin grant in `RegistrationService::register`
  now logs a `warn!` on failure instead of silently discarding the
  `Result` via `drop()`.
- v1→v2 upgrade path: `insert_imported_user` writes the `role`
  column instead of the removed `administrator` column.

#### Removed

- `admin: bool` from `TokenResponse`, `LoggedInUserData`, and
  `TokenRenewalData` — superseded by `role: String`.
- `UserCompact::is_admin()` — no longer needed.
- `casbin` crate dependency and all Casbin-related code
  (`CasbinConfiguration`, `CasbinEnforcer`, the SCREAMING_CASE
  `ACTION` enum, `unstable.auth.casbin` config section).
- `authorization::Service` struct — replaced by
  `RequirePermission<A>` consulting `PermissionMatrix` directly.
- `ExtractLoggedInUser` and `ExtractOptionalLoggedInUser`
  extractors — replaced by `RequirePermission<A>`.
- Dead `Action` variants `GetSettings` and `GetCanonicalInfoHash`.

### ADR-T-007 — JWT refactor

#### Added

- ADR-T-007 itself.
- Centralised JWT module (`src/jwt.rs`) consolidating all
  `jsonwebtoken` usage: key loading, signing, verification,
  algorithm configuration.
- `SessionClaims` with RFC 7519 registered claims (`sub`, `iss`,
  `aud`, `iat`, `exp`) plus advisory `role`, `username`, and
  revocation `gen` fields.
- `VerifyClaims` with `aud: "email-verification"` for purpose
  separation.
- RSA key pair configuration: `auth.private_key_path` /
  `auth.public_key_path` (or inline PEM via `..._pem`).
- Ephemeral auto-generated RSA-2048 key pair when no keys are
  configured. Sessions do not survive server restarts with ephemeral
  keys.
- `kid` (Key ID) header in every JWT for future key rotation.
- Configurable token lifetimes:
  `auth.session_token_lifetime_secs` (default: 2 weeks) and
  `auth.email_verification_token_lifetime_secs` (default: ~10 years).
- `token_generation` column on `torrust_users` (SQLite + MySQL
  migrations).
- Token revocation: password changes, role changes (admin grant),
  and bans increment `token_generation`; tokens with an older `gen`
  claim are rejected.
- `JsonWebToken::validate_session` as the sole entry point for
  verifying a session JWT, the token-generation counter, and the
  banned-user check. All callers delegate here.
- `BearerToken` extractor rejects missing/malformed `Authorization`
  headers at the extraction boundary
  (`AuthError::TokenNotFound` / `AuthError::TokenInvalid`).
- `ExtractOptionalLoggedInUser` returns `None` for anonymous
  requests instead of erroring.
- `AuthError::TokenRevoked` variant for revoked-token responses.
- Crate tests for the JWT module (session +
  email-verification round-trips, audience cross-contamination,
  tampered/garbage tokens) and for `parse_token`.

#### Changed

- `Authentication::get_user_id_from_bearer_token` takes
  `BearerToken` directly instead of `Option<BearerToken>`.
- `parse_token` returns `Result` instead of panicking on malformed
  headers.
- JWT `exp` validation relies solely on the `jsonwebtoken` library;
  the redundant manual expiration check is removed.
- Token signing uses `Result` propagation instead of `.unwrap()` /
  `.expect()`.
- `UserClaims` is now a type alias for `SessionClaims`.
- `VerifyClaims` moved from `mailer` into the `jwt` module
  (re-exported for backward compatibility).
- JWT session token `role` claim carries the database `role`
  directly (`"registered"`, `"admin"`) instead of the previous
  mapping (`"user"`, `"admin"`).

#### Removed

- `bearer_token::Extract` wrapper struct (replaced by `BearerToken`
  directly).
- `get_optional_logged_in_user` free function (logic moved into
  extractors).
- `get_claims_from_bearer_token` private method on `Authentication`
  (inlined).
- `ClaimTokenPepper` / `JwtSigningSecret` /
  `user_claim_token_pepper` config keys.

### ADR-T-006 — Error system

#### Added

- ADR-T-006 itself.
- ~190 crate-level tests for the domain error system
  (`src/tests/errors/`): status-code mapping, display messages,
  `From` impl coverage, and `ApiError` delegation.

#### Changed

- Service functions return domain-specific
  `Result<T, DomainError>` instead of `Result<T, ServiceError>`.
- Each domain error co-locates its HTTP status-code mapping via a
  `status_code()` method.
- Error `From` impls use `tracing::error!` instead of `eprintln!`.
- Standardise all error derives on `thiserror`.

#### Removed

- `ServiceError` enum and `ServiceResult` type alias from
  `src/errors.rs`.
- `http_status_code_for_service_error` and
  `map_database_error_to_service_error` helpers.
- `IntoResponse` impl for `database::Error` (now handled by domain
  errors).

### Dependencies

- Every dependency of the workspace refreshed to its newest release, majors included. The lock moved to the newest versions the existing requirements admitted, and six requirements crossed a major: `bittorrent-primitives` 0.2 is replaced by `torrust-info-hash` 0.2, the crate its own 0.3 release deprecates the whole of itself in favour of; `jsonwebtoken` 10 to 11; `tower-http` 0.6 to 0.7; `argon2` 0.5 to 0.6 together with `pbkdf2` 0.12 to 0.13, which share a `password-hash` release and cannot move apart; and `tera` 1 to 2. No user-visible behaviour moves: the HTTP API, its responses, the JWT format, the email template language and the on-disk and on-wire form of infohashes are all unchanged. Stored password hashes keep verifying — the PHC string carries the algorithm, version, parameters and salt each hash was written with, so credentials created under the previous releases are read back with their own parameters and remain valid; this is not a breaking change for deployments. `sqlx` stays at 0.8 because 0.9 requires Rust 1.94, above this project's 1.89 floor.
- Requirements of the bare form `0`, which admit every breaking pre-1.0 release, are narrowed to the `0.x` line the sources are written against, so the manifests now declare what the code actually depends on.

### Security

- Dev-only ports (MySQL 3306, tracker 6969/7070/1212, mailcatcher
  1025/1080) no longer bind to `0.0.0.0`; bound to `127.0.0.1`.
- Entry script debug mode now emits structured JSON phase records instead of
  enabling `set -x`, avoiding shell-trace leakage of env vars into logs.
- Compose credentials annotated as DEV-ONLY with TODO for Docker
  secrets migration (ADR-T-009 §S1).

### Fixed

- MySQL compose healthcheck was referencing a non-existent Docker
  secret (`/run/secrets/db-password`); now uses
  `$$MYSQL_ROOT_PASSWORD`.
- Entry script `USER_ID` validation:
  `-z "$USER_ID" && "$USER_ID" -lt 1000` always short-circuited to
  an error when `USER_ID` was unset; corrected to `||`.
- Containerfile release `HEALTHCHECK` trailing whitespace removed.

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
