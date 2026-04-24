# ADR-T-009 — Implementation Plan

**Companion to:** [adr/009-container-infrastructure-refactor.md](009-container-infrastructure-refactor.md)
**Status:** Tracking
**Date:** 2026-04-19

This document captures the *how* of ADR-T-009. The ADR records
the decisions (D1–D9); this plan records the phases, file
lists, snippets, dependency ordering, and merge-conflict notes
needed to land them. Decisions and rationale are not
re-litigated here — when in doubt, defer to the ADR.

## Phase Status

| Phase | Title                              | Status      |
|-------|------------------------------------|-------------|
| 1     | Build hygiene                      | Done        |
| 2     | Helper binaries (D5)               | Done        |
| 3     | Extract `index-config` crate       | Done        |
| 4     | Runtime base split (D4, D7)        | Done        |
| 5     | Schema (D2)                        | Not started |
| 6     | Config probe                       | Not started |
| 7     | Entry-script contract              | Not started |
| 8     | Compose split                      | Not started |
| 9     | Documentation & audit (D8, D9)     | Not started |

## Phase Dependency Graph

```
Phase 1  (build hygiene) ──────────┐
Phase 2  (helpers: D5) ────────────┤  [Phases 1, 2, 4 are
Phase 4  (runtime base split) ─────┤   mutually independent]
                                   │
Phase 3  (extract index-config) ───┐
                                   │
Phase 5  (schema: D2) ─────────────┤  depends on: Phase 3
                                   │  (Phase 5 edits files
                                   │   Phase 3 moves)
                                   ▼
                            Phase 6 (config probe)
                            depends on: Phase 3, Phase 5
                                   │
                                   ▼
                            Phase 7 (entry-script contract)
                            depends on: Phase 2, Phase 4, Phase 5, Phase 6
                                   │
                                   ▼
                            Phase 8 (compose split)
                            depends on: Phase 7
                                   │
                                   ▼
                            Phase 9 (docs & audit)
                            depends on: all above
```

**Legend.** Phases 1, 2, and 4 are mutually independent and
can land in any order or in parallel. Phase 3 must precede
Phase 5: Phase 5 edits `src/config/v2/database.rs`,
`src/config/v2/mod.rs`, and `src/config/v2/tracker.rs`,
which Phase 3 *moves* to `packages/index-config/`. If Phase
5 lands first, its edits go to the soon-to-be-deleted paths
and Phase 3 must re-apply them at the new locations — extra
work and a real risk of losing changes in the move. Sequence
Phase 1 before Phase 4 when possible to minimise textual
conflicts in the Containerfile (both touch it, but in
different sections). Phase 6 (the config-resolution helper)
needs Phase 3 to depend on the extracted
`torrust-index-config` crate without dragging in the root
crate's dep graph, and needs Phase 5 so the
mandatory-`connect_url` schema change is reflected in what the
helper sees. Phase 7 consumes the helper from Phase 6, the
runtime base from Phase 4, and the entry-script rename from
Phase 2.

**Merge-conflict notes.**

- Phase 2 renames `health_check` → `torrust-index-health-check`
  and `torrust-generate-auth-keypair` →
  `torrust-index-auth-keypair` in the `Containerfile`'s
  `cp -l ...` lines and the entry script's keygen invocation
  (§2.2 step 4 touches `share/container/entry_script_sh`,
  migrating the keygen consumer from `sed` to `jq`), and
  introduces the `jq_donor` stage in the `Containerfile`
  so both runtime bases gain `/usr/bin/jq`;
  Phase 6 adds a parallel `torrust-index-config-probe` line;
  Phase 4 restructures the same files. Expect textual
  conflicts in both `Containerfile` and `entry_script_sh` if
  Phases 2 and 4 land in parallel — Phase 4's `runtime_release`
  / `runtime_debug` stages each need a `COPY --from=jq_donor`
  line that Phase 2 owns.
- Phase 3 moves files under `src/config/` to
  `packages/index-config/`. Phase 5 edits the same files
  (specifically `src/config/v2/database.rs`,
  `src/config/v2/mod.rs`, and `src/config/v2/tracker.rs`).
  This is the hard ordering constraint captured in the graph
  above — Phase 3 must land first.
- Phase 7 modifies the entry script Phase 4 also rewrites;
  Phase 7 is sequenced after Phase 4 to avoid that conflict.
  Phase 2's keygen-invocation rename (§2.2 step 4) also
  touches the entry script; Phase 7 depends on Phase 2
  explicitly (see the graph above), so no parallel conflict
  arises as long as the dependency graph is respected.

---

## Phase 1 — Build Hygiene (D6, D9 build-context part)

**Status:** Landed (2026-04-24).
**Files.** `.containerignore`, `Containerfile` (ARG / ENV /
EXPOSE block only), `docs/containers.md` (operator-visible
note), `CHANGELOG.md`.

1. **Drop build-time port `ARG`s.** Remove `ARG API_PORT` /
   `ARG IMPORTER_API_PORT`. Keep `ENV API_PORT=3001` /
   `ENV IMPORTER_API_PORT=3002` so `HEALTHCHECK` and the
   listener resolve at runtime.

   Document in `docs/containers.md` (see also
   [Phase 9](#phase-9--documentation--audit-d8-d9-docs-part))
   that `EXPOSE` freezes the default port into image metadata
   at build time; runtime overrides reach the listener and
   the healthcheck but not `docker inspect`.

2. **Tighten `.containerignore`.** Add `adr/` and `docs/`
   only. Verify with `docker build --no-cache` that nothing
   required is excluded.

   *Landed:* `/adr/` and `/docs/` added; the file's missing
   trailing newline was also fixed in the same change. The
   `--no-cache` verification is deferred to the next
   end-to-end image build (no container engine is available
   in the agent's sandbox); the explicit "do not exclude"
   list below was honoured.

   **Do not** exclude `packages/render-text-as-image/` — it is
   a workspace member and a path dependency of the root crate;
   removing it from the build context breaks
   `cargo chef --workspace` and the `torrust-index` build
   itself. (See [Carry-Over](009-container-infrastructure-refactor.md#carry-over-items)
   in the ADR for the long-term plan to promote it to a
   published crate, after which it can safely be excluded.)

   **Do not** exclude `tests/fixtures/` or
   `migrations/<other-driver>/` without a full CI matrix run
   (SQLite + MySQL) confirming no test references them.
   Specifically, `tests/e2e/` and `src/databases/` contain
   the paths that read migration and fixture files at test
   time. The in-image `test` / `test_debug` stages run the
   workspace suite via
   `nextest --workspace-remap /test/src/`; missing fixtures
   fail silently and (per Phase 4 below) block image
   production.

---

## Phase 2 — Health-check & Auth-keypair Helpers (D5)

**Status:** Landed (2026-04-24).
**Files.** New `packages/index-cli-common/` library crate;
new `packages/index-health-check/` crate (`Cargo.toml`,
`src/lib.rs`, `src/bin/torrust-index-health-check.rs`,
`src/tests/`, `tests/health_check.rs`); new
`packages/index-auth-keypair/` crate (`Cargo.toml`,
`src/lib.rs`, `src/bin/torrust-index-auth-keypair.rs`,
`src/tests/`, `tests/keypair_generation.rs`); root
`Cargo.toml` (workspace members, remove `[[bin]]` entry);
`Containerfile` (`cp -l .../release/torrust-index-health-check`
and `.../torrust-index-auth-keypair` lines, renamed from the
former `health_check` and `torrust-generate-auth-keypair`).

*Landed:* both helpers were extracted into separate workspace
crates with a shared `torrust-index-cli-common` scaffolding
crate. Each helper carries `src/lib.rs` (domain logic) and
`src/bin/<binary>.rs` (the `main` shim) so that the public
surface is reachable from integration tests in `tests/` —
a deliberate refinement of the plan's `src/main.rs` layout
that keeps the produced binary names unchanged. Both old
`src/bin/*.rs` files were removed; the old `[[bin]]` entry
for `torrust-generate-auth-keypair` was deleted from the
root `Cargo.toml`. The `jq_donor` stage was added to the
`Containerfile` and its binary copied into the (single,
pre-Phase-4) `runtime` stage with `--chmod=0500 --chown=0:0`;
when Phase 4 splits the runtime base, the `COPY --from=jq_donor`
line must be re-added to both `runtime_release` and
`runtime_debug`. The entry script's keygen consumer was
migrated from `sed` PEM-block extraction to `jq -r
.private_key_pem` / `jq -r .public_key_pem` in the same
change. `cargo tree -e normal` confirms neither helper crate
links `reqwest`, `tokio`, `hyper`, `rustls`, `native-tls`,
or `openssl`. About twenty crate-level and integration tests
in the new packages all pass.

The third small helper (`torrust-index-config-probe`) lands
in Phase 6 once Phase 3 has extracted the config crate it
depends on. It is no longer co-located with the health-check
helper because its dep closure is materially different.

### 2.0 Shared CLI scaffolding (`index-cli-common`)

Create `packages/index-cli-common/` as library crate
`torrust-index-cli-common`. This crate provides the P9
universal baseline scaffolding so each helper binary's
`main` is only domain logic.

**Public API:**

```rust
/// Refuse to run if stdout is a terminal (P8).
/// Prints a diagnostic to stderr and exits with code 2.
pub fn refuse_if_stdout_is_tty(binary_name: &str);

/// Initialise `tracing-subscriber` with JSON output on stderr.
pub fn init_json_tracing(level: tracing::Level);

/// Serialise `value` as one JSON object + trailing newline to stdout.
pub fn emit<T: serde::Serialize>(value: &T) -> std::io::Result<()>;

/// Common `--debug` flag for all helpers. Flatten into each
/// binary's `Args` struct via `#[command(flatten)]`.
#[derive(clap::Args)]
pub struct BaseArgs {
    /// Enable debug-level logging on stderr.
    #[arg(long)]
    pub debug: bool,
}
```

**Dependencies.** The P9 baseline and nothing else: `clap`,
`tracing`, `tracing-subscriber` (with `json` feature),
`serde`, `serde_json`.

Every binary's `main` reduces to:

```rust
fn main() -> std::process::ExitCode {
    let args = Args::parse();
    refuse_if_stdout_is_tty("torrust-index-<name>");
    init_json_tracing(if args.base.debug { Level::DEBUG } else { Level::INFO });
    match run(&args) {
        Ok(out) => { emit(&out).unwrap(); ExitCode::SUCCESS }
        Err(e)  => { error!(error = %e, "…"); ExitCode::from(e.exit_code()) }
    }
}
```

### 2.1 Health-check rewrite

1. **Extract into a new workspace crate.** Create
   `packages/index-health-check/` as crate
   `torrust-index-health-check`. Move
   `src/bin/health_check.rs` to `src/main.rs`. The produced
   binary is named after the crate
   (`torrust-index-health-check`) — no `[[bin]]` override is
   added, and every reference in the `Containerfile` and
   `docs/containers.md` is updated from the old `health_check`
   name in the same change. The `index-` prefix is deliberate
   — a tracker counterpart also ships a health binary, and
   prefix-by-product avoids future ambiguity.

2. **Rewrite stdlib-only networking.** Replace
   `reqwest`/`tokio`/TLS with `std::net::TcpStream` + minimal
   HTTP/1.1 GET (~30 lines). Bind `set_read_timeout` /
   `set_write_timeout` for a short connect/read window. No
   async runtime.

3. **JSON stdout (P9).** Emit a single JSON object on
   success (exit 0):
   ```json
   {"target": "http://localhost:3001/health_check", "status": 200, "elapsed_ms": 4}
   ```
   On failure (exit ≠ 0), stdout is empty — the exit code is
   the sole branch signal for callers (Docker, the entry
   script), and the diagnostic goes to stderr via `tracing`.
   This matches the P9 convention: success emits JSON on
   stdout; failure emits nothing on stdout and logs to stderr.
   The P9 scaffolding from `index-cli-common` handles TTY
   refusal, tracing init, and `emit()`. Docker only looks at
   the exit code; the JSON is useful when an operator invokes
   the binary manually for diagnosis.

4. **Test failure paths.** Cover non-2xx response, connection
   refused, read timeout, and malformed status line — those
   are the actual healthcheck signals. Use a `TcpListener` on
   an ephemeral port.

### 2.2 Auth-keypair generator extraction

`torrust-generate-auth-keypair` is currently a `[[bin]]` in
the root `torrust-index` crate
([`src/bin/generate_auth_keypair.rs`](../src/bin/generate_auth_keypair.rs)).
Its source uses only `rsa`, `pkcs8`, `clap`, `tracing`, and
`tracing-subscriber` — but because it lives in the root
package, its link graph inherits the full
`tokio`/`hyper`/`rustls`/`reqwest` closure. It also runs as
root during first boot and handles private key material, so
the attack-surface argument is at least as strong as for the
health-check binary.

1. **Extract into a new workspace crate.** Create
   `packages/index-auth-keypair/` as crate
   `torrust-index-auth-keypair`. Move
   `src/bin/generate_auth_keypair.rs` to `src/main.rs`.
   Remove the `[[bin]]` entry from the root `Cargo.toml`.
   The produced binary is named after the crate
   (`torrust-index-auth-keypair`); every reference in the
   `Containerfile`, entry script, and `docs/containers.md`
   is updated from the old `torrust-generate-auth-keypair`
   name in the same change.

2. **Manifest-level dep enforcement.** The crate's domain dep
   is `rsa` (which re-exports `pkcs8`). The P9 universal
   baseline (`clap`, `tracing`, `tracing-subscriber`, `serde`,
   `serde_json`) comes via `torrust-index-cli-common`.
   Specifically `tokio`, `reqwest`, and every TLS crate are
   absent by manifest.

3. **JSON stdout (P9).** The output changes from raw PEM
   blocks to a single JSON object:
   ```json
   {"private_key_pem": "-----BEGIN PRIVATE KEY-----\n...", "public_key_pem": "-----BEGIN PUBLIC KEY-----\n..."}
   ```
   This eliminates the `sed` post-processing in the current
   documented usage. Consumers use `jq -r .private_key_pem`
   (shell) or `serde_json::from_reader::<KeypairOutput>`
   (Rust). The existing TTY guard migrates to the shared
   `refuse_if_stdout_is_tty` from `index-cli-common`,
   unifying on exit code 2 (was exit 1).

4. **Update entry script and introduce `jq` to the runtime
   image.** The keygen invocation in
   `share/container/entry_script_sh` changes from
   `torrust-generate-auth-keypair` to
   `torrust-index-auth-keypair`, and the consumer migrates
   from `sed` PEM-block extraction to `jq -r
   .private_key_pem` / `jq -r .public_key_pem` in the same
   change. `sed` cannot recover usable PEM from the new
   single-line JSON output (the PEM body's newlines are
   JSON-escaped as the two literal characters `\` and `n`,
   and the `-----END` marker is no longer on its own
   line), so producer and consumer must flip together.

   This requires `jq` in the runtime image starting in
   Phase 2. Add a dedicated `jq_donor` stage to the
   `Containerfile` that installs `jq` from a pristine
   `rust:slim-trixie` base *before any user code runs*,
   then have both runtime bases copy the binary from that
   stage:

   ```dockerfile
   ## jq donor (pristine base, no user code)
   FROM rust:slim-trixie AS jq_donor
   RUN apt-get update && \
       apt-get install -y --no-install-recommends jq && \
       rm -rf /var/lib/apt/lists/*

   # In each runtime stage:
   COPY --from=jq_donor --chmod=0500 --chown=0:0 /usr/bin/jq /usr/bin/jq
   ```

   Using the same `rust:slim-trixie` base as the existing
   `tester` stage guarantees the binary matches the
   runtime's architecture. The separate stage ensures the
   `jq` binary is sourced from a clean Debian install — no
   project files, `cargo binstall` artefacts, or
   third-party tool installations have touched the image
   at that point. `jq` is invoked only during the entry
   script's root-phase (before `su-exec` drops privileges);
   like busybox and `su-exec`, it is mode `0500 root:root`
   so the unprivileged `torrust` user gets `EACCES` after
   privilege drop. It is not a busybox applet so it does
   not appear in §4.4's curated symlink loop.

   The binary location for `torrust-index-auth-keypair`
   in the image remains `/usr/bin/` (same as the other
   helpers).

5. **Tests.** The existing tests in
   `src/bin/generate_auth_keypair.rs` move with the code.
   Add a test that the generated JSON output round-trips
   through `serde_json` and the PEM blocks are parseable
   by `rsa::RsaPrivateKey::from_pkcs8_pem` /
   `rsa::RsaPublicKey::from_public_key_pem` — the
   round-trip the application performs at startup.

---

## Phase 3 — Extract `index-config` Crate (foundation for Phase 6)

**Status:** Landed (2026-04-24).
**Files.** New `packages/index-config/` crate (Cargo.toml +
the moved sources); root `Cargo.toml` (workspace members and
new path dependency); root `src/config/mod.rs` (shrinks to
re-exports + the runtime `Configuration` wrapper); the small
set of inward dependencies catalogued below.

*Landed:* the parsing surface of `src/config/` was moved
verbatim into the new `torrust-index-config` workspace crate
under `packages/index-config/` (schema modules, validator,
`load_settings`, `Info`, `Error`, the `CONFIG_OVERRIDE_*`
and `ENV_VAR_CONFIG_TOML*` constants, plus a `pub type
DynError` alias to break the inward dependency on the web
layer). The runtime `Configuration` wrapper holding
`tokio::sync::RwLock<Settings>` and its `async` accessors
(`get_all`, `get_site_name`, `get_api_base_url`) stayed in
the root crate and now sits beneath a `pub use
torrust_index_config::*;` re-export shim in
[`src/config/mod.rs`](../src/config/mod.rs), so every
existing `use crate::config::Settings;` (and similar) call
site continues to compile unchanged. The permission *value*
types (`Role`, `Action`, `Effect`, `PermissionOverride`,
`RoleParseError`) moved into
[`packages/index-config/src/permissions.rs`](../packages/index-config/src/permissions.rs)
and are re-exported from `crate::services::authorization`
for backwards compatibility; the `Permissions` *trait* and
`PermissionMatrix` runtime policy stayed in the root crate.
The `Tsl` → `Tls` clean-break rename was performed in the
same change — type, field, serde wire key, local variables,
shipped TOML defaults, and the JSON example in
`src/web/api/server/v1/contexts/settings/mod.rs` were all
updated together; `grep -rE 'Tsl|\.tsl' src/ share/
packages/` returns zero hits. Crate-level tests live in
[`packages/index-config/src/tests/`](../packages/index-config/src/tests/mod.rs)
and the public-API integration tests
(`permission_overrides`, `round_trip`, `shipped_samples`)
live in
[`packages/index-config/tests/`](../packages/index-config/tests/round_trip.rs);
all ~forty tests in the new crate pass alongside the
existing workspace suite, and `cargo tree -p
torrust-index-config -e normal` confirms `tokio`, `reqwest`,
`sqlx`, `hyper`, `rustls`, `native-tls`, and `openssl` are
all absent from the dep closure.

The dep list in §3 below was amended in the same change to
include `serde_json` and `lettre` — see the paragraph after
this section's intro for the rationale. The §3.5 acceptance
"forbidden crates" exclusion check still holds against the
amended set.

The `torrust-index-config-probe` helper introduced in Phase 6
must call the same `figment` + `serde` parser the application
uses, otherwise it reintroduces the disagreement Phase 7
exists to eliminate. The application's parser today is
entangled with the root crate's runtime types (`tokio`,
`reqwest`-adjacent error plumbing); a probe binary that
depends on the root crate would inherit that closure.

This phase extracts the *parsing* surface of `src/config/`
into a small workspace crate `torrust-index-config` whose
non-stdlib deps are `serde`, `serde_json`, `serde_with`,
`figment`, `toml`, `url`, `camino`, `derive_more`,
`thiserror`, `tracing`, and `lettre` — the things you need
to deserialise the schema and not one crate more.
Specifically `tokio`, `reqwest`, `sqlx`, and every TLS
crate are absent by manifest.

`lettre` enters the dep set because the schema parses
`smtp.from` / `smtp.reply_to` directly into
`lettre::message::Mailbox` (see
`packages/index-config/src/v2/mail.rs`); it is pulled in
with `default-features = false` and only the `builder` and
`serde` features so none of its async / TLS / transport
machinery leaks in. `serde_json` is used by
`Settings::to_json` (the redaction-friendly debug rendering
in `packages/index-config/src/v2/mod.rs`) and by the crate's
own tests. Both are confirmed absent from the forbidden list
by the `cargo tree` acceptance check in §3.5.

### 3.1 What moves

In scope for the extraction:

- All of `src/config/v2/` (the schema modules).
- `src/config/validator.rs`.
- From `src/config/mod.rs`: the `Settings` / `Info` /
  `Metadata` / `Version` / `Tsl` / `Error` types,
  `load_settings`, `check_mandatory_options`, the
  `CONFIG_OVERRIDE_*` constants, and the `ENV_VAR_CONFIG_TOML*`
  constants.

  (Note: `Tsl` is a typo for `Tls` in the current source.
  **Clean break: rename both the type and the field to
  `Tls` / `tls`** as part of the extraction, with no
  backwards-compatibility alias or `serde(rename)` shim.

  The rename touches every occurrence in one change:

  - **Type:** `Tsl` → `Tls` (definition in
    `src/config/mod.rs`, imports in `src/config/v2/net.rs`,
    `src/web/api/mod.rs`, `src/web/api/server/mod.rs`).
  - **Field:** `Network::tsl` → `Network::tls` (definition
    in `src/config/v2/net.rs`, call site
    `settings.net.tsl.clone()` in `src/app.rs`).
  - **Serde wire key:** TOML `[net.tsl]` → `[net.tls]` and
    JSON `"tsl"` → `"tls"` (driven by the field name;
    no `serde(rename)` needed).
  - **Local variables:** `opt_net_tsl` → `opt_net_tls` in
    `src/app.rs`; `opt_tsl` → `opt_tls` in
    `src/web/api/mod.rs` and `src/web/api/server/mod.rs`;
    `tsl_config` / `tsl` → `tls_config` / `tls` in
    `src/web/api/server/mod.rs`.
  - **Shipped defaults:** the commented-out `#[net.tsl]`
    block in
    `share/default/config/index.development.sqlite3.toml`
    becomes `#[net.tls]`; the "TSL" comment above it
    becomes "TLS".
  - **Doc-comments and code comments:** "TSL" → "TLS"
    in `src/config/v2/net.rs`, `src/web/api/server/mod.rs`,
    and `src/web/api/server/v1/contexts/settings/mod.rs`
    (JSON example `"tsl": null` → `"tls": null`).
  - **Internal helpers:** `Network::default_tsl()` →
    `Network::default_tls()`.

  **Impact on existing consumers.** This is a breaking
  change for any operator TOML that uses `[net.tsl]` and
  for any API consumer that reads the `"tsl"` JSON key.
  Both must update to `tls`. The typo has been present
  since the schema's introduction; correcting it now
  (alongside the other schema-breaking changes in D2) is
  cheaper than carrying it indefinitely.

  The grep-verified call-site count is around twenty Rust
  sites plus one TOML and one doc-comment JSON example.
  All are updated in the same commit.)

Out of scope (stays in the root crate):

- The `Configuration` wrapper struct holding
  `tokio::sync::RwLock<Settings>` and its `async` accessors
  (`get_all`, `get_site_name`, `get_api_base_url`). This is
  application *runtime state* over a parsed `Settings`, not
  parsing. It moves to `src/config_state.rs` (or stays in
  `src/config/mod.rs` alongside the re-exports) and depends
  on the new crate.

### 3.2 Inward dependencies to resolve

A grep of `^use crate::|^use super::` across `src/config/`
turns up three boundary-crossing imports:

1. **`src/config/mod.rs` → `crate::web::api::server::DynError`.**
   Used inside `Error::UnableToLoadFromConfigFile { source: DynError }`.
   `DynError` is a project-wide alias for
   `Arc<dyn std::error::Error + Send + Sync>`. The fix is to
   define a `pub type DynError = ...` alias inside the new
   crate (it is one line) so the config crate has no
   dependency on the web layer. The alias must be `pub`
   because the `Error` enum variants that expose it are
   themselves `pub`; the root crate's re-export shim
   (§3.3) re-exports it alongside everything else.

2. **`src/config/v2/permissions.rs` → `crate::services::authorization::PermissionOverride`.**
   `PermissionOverride` is a value type (`#[derive(Deserialize)]`
   struct), not a service. Its fields reference three sibling
   types — `Role`, `Action`, and `Effect` — which are also
   pure serde enums with no service-layer dependencies; all
   four move together. These types were introduced in
   [ADR-T-008](008-roles-and-permissions-refactor.md); the
   move does not alter their semantics or their API — only
   their crate home. The fix is to move
   `PermissionOverride`, `Role`, `Action`, and `Effect` (and
   only those types, not the surrounding authorization
   service) into the new crate under
   `permissions::{PermissionOverride, Role, Action, Effect}`,
   and re-export them
   from `src/services/authorization/` so existing call sites
   continue to compile unchanged. If this turns out to drag
   in further types, fall back to defining the struct twice
   with a `From` conversion at the boundary — measure first,
   duplicate only if needed.

3. **`src/config/v2/net.rs` → `crate::config::Tsl`.**
   `Tsl` lives in `src/config/mod.rs` today and moves with
   the rest of the schema (type renamed to `Tls`, field
   renamed to `tls`, per the clean-break note in §3.1).
   After extraction this becomes a sibling-module import
   inside the new crate; no cross-crate work needed.

### 3.3 Compatibility shim in the root crate

`src/config/mod.rs` becomes a thin re-export module so every
existing `use crate::config::Settings;` (and similar) keeps
compiling without churn:

```rust
pub use torrust_index_config::*;
// Plus the Configuration wrapper and any application-only
// helpers that were never part of the parsing surface.
```

No `Tsl` compatibility alias is needed — the clean-break
rename (§3.1) updates every call site in the same commit.
Verify with `cargo check --workspace --all-features` after
the move.

### 3.4 Tests move with the code

Crate-level tests under `src/config/tests/` (if any exist
today) move into `packages/index-config/src/tests/`.
Doc-tests on the moved items move with the items. The root
crate's integration tests under `tests/` continue to use
`use torrust_index::config::...;` re-exports unchanged.

### 3.5 Acceptance

- `cargo check -p torrust-index-config` passes without
  depending on the root crate `torrust-index` (no circular
  dependency; the config crate is a leaf).
- `cargo tree -p torrust-index-config -e normal --prefix none`
  excludes `tokio`, `reqwest`, `sqlx`, `hyper`, `rustls`,
  `native-tls`, and `openssl`.
- `cargo check --workspace --all-features` and
  `cargo nextest run --workspace --all-features` pass.
  Call sites outside `src/config/` are updated only for the
  `Tsl` → `Tls` / `tsl` → `tls` clean-break rename (§3.1);
  no other import changes are required (re-export shim is
  doing its job for everything else).
- `grep -rE 'Tsl|\.tsl' src/ share/` returns zero hits
  (clean break verified).

---

## Phase 4 — Runtime Base Split (D4, D7)

**Status:** Landed (2026-04-24).
**Files.** `Containerfile` (multi-stage restructure),
`share/container/entry_script_sh` (`adduser` line, `USER_ID`
guard).

The largest phase and the one the security story hangs on.

*Landed:* the multi-stage restructure went in as designed —
`busybox_donor` / `busybox_preflight` / `etc_seed` /
`adduser_preflight` / `runtime_assets` / `preflight_gate` are
all present, both runtime bases (`runtime_release` and
`runtime_debug`) layer onto them, and the per-binary
`0500 root:root` tightening for the helper binaries was
applied in the producing `test` / `test_debug` stages so
the runtime stages pick the bits up unchanged via
`COPY --from=test /app/ /usr/`. A handful of practical
deviations from the §4.2 sketch worth recording:

- **usrmerge in the release base.** `gcr.io/distroless/cc-debian13`
  ships `/bin` as a symlink to `/usr/bin`, so a recursive
  `COPY --from=runtime_assets / /` fails with
  *"cannot copy to non-directory"*. The release base
  therefore copies each curated path explicitly (the
  `etc_seed` triplet, busybox, su-exec, entry script) and
  materialises the curated symlinks in `/usr/bin/<applet>`
  rather than `/bin/<applet>`. Resolution at runtime is
  identical because the base's `/bin → /usr/bin` symlink
  forwards bare-name lookups, and the pinned `PATH` includes
  both directories.
- **`adduser_preflight` UID.** The original sketch used UID
  `65534` (the conventional `nobody`); busybox `adduser`
  rejects UIDs outside `0..60000` with
  *"number 65534 is not in 0..60000 range"*. Lowered to
  `59999` (well above any realistic `USER_ID=1000` and still
  well below the cap) and the diff in [Containerfile](Containerfile)
  matches the sketch above.
- **`addgroup` and `grep` added to the curated symlink loop.**
  See §4.4 for rationale; the table above was updated in
  the same change so the three sources of truth (loop, table,
  [docs/containers.md](docs/containers.md)) all agree.
- **Debug image keeps `HEALTHCHECK`.** The pre-Phase-4
  `runtime` stage shipped no `HEALTHCHECK` for the debug
  variant. Phase 4 brings the debug final target in line with
  release: the same two-probe `HEALTHCHECK` block now runs on
  both, and the debug `test_debug` stage was extended to
  ship `torrust-index-health-check` alongside `torrust-index`
  and `torrust-index-auth-keypair` (also tightened to
  `0500 root:root`). The default `CMD` was set to
  `["/usr/bin/torrust-index"]` so the debug image is a
  drop-in replacement for release; operators reach an
  interactive shell with `docker run … sh` (or any other
  curated applet) at run time.
- **Entry script.** `mkdir -p` was extended to include
  `/var/log/torrust/index/` because the new runtime stages
  no longer pre-create the volume sub-directories the way
  the old monolithic `runtime` stage did. Everything else in
  §4.1 (busybox-`adduser` short options, numeric + `!= 0`
  `USER_ID` guard) landed verbatim.

The §4.5 acceptance script (full image build for both
targets) is still pending — no container engine is available
in the agent's sandbox. Run
`podman build --target release .` and
`podman build --target debug .` before publishing.

### 4.1 Entry-script changes

1. **`adduser` to busybox short-option form.**
   ```sh
   # before
   adduser --disabled-password --shell "/bin/sh" --uid "$USER_ID" "torrust"
   # after
   adduser -D -s /bin/sh -u "$USER_ID" torrust
   ```
   Works uniformly against busybox `adduser` on both runtime
   bases. Distroless `cc-debian13` ships `/etc/passwd` and
   `/etc/group` but not `/etc/shadow`; `-D` honours that.
   The `adduser_preflight` build stage (§4.2) exercises this
   exact invocation against the shadow-less `etc_seed` layout
   so a future busybox bump surfaces the failure at build
   time, not first boot.
   Busybox `adduser -s` does not consult `/etc/shells`, which
   the lean base also lacks.

2. **`USER_ID` guard becomes "is numeric" + `-eq 0` (D7).**
   ```sh
   case ${USER_ID:-} in
       ''|*[!0-9]*)
           echo "ERROR: USER_ID is unset or not numeric" >&2
           exit 1
           ;;
   esac
   if [ "$USER_ID" -eq 0 ]; then
       echo "ERROR: USER_ID is 0 (root) — refusing to run as root" >&2
       exit 1
   fi
   ```

### 4.2 Containerfile restructure

Factor the shared runtime ingredients into a base-agnostic
`FROM scratch` stage, then layer onto two parallel runtime
bases.

```dockerfile
## ── Runtime asset bundle (base-agnostic) ─────────────────────
FROM gcr.io/distroless/cc-debian13:debug AS busybox_donor

# Preflight: assert the donor still ships a real busybox ELF
# at the path we copy from, and that its `install` applet
# supports `-D` (the entry script's `inst()` helper depends on
# it). Cheap insurance against a future base reshuffle.
FROM busybox_donor AS busybox_preflight
RUN ["/busybox/sh", "-c", \
     "test -f /busybox/busybox \
      && /busybox/busybox --help >/dev/null \
      && /busybox/busybox install --help 2>&1 | grep -q -- '-D'"]

# Minimal /etc account files generated locally so adduser
# works regardless of base. The base image's own
# /etc/nsswitch.conf (which includes `hosts: files dns`) is
# deliberately left alone — the application makes outbound
# connections where host-name resolution must work.
FROM busybox_preflight AS etc_seed
# /etc/profile is seeded as an empty file so the debug image's
# `ENV ENV=/etc/profile` points at a real path. Operators who
# bind-mount a richer profile over it get the expected
# behaviour; absent that, busybox `sh` sources an empty file
# and continues silently rather than warning on every invocation.
RUN ["/busybox/sh", "-c", \
     "mkdir -p /seed/etc && \
      printf 'root:x:0:0:root:/:/bin/sh\\n'    > /seed/etc/passwd && \
      printf 'root:x:0:\\n'                    > /seed/etc/group  && \
      : > /seed/etc/profile"]

# Preflight: assert `adduser -D` works without /etc/shadow.
# The entry script runs `adduser -D -s /bin/sh -u $USER_ID
# torrust` at first boot against the etc_seed layout (passwd +
# group, no shadow). Busybox adduser behaviour when shadow is
# absent varies by version; this stage catches regressions at
# build time rather than first boot.
# Scope: this preflight asserts the *busybox adduser applet*
# works against the shadow-less etc_seed layout. It does NOT
# exercise the entry script's quoting or variable
# interpolation (`-u "$USER_ID"`) — that coverage comes
# from the integration tests against entry_script_sh (§7.1).
FROM busybox_donor AS adduser_preflight
COPY --from=etc_seed /seed/etc/passwd /etc/passwd
COPY --from=etc_seed /seed/etc/group  /etc/group
# Busybox `adduser` rejects UIDs outside 0..60000 (below the
# conventional `nobody` value of 65534). Use a value in range
# that is still comfortably above `USER_ID=1000`.
RUN ["/busybox/sh", "-c", \
     "/busybox/adduser -D -s /bin/sh -u 59999 testuser \
      && /busybox/grep -q '^testuser:' /etc/passwd \
      && /busybox/test -d /home/testuser"]

FROM scratch AS runtime_assets
COPY --from=etc_seed --chmod=0644 --chown=0:0 /seed/etc/ /etc/
# Single busybox binary, root-only. Copy the file directly
# (not via the /busybox/sh symlink) to avoid depending on the
# donor's symlink layout.
COPY --from=busybox_preflight --chmod=0700 --chown=0:0 \
    /busybox/busybox /bin/busybox
# `gcc` is the existing build stage in the current
# Containerfile that compiles vendored su-exec.
COPY --from=gcc --chmod=0700 --chown=0:0 \
    /usr/local/bin/su-exec  /bin/su-exec
COPY --chmod=0555 --chown=0:0 \
    ./share/container/entry_script_sh  /usr/local/bin/entry.sh

## ── Preflight gate (aggregates all donor-validation stages) ──
# Both runtime bases COPY from this stage, creating an
# explicit BuildKit dependency edge that prevents any
# preflight from being pruned regardless of which image
# variant is built.  The donor busybox is the same binary
# both images use (release extracts it; debug inherits the
# tree), so both preflights are relevant to both bases.
#
# `FROM scratch` has no shell and no executables, so a RUN
# cannot work here.  The COPYs are the dependency mechanism;
# the sentinel files they produce are cleaned up in each
# runtime base's own RUN layer.
FROM scratch AS preflight_gate
COPY --from=busybox_preflight /etc/passwd /tmp/.busybox-ok
COPY --from=adduser_preflight /etc/passwd /tmp/.adduser-ok

## ── Runtime base: release (root-only curated subset) ─────────
FROM gcr.io/distroless/cc-debian13 AS runtime_release
COPY --from=runtime_assets / /
COPY --from=preflight_gate /tmp/.adduser-ok /tmp/.preflight-sentinel
# Pin PATH so a future base-image change cannot silently break
# the entry script's bare-name lookups.
ENV PATH=/usr/local/bin:/bin:/usr/bin:/sbin
# Materialise the curated applet set as symlinks to the
# single root-only /bin/busybox. Done here (not in the scratch
# stage) because `FROM scratch` has no executable for `RUN` to
# invoke. The exec form is required: the shell form would need
# a pre-existing /bin/sh, and at this layer /bin/sh does not
# yet exist (it is about to be created by the loop).
#
# Build runs as root (no USER directive yet) so the 0700 mode
# permits the build step. After su-exec drops to the torrust
# user at runtime, /bin/busybox (and every symlink to it)
# returns EACCES.
#
# The applet list must match the §4.4 curated-applet table —
# update both in the same change. CI reconciliation (§4.4)
# warns on drift.
#
# The final rm removes the preflight sentinel that
# preflight_gate carries for the BuildKit dependency edge;
# it has no runtime purpose and should not ship.
RUN ["/bin/busybox", "sh", "-c", \
     "for a in sh adduser install mkdir dirname chown chmod tr mktemp cat printf rm echo; do \
        /bin/busybox ln -s busybox /bin/$a; \
      done && rm -f /tmp/.preflight-sentinel"]

## ── Runtime base: debug (full busybox on PATH) ───────────────
FROM gcr.io/distroless/cc-debian13:debug AS runtime_debug
# The :debug base ships a full /busybox/ tree with default
# (world-executable) permissions. Rather than copying the
# root-only runtime_assets bundle and then re-exposing a
# subset, simply layer the few root-only pieces the entry
# script needs (su-exec, entry.sh, /etc seed) and put
# /busybox/ on PATH so the unprivileged user retains access
# to the complete applet set — that is the debug image's
# purpose.
COPY --from=etc_seed --chmod=0644 --chown=0:0 /seed/etc/ /etc/
COPY --from=preflight_gate /tmp/.adduser-ok /tmp/.preflight-sentinel
# Pull su-exec from runtime_assets (which already copies it
# from gcc with the correct mode/ownership) so there is a
# single source for the compiled binary regardless of base.
COPY --from=runtime_assets /bin/su-exec /bin/su-exec
COPY --chmod=0555 --chown=0:0 \
    ./share/container/entry_script_sh  /usr/local/bin/entry.sh
# Materialise /bin/sh → /busybox/sh so root's recorded login
# shell in /etc/passwd (`/bin/sh`, seeded by etc_seed) and
# any consumer that execs an absolute `/bin/sh` (e.g.
# `su -`, certain runtime defaults) resolve to the donor's
# busybox. The release base creates the same symlink as part
# of its curated-applet loop; the debug base needs an
# explicit one because /bin/busybox doesn't exist here.
# The sentinel removal is folded into the same RUN layer.
RUN ["/busybox/sh", "-c", \
     "/busybox/ln -s /busybox/sh /bin/sh && rm -f /tmp/.preflight-sentinel"]
ENV PATH=/usr/local/bin:/busybox:/bin:/usr/bin:/sbin
```

The existing `test` and `test_debug` stages from the
current Containerfile are preserved with one change: the
`chmod -R a+x /app/bin` line is replaced by per-binary
modes. The application binary (`torrust-index`) keeps `0755`
(the unprivileged user must execute it); the root-phase-only
helper binaries (`torrust-index-health-check`,
`torrust-index-auth-keypair`, `torrust-index-config-probe`)
are tightened to `0500 root:root` — same posture as busybox,
`su-exec`, and `jq`. The healthcheck binary is invoked from
the `HEALTHCHECK` instruction, which runs as root (no
`--user` in the directive), so `0500` is sufficient.

The mode-and-ownership tightening happens in the `test` /
`test_debug` stages (where the binaries are produced), not
in the runtime stages, so the bits are baked in before the
`COPY --from=test /app/ /usr/` line in each runtime stage
ships them unchanged. Concretely, the existing
`chmod -R a+x /app/bin` becomes:

```dockerfile
# Replaces `chmod -R a+x /app/bin` in the test/test_debug
# stages. Application binary stays world-executable; helper
# binaries tighten to root-only (0500) and root-owned.
RUN chmod 0755 /app/bin/torrust-index && \
    chown 0:0 /app/bin/torrust-index-health-check \
              /app/bin/torrust-index-auth-keypair \
              /app/bin/torrust-index-config-probe && \
    chmod 0500 /app/bin/torrust-index-health-check \
               /app/bin/torrust-index-auth-keypair \
               /app/bin/torrust-index-config-probe
```

The runtime stages then pick the artefacts up via the
existing `COPY --from=test /app/ /usr/` (and
`COPY --from=test_debug /app/ /usr/`) lines without further
mode/ownership flags — `COPY` preserves both from the
source stage when no `--chmod` / `--chown` overrides are
present. Adding overrides at the runtime-stage `COPY` would
work too, but per-binary `--chmod` flags on a directory
copy are awkward; doing it once in the producing stage is
cleaner and keeps the per-binary intent next to the
binaries' build artefacts.

The two final targets differ in which `runtime_*` they
`FROM`:

```dockerfile
## ── Final: release ───────────────────────────────────────────
FROM runtime_release AS release
ENV TORRUST_INDEX_CONFIG_TOML_PATH=/etc/torrust/index/index.toml \
    TORRUST_INDEX_DATABASE_DRIVER=sqlite3 \
    USER_ID=1000 API_PORT=3001 IMPORTER_API_PORT=3002 \
    TZ=Etc/UTC RUNTIME=release
EXPOSE 3001/tcp 3002/tcp
VOLUME ["/var/lib/torrust/index","/var/log/torrust/index","/etc/torrust/index"]
COPY --from=test /app/ /usr/
# jq binary for entry-script JSON consumption (§2.2 step 4).
# Root-only (0500) — same posture as busybox and su-exec.
COPY --from=jq_donor --chmod=0500 --chown=0:0 /usr/bin/jq /usr/bin/jq
ENTRYPOINT ["/usr/local/bin/entry.sh"]
HEALTHCHECK --interval=5s --timeout=5s --start-period=3s --retries=3 \
  CMD /usr/bin/torrust-index-health-check "http://localhost:${API_PORT}/health_check" \
   && /usr/bin/torrust-index-health-check "http://localhost:${IMPORTER_API_PORT}/health_check"
CMD ["/usr/bin/torrust-index"]

## ── Final: debug ─────────────────────────────────────────────
# Mirrors release with two differences:
#   1. runtime base: `runtime_debug` leaves the donor's
#      `/busybox/` tree in place and on PATH so the
#      unprivileged `torrust` user retains shell access.
#   2. ENV block: adds `ENV=/etc/profile` (busybox `sh`
#      sources it on every interactive non-login invocation,
#      per POSIX) and sets `RUNTIME=debug`.
#
# Note: `$ENV` semantics are POSIX-standard and inverted
# from the more familiar `/etc/profile` rule. Busybox `sh`
# follows POSIX:
#   * Interactive non-login shells source `$ENV` (so a
#     plain `docker exec -it … sh` *does* load
#     /etc/profile).
#   * Login shells (`sh -l`, `su -`) source the system's
#     login profile mechanism instead, *not* `$ENV`.
# In practice the default `docker exec -it … sh` an operator
# reaches for is interactive non-login, so the profile loads
# without any extra flag. Operators who want the file in a
# non-interactive context (e.g. piping a script over stdin)
# must source it explicitly with `. /etc/profile`.
# Everything else — healthcheck, entrypoint, CMD — is
# identical. The debug image runs the same application;
# you just have a usable shell when you `docker exec` in.
FROM runtime_debug AS debug
ENV TORRUST_INDEX_CONFIG_TOML_PATH=/etc/torrust/index/index.toml \
    TORRUST_INDEX_DATABASE_DRIVER=sqlite3 \
    USER_ID=1000 API_PORT=3001 IMPORTER_API_PORT=3002 \
    TZ=Etc/UTC ENV=/etc/profile RUNTIME=debug
EXPOSE 3001/tcp 3002/tcp
VOLUME ["/var/lib/torrust/index","/var/log/torrust/index","/etc/torrust/index"]
COPY --from=test_debug /app/ /usr/
# jq binary for entry-script JSON consumption (§2.2 step 4).
# Root-only (0500) — same posture as busybox and su-exec.
COPY --from=jq_donor --chmod=0500 --chown=0:0 /usr/bin/jq /usr/bin/jq
ENTRYPOINT ["/usr/local/bin/entry.sh"]
HEALTHCHECK --interval=5s --timeout=5s --start-period=3s --retries=3 \
  CMD /usr/bin/torrust-index-health-check "http://localhost:${API_PORT}/health_check" \
   && /usr/bin/torrust-index-health-check "http://localhost:${IMPORTER_API_PORT}/health_check"
CMD ["/usr/bin/torrust-index"]
```

### 4.3 Design notes

- **One binary, many names.** Busybox dispatches on `argv[0]`,
  so `/bin/sh`, `/bin/adduser`, … all symlinked to
  `/bin/busybox` invoke the corresponding applet. The asset
  bundle contains exactly one busybox binary (~1 MB).

- **Curated subset, root-only.** Every applet the entry
  script needs is reachable at `/bin/<name>`; the
  unprivileged `torrust` user gets `EACCES` on `/bin/busybox`
  (and therefore every symlink to it) after `su-exec` drops
  privileges.

- **The `torrust` user has no usable login shell in release.**
  `adduser -s /bin/sh` records `/bin/sh` as the user's login
  shell, but in the release image `/bin/sh → /bin/busybox` is
  `0700 root:root`, so `su torrust` / `docker exec -u 1000 … sh`
  cannot spawn one. The application binary remains `0755` and
  executable; nothing in the normal request path shells out.
  In the debug image, `/busybox/sh` is on PATH and
  user-accessible — that is the debug image's purpose.

- **Security assumption.** The root-only mode relies on the
  unprivileged `torrust` user's GID set not including 0 and
  on no `cap_dac_*` capabilities being granted to the runtime
  process. Both hold under the documented compose/run flow.

- **PATH is pinned in both bases** so the entry script's
  bare-name calls resolve deterministically. In release,
  PATH does not include `/busybox/`; in debug, `/busybox/`
  appears on PATH before `/bin/` so the `:debug` base's
  user-accessible applets are the default resolution.

- **`docker exec -u root … sh` still works** for emergency
  production debugging on the release image. The `sh` lookup
  resolves through the pinned PATH to `/bin/sh → /bin/busybox`;
  `0700 root:root` permits root invocation. This is the
  documented break-glass procedure. In the debug image,
  `docker exec -u 1000 … sh` works directly — no root
  escalation needed.

- **`runtime_assets` is `FROM scratch`**, base-agnostic, and
  used only by the release base. It bundles the root-only
  busybox, `su-exec`, entry script, and account-file seed.
  The debug base copies `etc_seed` directly, pulls `su-exec`
  from `runtime_assets` (the single compiled artifact), skips
  the root-only `/bin/busybox`, and relies on the `:debug`
  donor's native `/busybox/` tree for all applets.

- **`preflight_gate` aggregates all donor-validation stages**
  (`busybox_preflight`, `adduser_preflight`) behind a single
  `FROM scratch` stage. Both `runtime_release` and
  `runtime_debug` COPY a sentinel from it, creating an
  explicit BuildKit dependency edge that prevents any
  preflight from being pruned regardless of which image
  variant is built. The sentinel is removed in each base's
  own `RUN` layer so it does not ship.
  `busybox_preflight` is also structurally upstream of
  `etc_seed` (which derives from it), so the busybox
  assertion cannot be pruned even without the gate; the
  gate remains because `adduser_preflight` has no such
  structural path — without the gate, a debug-only build
  could skip it entirely.

- **Distroless `nonroot` UID is not preserved.**
  `runtime_assets` overwrites `/etc/passwd` with a root-only
  seed. Privilege drop here uses `su-exec` to a
  runtime-created `torrust` user instead, so `nonroot` is
  unused. Re-add to `etc_seed` if a future workflow needs it.

- **Base `nsswitch.conf` is preserved.** `etc_seed` seeds
  only `passwd`, `group`, and `profile` — not
  `nsswitch.conf`. The `COPY /seed/etc/ /etc/` layer
  overwrites files present in both (deliberately replacing
  the base's `passwd` and `group` with root-only versions)
  but files absent from `etc_seed` — notably `nsswitch.conf`
  — are inherited from the base unchanged, so the base's
  `hosts: files dns` entry (needed for outbound name
  resolution to tracker, MySQL, SMTP hosts) survives.

- **No `/busybox/` directory in release.** The lean
  distroless base does not ship it; we extract only the
  single binary. The R6 bypass concern is eliminated
  entirely.

- **Busybox `install -D` compatibility.** The entry script
  calls `install -D -m 0640 ...` to seed config and database
  files. Busybox `install` supports `-D`, but its behaviour
  diverges from coreutils in edge cases; the
  `busybox_preflight` stage asserts the donor's busybox
  includes the applet so the failure surfaces at build, not
  first-boot.

- **`ENV` / `EXPOSE` / `VOLUME` blocks duplicated** between
  the two final targets. Dockerfile has no real mixin; keep
  the duplication honest and visible.

### 4.4 Curated applet reference

The release-base symlink loop must cover every applet the
entry script invokes by bare name. The table below is the
authoritative list; update it when the entry script changes.
Shell built-ins (`test`, `[`, `read`, `eval`, `case`,
`cd`, `exec`, `set`, `trap`, `export`, `.`) do not need
applet symlinks — busybox `sh` provides them internally.

| Applet     | Used by                                              |
|------------|------------------------------------------------------|
| `sh`       | Entry script interpreter (`#!/bin/sh`)                |
| `adduser`  | §4.1 — create `torrust` user at first boot            |
| `addgroup` | Defensive: some busybox builds of `adduser` exec the matching `addgroup` applet rather than calling it as an internal function. Linking it costs nothing and avoids a first-boot regression on a future donor bump. |
| `install`  | `inst()` helper — seed config/database templates      |
| `mkdir`    | Create volume subdirectories, auth-key dirs           |
| `dirname`  | §7.1 — resolve parent of auth-key and database paths  |
| `chown`    | Fix ownership on volumes, auth keys, seeded files     |
| `chmod`    | Fix permissions on volumes, auth keys                 |
| `tr`       | `to_lc` / `clean` helpers (lower-casing, char-class strip); §7.1 auth-key loops |
| `mktemp`   | Temporary file for auth-key generation                |
| `cat`      | MOTD assembly, profile sourcing                       |
| `printf`   | MOTD lines                                            |
| `rm`       | Clean up temp files                                   |
| `echo`     | Error messages, MOTD profile hook                     |
| `grep`     | Reserved for §7.1 entry-script extensions and ad-hoc operator break-glass debugging via `docker exec -u root`. |

`su-exec` is a standalone binary at `/bin/su-exec`, not a
busybox applet — it is not part of this loop.

**CI reconciliation.** To prevent the table and the symlink
loop from drifting, add a CI step that extracts the applet
list from the Containerfile's symlink-loop `for` statement
and compares it against a grep of bare-name external commands
used in `share/container/entry_script_sh`. The check is
advisory (warns on mismatch, does not block the build)
because the grep is necessarily heuristic — shell built-ins,
aliases, and dynamically constructed command names produce
false positives that a human must triage. The table above
remains the authoritative source; the CI step exists to
surface *potential* drift, not to enforce an exact match.

---

## Phase 5 — Schema & Credential Strip (D2)

**Files.** `share/default/config/*.toml`,
`src/config/v2/database.rs`, `src/config/v2/mod.rs`,
`src/config/v2/tracker.rs`.

### 5.1 Strip credentials and environment-coupled values

Remove literal `connect_url`, `token`, and `[mail.smtp]`
values across *all* files under `share/default/config/`. The
SQLite `connect_url` values contain no credentials but are
removed for consistency: a single rule ("no `connect_url`,
`token`, or environment-coupled host in shipped defaults") is
easier to enforce and audit than per-driver exceptions.

**In-scope file list (verified).** All eight files under
[`share/default/config/`](../share/default/config/) carry the
dev-only `token = "MyAccessToken"` and so are touched by this
phase — including
[`index.development.sqlite3.toml`](../share/default/config/index.development.sqlite3.toml),
which is not container-only but ships through the same
defaults dispatch. Five additionally carry `connect_url`,
`[mail.smtp]`, or `[auth]` path entries removed by the same
pass:

- `index.container.mysql.toml` — `connect_url`, `token`, `[auth]` paths, `[mail.smtp]`
- `index.container.sqlite3.toml` — `connect_url`, `token`, `[auth]` paths, `[mail.smtp]`
- `index.development.sqlite3.toml` — `token`
- `index.private.e2e.container.sqlite3.toml` — `connect_url`, `token`, `[auth]` paths, `[mail.smtp]`
- `index.public.e2e.container.mysql.toml` — `connect_url`, `token`, `[auth]` paths, `[mail.smtp]`
- `index.public.e2e.container.sqlite3.toml` — `connect_url`, `token`, `[auth]` paths, `[mail.smtp]`
- `tracker.private.e2e.container.sqlite3.toml` — `token`
- `tracker.public.e2e.container.sqlite3.toml` — `token`

**Bare-metal developer impact.**
`index.development.sqlite3.toml` is not container-only —
it is the starting-point template for `cargo run`-based
development. After this change, a developer who copies
it verbatim must supply `connect_url` and `token` via
env var or add them to their local copy. This is the
canonical file behind the ADR’s Consequences note
(“Bare-metal developers … are also affected”).

The `[auth]` path entries (`private_key_path`,
`public_key_path`) in the container-oriented files are
environment-coupled values (they encode the container's
volume layout). D2's rule strips them for the same reason
it strips `connect_url`: the entry script owns these paths
and exports them via `TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__*`
(see D3).

The two `tracker.*` files ship from the index repo because
they are consumed by the e2e compose flow alongside the
index-side defaults; they are not tracker-internal config.
Note: these files use the **tracker service's** own config
schema (with `[core.database]`, `[[udp_trackers]]`,
`[http_api]` sections) — their `[tracker].token` is the
tracker's admin-API access token, not the index's
`config::v2::tracker::Tracker::token` field. The
schema-level mandatory-token change (§5.2's pattern) applies
only to the index's `Tracker` struct; the tracker TOMLs are
touched here solely for the P1 credential-stripping rule.

Folding the mail strip into this phase (rather than into
Phase 8) keeps Phase 8 a pure compose-layer change and keeps
the "no environment-coupled values in shipped defaults" rule
in one place.

**Mechanism.** Ship the keys absent and rely on the existing
`TORRUST_INDEX_CONFIG_OVERRIDE_*` env-var mechanism. The
operator must supply
`TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN` and
`..._DATABASE__CONNECT_URL` at startup.

**Make `tracker.token` mandatory.** Drop
`#[serde(default = "Tracker::default_token")]` on
`Tracker::token` and remove `Tracker::default_token()` in
[`src/config/v2/tracker.rs`](../src/config/v2/tracker.rs),
same pattern as `connect_url` below. This keeps one rule for
credentials ("no defaults") rather than a sentinel-rejection
branch in the probe. The probe's exit-4 gate for empty tokens
remains as defence in depth — it covers a real gap because
`ApiToken`'s `#[derive(Deserialize)]` constructs the inner
`String` directly, bypassing the `assert!(!key.is_empty())`
guard in `ApiToken::new`; `token = ""` in TOML would
otherwise silently produce an empty token.
`impl Default for Tracker`
cannot supply a `token` value any more, so drop the impl
entirely. Also delete `Settings::default_tracker()` (which
becomes dead code once its `#[serde(default)]` attribute is
removed) and `Tracker::default_token()`. The audit below
confirms there is exactly one caller and it is rewired in
the same change.

**Audit of `Tracker::default` consumers (verified).** A grep
for `Tracker::default|Tracker \{|Tracker\{|default_tracker|default_token`
across `src/` and `tests/` turns up exactly one production
call site:
[`src/config/v2/mod.rs`](../src/config/v2/mod.rs), where
`Settings::default_tracker()` → `Tracker::default()` serves
as the serde default for the enclosing `Settings.tracker`
field via `#[serde(default = "Settings::default_tracker")]`.
The two `pub struct Tracker { ... }` hits in
`src/web/api/client/v1/contexts/settings/mod.rs` and
`tests/common/contexts/settings/mod.rs` are unrelated
(API-shape DTOs, not the config struct). No test code calls
`config::v2::tracker::Tracker::default()` directly. Dropping
`impl Default for Tracker` therefore forces a single parallel
change: remove the `#[serde(default = "default_tracker")]`
attribute on `Settings.tracker` so an absent `[tracker]`
block fails the same way as an absent `token`. This mirrors
the `Database` pattern exactly (§5.2).

**Interaction with `check_mandatory_options`.** The existing
`load_settings()` validates `"tracker.token"` as a mandatory
option via `figment.find_value()` — a check that predates
this refactor. Once the `#[serde(default)]` is removed,
`figment.extract()` produces a serde `missing field` error
for the same case, making `check_mandatory_options`'s
`tracker.token` entry redundant. Remove `"tracker.token"`
from the `mandatory_options` array in the same change.
`"logging.threshold"` and `"metadata.schema_version"` remain
(both still have serde defaults that this phase does not
touch). If a future phase removes all entries, the function
itself can be deleted.

### 5.2 Make `database.connect_url` mandatory

Drop the `#[serde(default = "...")]` attribute on
`connect_url` in
[`src/config/v2/database.rs`](../src/config/v2/database.rs)
and remove the `impl Default for Database` block. Keep the
field typed as `Url`.

**Audit of `Database::default` consumers (verified).** A grep
for `Database::default|Database \{|Database\{` across `src/`
and `tests/` turns up exactly one production call site:
[`src/config/v2/mod.rs`](../src/config/v2/mod.rs), where
`default_database()` serves as the serde
default for the enclosing `TorrustIndex.database` field.
No test code in `src/tests/` or `tests/` calls
`config::v2::database::Database::default()`. Dropping
`impl Default for Database` therefore forces a single
parallel change: remove the
`#[serde(default = "default_database")]` attribute on
`TorrustIndex.database` so an absent `[database]` block
fails the same way as an absent `connect_url`.

The two `pub struct Database { ... }` hits in
`src/web/api/client/v1/contexts/settings/mod.rs` and
`tests/common/contexts/settings/mod.rs` are unrelated
(API-shape DTOs, not the config struct).

With the serde default gone, omitting `connect_url` fails
deserialisation with a precise `missing field 'connect_url'`
error pointing at the exact section. No
`check_mandatory_options` branch is needed.

**Sub-options considered and rejected.**

- **(a)** Same as the chosen change but *also* changes the
  field to `Option<Url>` and adds a `check_mandatory_options`
  branch. Conflates "mandatory" with "type change" and forces
  every `&Url` consumer to handle the `Option`. Rejected.
- **(b)** Keep the serde default and reject the known
  sentinel value in `check_mandatory_options`. Brittle (a
  typo-equivalent slips through) and the invariant lives far
  from the type. Rejected.
- **(c)** Two-stage `RawDatabase` → `Database` validation.
  Adds a phantom type whose only job is to be unwrapped once.
  Rejected.

---

## Phase 6 — Config Probe Helper

**Files.** New `packages/index-config-probe/` crate.

The current entry script unconditionally seeds the empty
database template at
`/var/lib/torrust/index/database/sqlite3.db` regardless of
what `..._DATABASE__CONNECT_URL` resolves to. Phase 5 makes
`connect_url` mandatory, so the script must now read it and
decide whether to seed at all and where.

The same problem shape recurs for auth keys (§7.1): the
script wants to know "what would the application resolve
for this field, given the operator's TOML *and* env-var
overrides?" — and shell can't answer that question without
re-implementing the parser.

**The shell does not parse anything.** The chosen approach is
a single helper that calls the application's actual config
loader — the one extracted in Phase 3 — and emits the
resolved values as a JSON object on stdout (P9). Script and
application are byte-identical on what they see because they
share the parser. The script consumes the JSON via `jq`;
the future Rust entry binary (see
[Carry-Over](009-container-infrastructure-refactor.md#carry-over-items))
will consume it via `serde_json::from_reader`.

### 6.1 New helper crate `torrust-index-config-probe`

A small workspace crate `packages/index-config-probe/`
(binary `torrust-index-config-probe`) loads the same
`Settings` the application loads and emits the
container-relevant resolved values as a JSON object on
stdout.

**Crate dependencies.** `torrust-index-config` (path
dependency from Phase 3) and `torrust-index-cli-common`
(P9 scaffolding from §2.0). The helper inherits the parsing
surface (`figment`, `toml`, `serde`, `serde_with`, `url`,
`camino`, `derive_more`, `thiserror`, `tracing`) via
`torrust-index-config`; it adds no parsing of its own.
`figment` is declared with `default-features = false` and
an explicit feature allowlist (`toml`, `env`) in
`torrust-index-config`'s `Cargo.toml` so a future feature
flip cannot smuggle `tokio` in transitively.

**Contract.**

```text
Usage: torrust-index-config-probe

Loads the application's configuration through the same
torrust-index-config loader the application uses, honouring
TORRUST_INDEX_CONFIG_TOML, TORRUST_INDEX_CONFIG_TOML_PATH,
and every TORRUST_INDEX_CONFIG_OVERRIDE_* env var. No CLI
flags override the config-file path — callers who need a
different path set TORRUST_INDEX_CONFIG_TOML_PATH in the
environment before invoking the probe, the same mechanism
the application uses. This preserves the "shared parser"
property: the probe's resolution is byte-identical to the
application's because it uses the exact same code path with
no bespoke flag-splicing logic.

Refuses to run when stdout is a TTY (exit 2, per P8).

On success (exit 0), emits one JSON object + trailing
newline on stdout (P9):

{
  "schema": 1,
  "database": {
    "driver": "sqlite",
    "path": "/var/lib/torrust/index/data.db"
  },
  "auth": {
    "private_key": {
      "pem_set": false,
      "path_set": true,
      "source": "path",
      "path": "/etc/torrust/index/auth/private.pem"
    },
    "public_key": {
      "pem_set": false,
      "path_set": true,
      "source": "path",
      "path": "/etc/torrust/index/auth/public.pem"
    }
  }
}

Field semantics:

  schema          Always 1. Incremented on breaking changes.
  database.driver URL scheme extracted from connect_url
                  (sqlite | mysql | mariadb). Not the
                  Containerfile's TORRUST_INDEX_DATABASE_DRIVER
                  env var (which takes sqlite3 / mysql).
  database.path   For sqlite: the file path (absolute,
                  relative, or ":memory:"). For non-sqlite:
                  null.
  auth.*.pem_set  Raw presence (non-empty after resolution)
                  before PEM-overrides-PATH precedence.
  auth.*.path_set Same for the path field.
  auth.*.source   Winner after precedence: "pem", "path",
                  or "none".
  auth.*.path     Resolved path if source is "path"; null
                  otherwise.

Exit codes:

  0  Recognised, well-formed configuration.
  1  Internal error (unhandled panic, unexpected I/O).
  2  Stdout is a TTY (P8), or clap argv-parse failure.
  3  Config-load failure (missing field, parse error, IO
     error). The underlying error message is forwarded
     verbatim to stderr via tracing.
  4  Security-critical field present but empty. Currently:
     tracker.token. The probe rejects it at the container
     boundary so that a bare ${VAR} in compose that
     substitutes to "" fails at startup.
  5  Unrecognised database scheme.
```

PEM material is *never* emitted, only its presence
(`"pem_set": true`). The probe's stdout is safe to log.

**Naming note.** `database.driver` is the URL *scheme*
extracted from `connect_url`, not the Containerfile's
`TORRUST_INDEX_DATABASE_DRIVER` env var. The two use
different taxonomies (`sqlite` vs. `sqlite3`). Phase 7
replaces the entry script's `TORRUST_INDEX_DATABASE_DRIVER`
dispatch with the probe's `database.driver` output, so the
env var becomes a Containerfile-level selector for which
default TOML to seed — it no longer drives runtime database
decisions.

**URL resolution behaviour.** The helper does *minimal*
decoding — just enough to dispatch on scheme and extract a
path for the entry script's seeding decisions. It does not
fully interpret the URL; the database engine validates and
rejects the connection string on its own terms.

`Settings::database.connect_url` is typed as `url::Url`, so
`url::Url::parse` runs at deserialisation time. The helper
reads `Url::scheme()` for driver dispatch. For `sqlite`
URLs, extracting the file path requires scheme-specific
logic because `url::Url` treats non-authority URLs
differently from hierarchical ones (e.g.
`sqlite://data.db?mode=rwc` puts `data.db` in the *host*
slot, not the path; `sqlite::memory:` is opaque). The
helper handles these cases explicitly — it is not a
pass-through of `.path()`. For non-`sqlite` schemes the
helper emits `null` for `database.path` and lets the engine
own the semantics entirely.

The script does not enumerate spellings; the table below
shows the helper's *dispatch* output, not a promise about
what the database engine will accept:

| Spelling                              | `database.driver` | `database.path`              |
|---------------------------------------|--------------------|------------------------------|
| `sqlite://data.db?mode=rwc`           | `"sqlite"`         | `"data.db"` (relative)       |
| `sqlite:///var/lib/torrust/index.db`  | `"sqlite"`         | `"/var/lib/torrust/index.db"`|
| `sqlite::memory:`                     | `"sqlite"`         | `":memory:"`                 |
| `sqlite:///srv/My%20Data/x.db`        | `"sqlite"`         | `"/srv/My Data/x.db"`        |
| `mysql://user:pass@host:3306/db`      | `"mysql"`          | `null`                       |
| `mariadb://...`                       | `"mariadb"`        | `null`                       |
| `postgres://...`                      | exit 5             | (stderr: "unsupported scheme: postgres") |
| (`connect_url` missing — see §5.2)    | exit 3             | (stderr: serde "missing field" message) |

Tests pin every row against the helper's actual output; rows
that turn out to behave differently from the table are *the
table's* bug, to be fixed in the docs, not in the helper.
The database engine remains the authority on whether a given
URL is actually usable — the helper's job is only to decide
"seed or not, and where".

**Why a separate crate, not a `[[bin]]` on `torrust-index-config`.**
Keeping the binary in its own crate keeps the parsing crate
binary-free (faster compile, no `clap` or P9 CLI deps in the
parser's tree), and lets a future contributor add another
small probe binary alongside this one without polluting the
config crate's manifest.

**Tests.** Unit tests cover every row of the spelling
table, the auth-key matrix (PEM-only, PATH-only, none, both
PEM+PATH), missing-`connect_url`, empty `tracker.token`,
and the `postgres:` scheme. Helper is a pure function of
`(env, files)` → JSON/exit; testing is straightforward.

---

## Phase 7 — Entry-Script Contract (D3)

**Files.** `share/container/entry_script_sh`,
`docs/containers.md`.

**Runtime execution order.** The snippets below are presented
by *topic* (auth-key logic in §7.1, probe integration in
§7.2), not by runtime order. When assembling the entry
script, the actual sequence is:

1. Probe invocation — §7.2.
2. `jq` field extraction — §7.2.
3. Schema version gate — §7.2.
4. Post-probe PEM/PATH mutual-exclusion check — §7.1.
5. Pair-completeness check (post-probe) — §7.1.
6. Cross-pair source consistency check — §7.1.
7. Three-way auth-key dispatch (post-probe) — §7.1.
8. Volumes-only directory guard — §7.1.
9. Key materialisation (generate missing key files) — §7.1.
10. Database seeding dispatch — §7.2.

Steps 3–9 reference variables populated by steps 1–2;
they must not appear before those steps in the assembled
script. All helper functions (`seed_sqlite`,
`key_configured`, `inst`) are defined at the top of the
script, before any executable statements, so they are
available at every call site. The legacy `to_lc`, `clean`,
and `cmp_lc` helpers become dead code after this phase
(the database dispatch migrates to the probe's
`database.driver` output and the MOTD section migrates to
a plain `case $RUNTIME in`) and are removed in the same
change.

**Shell discipline.** The entry script runs under `set -eu`
(added at the top, after the existing `DEBUG=1` → `set -x`
line). `set -e` ensures unchecked failures (e.g. `mkdir -p`)
abort immediately rather than proceeding with a broken state.
`set -u` catches typos in variable names. The `${var:-}`
patterns throughout the snippets below are `set -u`-safe
by construction.

**`set -eu` audit prerequisite.** Before adding `set -eu`,
perform a line-by-line audit of the existing
`share/container/entry_script_sh` for commands whose non-zero
exit is currently benign. Known hazards in the current script:

- **`inst()` function.** The current implementation uses a
  bare `if ... fi` with no `else` clause:
  ```sh
  inst() {
      if [ -n "$1" ] && [ -n "$2" ] && [ -e "$1" ] && [ ! -e "$2" ]; then
          install -D -m 0640 -o torrust -g torrust "$1" "$2"; fi; }
  ```
  Safe under `set -e`: POSIX §2.9.4.1 specifies that an `if`
  compound with no `else` clause returns zero when the
  condition is false ("The exit status of the if command
  shall be … zero, if none was executed"). The `[` exit
  codes inside the condition are in a tested position and
  are not propagated. No fix needed.
- **`cmp_lc()` return value.** Removed in this phase (see
  §7 preamble); no audit action needed. If for any reason
  it survives: returns 1 on mismatch; safe when called
  inside `if`/`elif` (POSIX `set -e` exempts commands in
  conditional positions), but verify no bare call site
  exists.
- **`$RUNTIME` / `$USER_ID` / `$TORRUST_INDEX_DATABASE_DRIVER`.**
  Currently referenced without `${VAR:-}` guards. These are
  always set by the Containerfile's `ENV` declarations, but
  bare-metal testing of the script outside Docker will fail
  under `set -u`. Add `${VAR:-}` guards where appropriate, or
  document that the script is container-only.
- **`chown -R` / `chmod -R` on volumes (lines 23–24).**
  If a volume directory doesn't exist, these fail. Currently
  `mkdir -p` on the preceding line creates them, so the
  failure is unlikely — but verify the ordering is airtight.

Every command in the script that legitimately returns
non-zero must be wrapped in `if`, `||`, or `&&` to avoid
`set -e` triggering. As a general rule: any helper function
whose last statement is a conditional test must end with
`|| return 0` (or an explicit `return 0`) when the function
is called outside a conditional position — `set -e` does
not exempt return values that propagate from a function's
final command in that case. (Functions called *inside* a
conditional position — `if`, `&&`, `||` — inherit the
exemption for their entire body per POSIX §2.8.1.)
Bracket this audit as an explicit
sub-task of Phase 7; do not introduce `set -eu` without
completing it.

**Scoped `eval` for computed variable names.** Because the
probe emits structured JSON and the script extracts fields
with `jq`, the `getvar()` / `setvar()` / `assert_in_set()`
helpers from earlier drafts are unnecessary. Every variable
is assigned directly from a `jq -r` call. The auth-key
loops (§7.1) use a small number of scoped `eval`s to
dereference computed variable names (`auth_${pair}_source`,
etc.) — these are the *only* `eval` uses in the script, each
assigning from a variable the preceding `jq` call populated.
The alternative — duplicating each loop body once per key
pair — is acceptable if zero-`eval` is preferred; the two
formulations are equivalent for two pairs.

### 7.1 Respect auth-key overrides

The `Auth` schema treats `*_PEM` as a per-field override of
`*_PATH`. Throughout this section, `<PAIR>` ranges over
{`PRIVATE_KEY`, `PUBLIC_KEY`} so the env var spelling matches
the live contract (`..._AUTH__PRIVATE_KEY_PEM`,
`..._AUTH__PUBLIC_KEY_PATH`, etc., as verified against
[`src/config/v2/auth.rs`](../src/config/v2/auth.rs) and
[README.md](../README.md)).

For each key in {private, public}:

1. If `..._AUTH__<PAIR>_PEM` is set (non-empty), skip
   generation of that key — the application loads it from the
   env var directly.
2. Else, if `..._AUTH__<PAIR>_PATH` is set (non-empty), use
   that location for first-boot key generation (skip if the
   file already exists and is non-empty). Directory-creation
   semantics for this branch are specified below.
3. Else, **the entry script itself sets**
   `..._AUTH__<PAIR>_PATH` to its built-in container default
   (`/etc/torrust/index/auth/private.pem` and
   `.../public.pem`) **before exec'ing the application**, then
   proceeds as case 2. There is one source of truth (the
   script) and the application is informed of it through the
   same channel operators use.

**Cross-pair source constraint.** Both keys must use the
same delivery mechanism (both PEM or both PATH/none). The
generator emits a matched keypair in one invocation and
cannot produce a single key file, so a mixed configuration
(e.g. private via PEM, public via PATH) would leave the
script unable to materialise the PATH-side key. The
pair-completeness check (below) rejects such combinations
at startup.

**Post-probe mutual-exclusion check.** The probe emits
raw-presence booleans (`auth.*.pem_set`,
`auth.*.path_set`) alongside the resolved `source`.
The script extracts these via `jq` and enforces mutual
exclusion *after* parsing the probe output (step 3 in the
runtime execution order), so it catches all collision
shapes — env↔env, TOML↔env, TOML↔TOML — using the same
resolved config stack the application sees.

Pair-completeness ("both keys have a source, or neither")
is also checked post-probe — see the auth-key path
resolution block below — because an operator may configure
one key via env var and the other via mounted TOML. Only
the probe's resolved view can see both.

```sh
# ── Post-probe mutual-exclusion check (runs after §7.2's
# ── jq extraction) ────────────────────────────────────────
# Within a single key, PEM and PATH cannot both be set —
# regardless of source (env var, TOML, or a mix). The
# probe's pem_set / path_set raw-presence booleans
# report whether each field resolved to a non-empty value
# before PEM-overrides-PATH precedence. This catches cross-
# source collisions (e.g. PEM in mounted TOML, PATH in env
# var) that a pre-probe env-var-only check would miss.
for pair in private_key public_key; do
    pem_var="auth_${pair}_pem_set"
    path_var="auth_${pair}_path_set"
    eval "pem_set=\"\$$pem_var\""
    eval "path_set=\"\$$path_var\""
    if [ "$pem_set" = true ] && [ "$path_set" = true ]; then
        uc_pair=$(printf '%s' "$pair" | tr '[:lower:]' '[:upper:]')
        echo "ERROR: both ${uc_pair}_PEM and ${uc_pair}_PATH are set;" \
             "these are mutually exclusive — pick one." >&2
        exit 1
    fi
done
```

**Note on `eval` usage in the auth-key loops.** The
mutual-exclusion loop, three-way dispatch loop, and
volumes-only directory guard each use `eval` to dereference
computed variable names (e.g. `eval "src=\"\$$src_var\""`).
Every right-hand side is double-quoted inside the `eval`
string so that paths containing spaces survive expansion
(the probe's spelling table includes
`/srv/My%20Data/x.db` → `"/srv/My Data/x.db"`).
Every `eval`'d expansion references a variable populated by
the preceding `jq` extraction (§7.2), so the values are
well-formed by construction and are `set -u`-safe because
the runtime execution order (§7 preamble) guarantees §7.2's
assignments precede §7.1's checks. Under `set -x`, the trace
shows each assignment clearly. The alternative — duplicating
each loop body for private_key and public_key — is
acceptable if zero-`eval` is preferred; the two
formulations are equivalent for two pairs.

**Verification.** Add an integration test that runs the entry
script with no `..._AUTH__*` env vars in the environment, lets
it complete its pre-exec setup, and asserts that
`..._AUTH__PRIVATE_KEY_PATH` and `..._AUTH__PUBLIC_KEY_PATH`
are exported with non-empty values pointing at files the
script created. This replaces the original "two values agree
across files" invariant with a single-source-of-truth check.

**Directory-creation rules for case 2.** When the parent
directory of an override path does not exist, the entry
script (running as root) restricts auto-creation to the
volumes it already owns and chowns at startup
(`/etc/torrust/index/`, `/var/lib/torrust/index/`,
`/var/log/torrust/index/`); any path outside those roots is
the operator's responsibility:

After §7.2 integration the script dispatches on the
probe's `auth.*.source` JSON field. The probe reports what
the config stack resolves to — it does not inject container
defaults. When `source="none"` (no PEM, no PATH configured
anywhere — TOML or env vars), the script itself applies
case 3: it sets the built-in container default path and
exports the override env var so the application is informed
through the same channel operators use.

```sh
# ── Auth-key pair-completeness (post-probe) ───────────────
# Prerequisite: §7.2 steps 1–2 (probe invocation + jq
# extraction) must have run before this block.
# Both keys must have a source, or neither. Mirrors the
# application invariant in src/jwt.rs. This runs *after* the
# probe so that TOML-only, env-only, and mixed-source pairs
# are all visible.
#
# Test positively against the closed set {pem,path} rather
# than negatively against `none`: the JSON schema guarantees
# source is one of {pem,path,none}, so the two formulations
# are equivalent today, but the positive form fails closed
# if a future probe value is added without updating this
# block.
key_configured() {
    case $1 in
        pem|path) return 0 ;;
        *)        return 1 ;;
    esac
}
private_has=0; key_configured "$auth_private_key_source" && private_has=1
public_has=0;  key_configured "$auth_public_key_source"  && public_has=1
if [ "$private_has" -ne "$public_has" ]; then
    echo "ERROR: auth keys must be configured as a complete pair;" \
         "one key is configured but the other is not." >&2
    exit 1
fi

# Cross-pair mixed-source check: both path-side keys must use
# the same delivery mechanism.  The generator emits a matched
# keypair in one invocation, so it cannot produce a single
# key file — if one key is delivered via PEM (env var) and
# the other via PATH (file), the script has no way to
# materialise the PATH-side key from the PEM-side material.
# Reject early rather than silently skipping materialisation.
if [ "$private_has" -eq 1 ] && [ "$public_has" -eq 1 ] \
   && [ "$auth_private_key_source" != "$auth_public_key_source" ]; then
    echo "ERROR: private key source is '$auth_private_key_source'" \
         "but public key source is '$auth_public_key_source';" \
         "mixed PEM/PATH across the key pair is not supported" \
         "— use the same delivery mechanism for both keys." >&2
    exit 1
fi

# ── Auth-key path resolution (post-probe) ─────────────────
# Three-way dispatch per key, matching §7.1 cases 1/2/3.
# After this block, private_key_path and public_key_path are
# set for every non-PEM key.
for pair in private_key public_key; do
    src_var="auth_${pair}_source"
    pth_var="auth_${pair}_path"
    eval "src=\"\$$src_var\""
    eval "pth=\"\$$pth_var\""
    uc_pair=$(printf '%s' "$pair" | tr '[:lower:]' '[:upper:]')

    case $src in
        pem)
            # Case 1: app loads from env var directly.
            # Nothing for the script to do.
            continue
            ;;
        path)
            # Case 2: operator (or TOML) configured a path.
            eval "${pair}_path=\"\$pth\""
            ;;
        none)
            # Case 3: no auth source configured — apply the
            # container default and inform the application.
            case $pair in
                private_key) default=/etc/torrust/index/auth/private.pem ;;
                public_key)  default=/etc/torrust/index/auth/public.pem  ;;
            esac
            eval "${pair}_path=\"\$default\""
            export "TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__${uc_pair}_PATH=$default"
            ;;
    esac
done

# ── Volumes-only directory guard ──────────────────────────
# Applies to case 2 and case 3 (case 1 skipped above).
for pair in private_key public_key; do
    src_var="auth_${pair}_source"
    eval "src=\"\$$src_var\""
    [ "$src" = pem ] && continue

    eval "keypath=\"\${${pair}_path}\""
    d=$(dirname "$keypath")
    [ -d "$d" ] && continue
    case "$d" in
        /etc/torrust/index|/etc/torrust/index/*|\
        /var/lib/torrust/index|/var/lib/torrust/index/*|\
        /var/log/torrust/index|/var/log/torrust/index/*)
            mkdir -p "$d"
            chown torrust:torrust "$d"
            chmod 0700 "$d"
            ;;
        *)
            echo "ERROR: auth key path $d is outside the volumes" \
                 "the entry script manages." >&2
            echo "       Pre-create it with appropriate ownership," \
                 "or place keys under /etc/torrust/index/ or" \
                 "/var/lib/torrust/index/." >&2
            exit 1
            ;;
    esac
done

# ── Key materialisation (cases 2 and 3) ──────────────────
# Generate missing key files. Runs after the directory guard
# so the parent directory is guaranteed to exist.
# Both keys are generated together as a pair (the generator
# emits a matched keypair in one invocation). If either file
# is missing or empty, regenerate both — a half-pair is not
# useful.
if [ -n "${private_key_path:-}" ] && [ -n "${public_key_path:-}" ]; then
    if [ ! -s "$private_key_path" ] || [ ! -s "$public_key_path" ]; then
        keypair_json=$(/usr/bin/torrust-index-auth-keypair)
        printf '%s' "$keypair_json" | jq -r .private_key_pem > "$private_key_path"
        printf '%s' "$keypair_json" | jq -r .public_key_pem  > "$public_key_path"
        chown torrust:torrust "$private_key_path" "$public_key_path"
        chmod 0400 "$private_key_path"
        chmod 0400 "$public_key_path"
    fi
fi
```

The directory guard (above) converts an otherwise silent
runtime `EACCES` (when the application tries to read keys
from a directory it cannot enter) into an early, actionable
startup error. The materialisation block generates a fresh
RSA keypair via the Phase 2 helper binary (§2.2), consuming
its JSON output through `jq` — the same `jq` binary
introduced in §2.2 step 4 and shared with the probe
consumption in §7.2. Both keys are mode `0400` (owner
read-only); the application process runs as the `torrust`
user who owns the files, so read access is granted through
ownership rather than group bits.

**Known gap.** The TOML-invisibility gap noted in earlier
drafts is closed by the config-resolution probe
([§6.1](#61-new-helper-crate-torrust-index-config-probe)):
the probe loads the full TOML + env-var stack, so the script
sees the same resolved values the application sees regardless
of source. The probe does *not* inject container defaults —
it reports `source=none` honestly when nothing is configured.
The script's post-probe dispatch (the three-way case above)
applies case 3 only when `source=none`, i.e. when the
application would genuinely fall back to ephemeral keys.
Because the probe has already resolved the TOML layer, the
script's `export` in the `none` branch cannot shadow an
operator's TOML-provided path — it only fires when no path
exists to shadow.

### 7.2 Entry-script integration

**Temporal contract.** The probe runs exactly once, *before*
the script exports any `TORRUST_INDEX_CONFIG_OVERRIDE_*` env
vars of its own. The probe's output therefore reflects the
operator's true configuration (TOML + operator-supplied env
vars) with no script-injected values. Post-probe, the script
may `export` overrides (e.g. case-3 auth-key defaults); those
exports are consumed by the subsequent `exec` of the
application, not by the probe.

The script's probe-consumption section:

```sh
# Probe the resolved configuration via the same parser the
# application uses. This sees TOML + env-var overrides as
# the application sees them. The probe emits one JSON object
# on stdout (P9 contract).
probe_json=$(/usr/bin/torrust-index-config-probe) || exit $?

# ── Schema version gate ──────────────────────────────────
# Fail fast on a probe/script version mismatch rather than
# silently misinterpreting fields that changed meaning.
probe_schema=$(printf '%s' "$probe_json" | jq -r '.schema')
if [ "$probe_schema" != "1" ]; then
    echo "ERROR: config probe emitted schema=$probe_schema" \
         "but this entry script expects schema=1" \
         "— possible probe/script version mismatch" >&2
    exit 1
fi

# ── jq field extraction ──────────────────────────────────
# Each variable is assigned directly from a jq call. The
# JSON schema (§6.1) defines the field paths; jq -r returns
# the raw string value (no JSON quoting), or "null" for
# absent fields. The -e flag is not used here — null fields
# (e.g. database.path for MySQL) are legitimate.
database_driver=$(printf '%s' "$probe_json" | jq -r '.database.driver')
database_path=$(printf '%s' "$probe_json"   | jq -r '.database.path // empty')

auth_private_key_pem_set=$(printf '%s' "$probe_json"  | jq -r '.auth.private_key.pem_set')
auth_private_key_path_set=$(printf '%s' "$probe_json" | jq -r '.auth.private_key.path_set')
auth_private_key_source=$(printf '%s' "$probe_json"   | jq -r '.auth.private_key.source')
auth_private_key_path=$(printf '%s' "$probe_json"     | jq -r '.auth.private_key.path // empty')

auth_public_key_pem_set=$(printf '%s' "$probe_json"   | jq -r '.auth.public_key.pem_set')
auth_public_key_path_set=$(printf '%s' "$probe_json"  | jq -r '.auth.public_key.path_set')
auth_public_key_source=$(printf '%s' "$probe_json"    | jq -r '.auth.public_key.source')
auth_public_key_path=$(printf '%s' "$probe_json"      | jq -r '.auth.public_key.path // empty')

case $database_driver in
    sqlite)
        seed_sqlite "$database_path"
        ;;
    mysql|mariadb)
        # No file to seed; the application connects directly.
        ;;
    *)
        # The probe already rejects unknown schemes (exit 5),
        # so this branch indicates a probe-vs-script version
        # mismatch (e.g. probe was rebuilt with a new scheme
        # spelling but the script's case arms were not updated).
        echo "ERROR: unexpected database.driver='$database_driver'" \
             "from config probe — possible probe/script version mismatch" >&2
        exit 1
        ;;
esac
```

**Why `jq` instead of IFS-parsing or a heredoc loop.** The
probe emits structured JSON (§6.1). Earlier drafts used a
`while IFS== read -r k v` heredoc loop over LF-delimited
`key=value` output, with `getvar()` / `setvar()` / `eval`
helpers and an `assert_in_set()` intake-validation gate. The
JSON + `jq` approach eliminates all of that:

- **Minimal `eval`** — scoped to the auth-key loops in §7.1
  (computed variable dereference only; see the note there).
  No `getvar()` / `setvar()` wrappers, no indirect expansion
  of probe output.
- **No intake validation gate.** The JSON schema is the
  contract; `jq` extracts exactly the documented field paths.
  An unknown field is silently ignored (harmless). A missing
  required field produces an empty string, caught by the
  downstream dispatch (`case $database_driver in … *) …`).
  A structurally invalid JSON blob fails `jq` with exit 5,
  which the script's `set -e` propagates.
- **No forbidden-character analysis.** LF-delimited output
  required proving that no value could contain `\n` or `=`.
  JSON string escaping handles all byte values; `jq -r`
  unescapes them. The contract is the JSON RFC, not a
  bespoke forbidden-set.
- **No heredoc quoting analysis.** The `<<PROBE_OUTPUT` /
  `${parsed}` pattern required proving that shell expansion
  inside the heredoc could not corrupt values. With `jq`,
  the JSON is never interpolated into shell syntax — it
  flows through a pipe.

The cost is a `jq` dependency in the runtime image (see
§7.3).

**Seeding rules** (`seed_sqlite "$path"`):

- `path` is empty (MySQL/MariaDB, or `jq` returned empty for
  a null JSON field) — skip; the `case $database_driver`
  dispatch above already handles non-sqlite drivers.
- `path = :memory:` — skip silently; log an info-level note
  that the database is ephemeral.
- relative `path` — log a warning ("relative SQLite path;
  not seeding — application will create on first connect if
  `mode=rwc`") and skip. No `WORKDIR` is set in the
  Containerfile, so the runtime CWD is `/`; guessing an
  absolute location would write to the wrong place.
- absolute `path`, file exists and non-empty — leave alone
  (no seed, no chown; the operator owns the file).
- absolute `path`, file exists but zero bytes, or file does
  not exist — apply the volumes-only auto-mkdir rule from
  §7.1 to the parent directory, then delegate to the
  existing `inst()` helper (`install -D -m 0640 -o torrust
  -g torrust "$template" "$path"`). `inst()` already handles
  create-parents, set mode 0640, set ownership, and
  skip-if-dest-exists in a single `install -D` call — no
  reason to duplicate that with separate `cp` + `chown` +
  `chmod`. (The zero-byte case needs the file removed first
  so `inst()`'s existence check doesn't skip it.)

```sh
# ── seed_sqlite ───────────────────────────────────────────
# Seeds an empty SQLite database file at the resolved path.
# Called from the database-dispatch case arm above.
# $1 = resolved database path from the probe's
#      database.path field.
seed_sqlite() {
    _path=$1
    _template=/usr/share/torrust/default/database/sqlite3.db

    # Empty path: we only reach this function from the
    # sqlite) dispatch arm, so an empty path means the probe
    # decoded a sqlite URL but could not extract a file path
    # — a probe bug, not a no-op.
    if [ -z "$_path" ]; then
        echo "ERROR: probe reported sqlite driver but" \
             "database.path is empty — possible probe bug" >&2
        exit 1
    fi

    # In-memory: no file to seed.
    if [ "$_path" = ":memory:" ]; then
        echo "INFO: SQLite :memory: — no database file to seed" >&2
        return 0
    fi

    # Relative path: CWD is / inside the container; refuse
    # to guess an absolute location.
    case $_path in
        /*) ;; # absolute — fall through
        *)
            echo "WARN: relative SQLite path '$_path';" \
                 "not seeding — application will create on" \
                 "first connect if mode=rwc" >&2
            return 0
            ;;
    esac

    # Absolute path, file exists and non-empty: leave alone.
    if [ -s "$_path" ]; then
        return 0
    fi

    # Absolute path, zero-byte file: remove so inst() does
    # not skip it.
    if [ -e "$_path" ] && [ ! -s "$_path" ]; then
        rm -f "$_path"
    fi

    # Volumes-only auto-mkdir for the parent directory.
    _dir=$(dirname "$_path")
    if [ ! -d "$_dir" ]; then
        case "$_dir" in
            /etc/torrust/index|/etc/torrust/index/*|\
            /var/lib/torrust/index|/var/lib/torrust/index/*|\
            /var/log/torrust/index|/var/log/torrust/index/*)
                mkdir -p "$_dir"
                chown torrust:torrust "$_dir"
                chmod 0750 "$_dir"
                ;;
            *)
                echo "ERROR: database path $_dir is outside the" \
                     "volumes the entry script manages." >&2
                echo "       Pre-create it with appropriate" \
                     "ownership, or use a path under" \
                     "/var/lib/torrust/index/." >&2
                exit 1
                ;;
        esac
    fi

    # Delegate to inst() — handles install -D, mode, ownership.
    inst "$_template" "$_path"
}
```

**Verification.** Two layers:

- The helper's own unit tests cover URL parsing and
  auth-source resolution exhaustively.
- An integration test against the entry script covers the
  five seeding outcomes (`:memory:` skip, relative skip,
  non-empty untouched, zero-byte seeded, missing-under-volume
  seeded) and the missing-outside-volumes error.

**TOML-invisibility gap closed.** Because the helper is the
application's loader, an operator who provides
`connect_url` (or `auth.private_key_path`) only in a mounted
TOML is now visible to the script. The "known gap" in
[ADR D3](009-container-infrastructure-refactor.md#d3--single-source-of-truth-for-auth-key-paths)
no longer applies to the env-var-only path *or* the
TOML-only path; both work identically.

### 7.3 Containerfile and merge-conflict implications

The new helper binary is built and copied alongside
`torrust-index-health-check` in the existing `test` /
`test_debug` stages, and dropped into `/usr/bin/` of both
final images by the same `COPY --from=test ...` lines. No
additional stage is required; the workspace `cargo build` /
`cargo nextest` invocations already in those stages pick up
the new crate automatically.

**`jq` in the runtime image.** The probe consumption (§7.2)
shares the `jq` binary that Phase 2 (§2.2 step 4) added to
both runtime bases for auth-keypair JSON parsing. No
additional `jq_donor` stage is needed in Phase 7 — the
contract (root-only invocation during the entry script's
pre-drop phase, not a busybox applet, not on the
unprivileged user's PATH) is unchanged. If Phase 7 lands
ahead of Phase 2 for any reason, move the `jq_donor`
stage's introduction to whichever phase lands first; the
dependency is on the binary being present, not on which
phase introduced it.

Phase 4's runtime base split (§4.2) does not need to know
about the helper specifically — it lands wherever the test
stage's release binaries land. The merge-conflict notes at
the top of this document already cover the
`Containerfile`-touches-twice case.

### 7.4 Document the new contract

Update `docs/containers.md`: list every env var the entry
script reads, in what order, and what each defaults to.
Document the `jq` dependency and its provenance.

**`TORRUST_INDEX_DATABASE_DRIVER` scope narrowing.**
Document that `TORRUST_INDEX_DATABASE_DRIVER` is now a
Containerfile-level selector for which default TOML to
seed — it no longer drives runtime database decisions.
Phase 7 replaces the entry script's database dispatch
with the config probe's `database.driver` field (derived
from `connect_url`'s URL scheme). Operators who script
around `TORRUST_INDEX_DATABASE_DRIVER` expecting it to
control runtime behaviour must update their scripts to
supply `connect_url` instead. Note the taxonomy difference:
the env var uses `sqlite3` / `mysql`; the probe emits
`sqlite` / `mysql` / `mariadb`.

---

## Phase 8 — Compose Split (D1)

**Files.** `compose.yaml`, new `compose.override.yaml`,
`contrib/dev-tools/container/e2e/`, `Makefile`.

### 8.1 Restructure `compose.yaml` as production-shaped

- Remove `mailcatcher` (the *service* and the
  `index.depends_on: [..., mailcatcher, ...]` reference).
- Audit the base file (and `compose.yaml`-adjacent env
  blocks in `src/`) for every other reference to mail/SMTP
  and assess whether they need updating or removal. The
  `src/` mailer implementation is legitimate application code
  and stays; the audit targets stale env-var names,
  `depends_on` entries, and config-override keys that would
  wire the prod baseline back to the dev mailcatcher.

  Use two complementary greps; neither alone is sufficient:

  ```sh
  # 1. Casual / legacy spellings — catches dev-shaped names
  #    (mailcatcher, MAILER_HOST, smtp_*) that pre-date the
  #    config-override convention. The -i flag catches
  #    capitalisation variants (Mailcatcher, SMTP, Smtp)
  #    without expanding the alternation.
  grep -nriE 'mailcatcher|MAILER|SMTP|smtp_' compose.yaml src/

  # 2. The override-prefix form — catches every variant the
  #    application's env-var override mechanism produces,
  #    regardless of which sub-key (SERVER, PORT, USERNAME,
  #    PASSWORD, FROM, REPLY_TO, …) is set. The `__` is
  #    deliberate — it's the override-mechanism's section
  #    delimiter, so a single `MAIL__` substring catches the
  #    whole namespace without enumerating sub-keys.
  grep -nrE 'TORRUST_INDEX_CONFIG_OVERRIDE_MAIL__' compose.yaml src/
  ```

  Both are heuristics — variants like `mail_host` or `MTA_*`
  would still slip through and need manual review — but
  together they cover the two shapes that actually appear in
  this codebase. Run them as part of the Phase 8 PR
  description so the audit trail is visible, and add the
  `mailcatcher` grep as a CI lint on `compose.yaml` so
  future re-introductions are caught automatically.
  In particular,
  confirm no `index.environment:` block in the new
  prod-shaped baseline still names `mailcatcher` (e.g.
  `..._MAIL__SMTP__SERVER=mailcatcher`) — the grep catches
  this, but spelling the purpose out avoids the reader
  having to infer it.
- TOML-level mail blocks pointing at `mailcatcher` are
  stripped by §5.1; operators of the production-shaped
  baseline supply mail config via
  `TORRUST_INDEX_CONFIG_OVERRIDE_MAIL__*`.
- Remove `tty: true` from `index` / `tracker`.
- Reference credentials via bare `${VAR}` (no default, no
  `:?required` assertion).
- Bind all ports to `127.0.0.1` except the index API.

**Scope of the bare-`${VAR}` rule.** Applies to *credentials*
(`..._TRACKER__TOKEN`, `..._DATABASE__CONNECT_URL`, MySQL
root password, etc.) and to environment-coupled hostnames
(`..._MAIL__SMTP__SERVER`). The tracker service's
`TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN`
follows the same rule: bare `${VAR}` in the prod baseline,
with a `:-MyAccessToken` fallback added in
`compose.override.yaml` for the dev sandbox.
`make up-prod` validates it alongside the index-side
credentials.

The rule does **not** apply to operator selectors with a
sensible cross-environment default —
`TORRUST_INDEX_DATABASE_DRIVER` and friends keep their
`${VAR:-sqlite3}` defaults so the documented
`docker compose up` flow keeps working. Intra-compose
service-name DNS (`tracker`, `mysql`, `mailcatcher`) is also
excluded: these are compose-network identifiers resolved by
Docker's embedded DNS, not operator-supplied hostnames.

**Why not `${VAR:?required}` here.** Compose interpolates
each file independently before merging; a `:?required`
assertion in the base file fails *during base parse*, before
the override's defaults can be considered. Validation is
therefore deferred to the `make up-prod` wrapper.

**Defence in depth against empty-string substitution.**
Bare `${VAR}` with no fallback means a developer who runs
`docker compose -f compose.yaml up` (explicitly bypassing
the override) gets empty-string substitution rather than a
compose-level error. Three layers catch this before it
causes silent runtime misbehaviour:

1. **Config probe (§6.1)** — the principled gate. Runs
   inside the container at startup regardless of how compose
   was invoked. An empty `connect_url` fails
   `url::Url::parse` (exit 3); an empty `tracker.token` is
   rejected explicitly (exit 4). This layer cannot be
   bypassed.
2. **MySQL entrypoint** — the official MySQL image refuses
   to start with an empty root password.
3. **`make up-prod` wrapper (§8.3)** — fail-fast
   convenience. Validates required env vars *before*
   container start so the operator gets a single clear
   error rather than waiting for each container to boot and
   fail individually.

The `make up-prod` wrapper is defence in depth, not the
only line. The config probe is the container-boundary
validator.

### 8.2 Add `compose.override.yaml`

Auto-loaded by Compose v2 with the dev sandbox extras:
`mailcatcher` service, `tty: true` on relevant services,
permissive credential defaults via `${VAR:-...}`, optional
dev-only port exposures.

**Use long-form `depends_on:` to re-attach `mailcatcher`.**
Compose v2 merges `depends_on` additively *only* in
long-form. Short-form replaces rather than extends and would
silently drop the base's `tracker` / `mysql` dependencies.
The base file's own `depends_on` must also use long-form for
the additive merge to work — if the base uses short-form the
override silently replaces it:

```yaml
services:
  index:
    depends_on:
      mailcatcher:
        condition: service_started
  mailcatcher:
    image: dockage/mailcatcher:0.8.2
    # ...
```

### 8.3 Add Make targets

Introduces a top-level `Makefile` (none exists today; the
only current `Makefile` in the tree is the unrelated
[`contrib/dev-tools/su-exec/Makefile`](../contrib/dev-tools/su-exec/Makefile)).
Validation logic uses POSIX `sh` with `set -u` and explicit
`: "${VAR:?message}"` per required variable, so the file is
dependency-free and shellcheck-clean.

- **`make up-dev`** — plain `docker compose up` (override
  auto-loaded, dev defaults apply). No validation.
- **`make up-prod`** — validates required env vars are set,
  then runs `docker compose --file compose.yaml up` (override
  excluded). Produces a clear error on missing variables.
  The target must propagate `docker compose`'s own exit code
  when validation passes but compose itself fails, so that
  `make up-prod` never reports success while a service
  crashes in the background.

  A trivial recipe gives exit-code propagation for free
  (Make already propagates the recipe's exit code):

  ```make
  .PHONY: up-dev up-prod _validate-prod-env

  # Overridable so acceptance tests (and operators with a
  # non-default layout) can point at an alternate file
  # without filesystem manipulation. The `?=` form lets
  # `make up-prod COMPOSE_FILE=path/to/other.yaml` work; an
  # unset value falls back to the in-tree baseline.
  COMPOSE_FILE ?= compose.yaml

  up-dev:
  	docker compose up

  _validate-prod-env:
  	@sh -uc '\
  	  : "$${TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN:?required}" && \
  	  : "$${TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL:?required}" && \
  	  : "$${TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN:?required}" && \
  	  if grep -q "^[[:space:]]*mysql:" $(COMPOSE_FILE); then \
  	    : "$${MYSQL_ROOT_PASSWORD:?required}"; \
  	  fi'
  # NOTE: The MySQL check is name-coupled to the compose
  # service (`mysql:`). This is acceptable because it is
  # defence-in-depth only — the config probe (exit 3 on
  # empty connect_url) and the MySQL entrypoint (rejects
  # empty root password) are the authoritative gates.

  up-prod: _validate-prod-env
  	docker compose --file $(COMPOSE_FILE) up -d --wait
  ```

  Do not wrap the compose invocation in `if`/`||`/shell
  functions that might accidentally swallow the status.
  The `-d --wait` flags are deliberate: `up` without `-d`
  blocks indefinitely (which would hang Acceptance #4 and
  any CI invocation), and `--wait` makes Compose return
  non-zero if a service fails its healthcheck within the
  default timeout — the failure mode operators actually
  care about.

  Note on the `COMPOSE_FILE` make-variable vs.
  `COMPOSE_FILE` env var: Compose's own `COMPOSE_FILE` env
  var is *ignored* when `--file` is set on the command
  line, so the make-variable is the only way to redirect
  the recipe. The `_validate-prod-env` target's
  `grep ... $(COMPOSE_FILE)` honours the same override so
  the MySQL-credential check inspects the file that will
  actually be loaded.

### 8.4 Update E2E scripts

Audit every script under
[`contrib/dev-tools/container/e2e/`](../contrib/dev-tools/container/e2e/)
for hard-coded `docker-compose` (v1) invocations; migrate to
`docker compose` (v2). Scripts running a dev profile rely on
the override file; scripts running production-shaped
compositions pass `--file compose.yaml` explicitly.

### 8.5 Update `docs/containers.md`

Describe the two files with a clear "for development" / "for
deployment template" distinction.

---

## Phase 9 — Documentation & Audit (D8, D9 docs part)

**Files.** `docs/containers.md`, `README.md`,
`contrib/dev-tools/su-exec/AUDIT.md`, `CHANGELOG.md`.

### 9.1 Document the test-stage decision

Add a subsection in `docs/containers.md` explaining the
trade-off: the image build is gated on the test suite
passing, so flaky tests block image production until fixed.
This is deliberate — the alternative ("skip tests" build
path) would inevitably be used in production. Operators
should know it's there and have a clear escalation path when
the suite goes red on something unrelated.

### 9.1.1 Update README quickstart

Review the `README.md` quickstart / "Getting Started" section
for any inline `connect_url`, `token`, or `docker compose up`
examples that assume the pre-D2 defaults. Update to reflect
the mandatory-`connect_url` / mandatory-`token` change and
the compose baseline + override split introduced in Phase 8.

### 9.1.2 Update CHANGELOG

Add an entry to `CHANGELOG.md` under the appropriate
version section summarising the container infrastructure
refactor: mandatory `connect_url` / `tracker.token`,
compose split, runtime-base split, helper-crate
extractions, the new `torrust-index-config-probe`
entrypoint, the `[net.tsl]` → `[net.tls]` wire-key
rename (breaking change for operator TOMLs and JSON API
consumers), the PEM+PATH mutual-exclusion enforcement
(D3), and the narrowed scope of
`TORRUST_INDEX_DATABASE_DRIVER` (now a TOML-selection
knob only, no longer a runtime database dispatcher).

### 9.2 Internal code audit for vendored `su-exec`

Add `contrib/dev-tools/su-exec/AUDIT.md` with:

- **Provenance.** Upstream URL, commit/tag the vendored copy
  was taken from, date vendored, and SHA-256 of `su-exec.c` at
  that time.
- **Choice rationale.** Why `su-exec` rather than `gosu` (Go
  runtime, larger binary) or `setpriv` (util-linux dependency,
  not on the lean distroless base).
- **Audit log.** A table of dated entries (append-only,
  oldest first): reviewer, date,
  commit of this repo at review time, SHA-256 of `su-exec.c`,
  scope of review, conclusions. Each entry must contain a
  `SHA-256: <64-hex-chars>` line (the structured marker the
  CI extraction relies on — see Acceptance §8). Initial entry
  created in this phase. The acceptance check (Criterion #8) relies on the
  last SHA-256 in the `## Audit Log` section being the most
  recent; preserving chronological order is required.
- **Re-audit triggers.** Any change to the vendored `.c`
  file, or any CVE against historic `su-exec` versions.

  - **File-change trigger** — CI compares the SHA-256 of
    `su-exec.c` against the value recorded in `AUDIT.md`'s
    most recent entry; mismatch fails the build until a fresh
    audit entry is added.
  - **CVE trigger** — manual review duty. No automated CVE
    feed for an unmaintained project of this size is worth
    the false-positive cost.

  **Why no calendar trigger.** See ADR D8 — the rationale
  (unchanged static code has no time-dependent decay) lives
  there and is not repeated here.

  The file-change CI check lives alongside the existing
  container-image CI so it cannot be bypassed by the normal
  PR flow.

There is deliberately *no* "refresh procedure" section.
Upstream `su-exec` has not released in years; if a re-vendor
is ever needed, it will be a manual diff-and-review exercise.

---

## Acceptance Criteria — implementation detail

The ADR's [Acceptance Criteria](009-container-infrastructure-refactor.md#acceptance-criteria)
are the canonical list. The exact `docker run` and `grep`
commands used to verify them are below.

### 1. Runtime base split

All commands below override `--entrypoint` (bypassing
`entry.sh`'s privilege drop) and rely on the default
container user being root (no `USER` directive in the image)
so that `/bin/sh` — mode `0700 root:root` — is invokable for
the assertion runner itself. The negative-permission tests
then re-run with `--user 1000` to confirm the unprivileged
`torrust` user is denied.

Note: `--user 1000` bypasses the entry script's `adduser`,
so UID 1000 has no `/etc/passwd` entry in these tests.
Docker passes through numeric UIDs without resolution;
we are testing file-mode bits, not user resolution.

```sh
set -eu

# /busybox/ should be absent entirely (not just access-
# controlled). Asserting non-existence directly via `test -e`
# keeps the truth value clean and avoids relying on OCI exec
# error messages, which differ between runtimes.
#
# `test` here is a busybox `sh` built-in (POSIX §2.14), not
# an external applet — that's why it's deliberately absent
# from §4.4's curated symlink loop. Do not "fix" this by
# switching to an external probe; the equivalent `[ ! -e
# /busybox ]` is also a builtin and would work, but the
# `test` form reads more naturally with the leading `!`.
docker run --rm --entrypoint=/bin/sh release-image \
    -c '! test -e /busybox'
# Should exit 0: /busybox does not exist.

# /bin/sh is a symlink to /bin/busybox; symlink mode bits are
# irrelevant — access checks dereference to the target.
# The 0700 root:root on /bin/busybox is what denies the
# unprivileged user.
docker run --rm --user 1000 --entrypoint=/bin/sh release-image -c 'echo pwned'
# Should fail: permission denied.

# /bin/busybox should be root-only.
docker run --rm --user 1000 --entrypoint=/bin/busybox release-image sh -c 'echo pwned'
# Should fail: permission denied.

# /bin/su-exec should be root-only.
docker run --rm --user 1000 --entrypoint=/bin/su-exec release-image root sh
# Should fail: permission denied.
```

### 2. Credentials stripped

```sh
set -eu

# No literal dev credentials anywhere.
# (`secret_password` is a substring match catching the MySQL
# root password literal `root_secret_password`.)
! grep -rE '(secret_password|MyAccessToken)' share/default/config/

# No connect_url or token keys at all.
! grep -rE '^[[:space:]]*(connect_url|token)[[:space:]]*=' share/default/config/

# No mail config pointing at dev hosts.
! grep -rE 'mailcatcher' share/default/config/

# No residual [mail.*] blocks (the literal-mailcatcher grep
# above would still pass on an empty [mail.smtp] block left
# behind by an incomplete strip).
! grep -rE '^\[mail(\.|\])' share/default/config/
```

### 3. Dev compose works

```sh
set -eu

# Start the dev environment (override auto-loaded) and wait
# for the index to become healthy. --wait exits non-zero if
# any service fails its healthcheck within the timeout.
docker compose up -d --wait --wait-timeout 60

# Verify the index API is reachable.
curl -sf http://localhost:3001/health_check

docker compose down
```

### 4. Prod compose validates

```sh
set -eu

# With no credential env vars set, `make up-prod` must fail
# *before* any container starts — i.e. the Make target's
# validation shell, not a compose-level or container-level
# error.
unset TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN \
      TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL \
      MYSQL_ROOT_PASSWORD 2>/dev/null

output=$(make up-prod 2>&1) && {
    echo "FAIL: make up-prod succeeded with no credentials set" >&2
    exit 1
}
# The error should name at least one missing variable.
echo "$output" | grep -qE 'TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN|TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL'

# When validation passes, compose's own exit code must
# propagate. Supply valid env vars but redirect the recipe
# to a nonexistent compose file via the COMPOSE_FILE
# make-variable (§8.3) so compose itself fails fast.
#
# The make-variable is required: Compose's own COMPOSE_FILE
# env var is ignored when `--file` is on the command line,
# so `COMPOSE_FILE=... make up-prod` would silently load the
# real baseline and the test would not exercise the
# propagation path it claims to test.
export TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN=x
export TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL=sqlite::memory:
export TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN=x
export MYSQL_ROOT_PASSWORD=x

# `_validate-prod-env`'s mysql check greps the chosen
# compose file; since the file doesn't exist, that grep
# would fail and the validation step would fail with a
# misleading error. Pre-create an empty file so validation
# passes (no `mysql:` line → MYSQL_ROOT_PASSWORD check
# skipped) and only the actual `docker compose up` call
# fails. Clean up on exit.
empty_compose=$(mktemp --suffix=.yaml)
trap 'rm -f "$empty_compose"' EXIT

output=$(make up-prod COMPOSE_FILE="$empty_compose" 2>&1) && {
    echo "FAIL: make up-prod succeeded with empty compose file" >&2
    exit 1
}
```

### 5. Helper-binary dep closures

```sh
set -eu

# Neither helper may depend on an HTTP client, async runtime,
# or TLS stack. The `--prefix none` flag is essential:
# without it, transitive deps are indented with `├──`/`└──`
# glyphs and a `^`-anchored alternation would only catch the
# root crate itself, missing the very transitive
# re-introduction the check is meant to prevent.
#
# `( |$)` is used instead of `\b` because `\b` is GNU-grep-
# only; `( |$)` works on both GNU and BSD grep without
# changing the match semantics here.
forbidden='^(reqwest|tokio|tokio-[a-z0-9_-]+|hyper|hyper-[a-z0-9_-]+|rustls|rustls-[a-z0-9_-]+|native-tls|openssl|openssl-[a-z0-9_-]+)( |$)'

for crate in torrust-index-health-check torrust-index-auth-keypair torrust-index-config-probe; do
    cargo tree -p "$crate" -e normal --prefix none \
      | grep -Eq "$forbidden" && {
        echo "FAIL: $crate pulls in a forbidden dependency" >&2
        exit 1
      }
done
exit 0
```

These assertions are meaningful only because Phases 2 and
6 give each helper its own crate; `cargo tree` scopes to a
*package*, so a `--bin` filter on the root crate would be a
no-op (the root crate depends on `reqwest`/`tokio`
independently for the server). The P9 baseline (`clap`,
`tracing`, `tracing-subscriber`, `serde`, `serde_json`) and
each crate's domain deps (`rsa` for auth-keypair, `figment` +
`toml` for config-probe) are implicitly permitted — they do
not appear in the exclusion regex. The regex is the single
source of truth; no per-crate positive allowlists.

### 6. Helper JSON + TTY contract

Every helper binary refuses to write machine-readable output
to a TTY (P8) and emits structured JSON on stdout when piped
(P9). A single loop covers all helpers:

```sh
set -eu

for bin in torrust-index-health-check \
           torrust-index-auth-keypair \
           torrust-index-config-probe; do

    # ── TTY refusal (P8) ─────────────────────────────────
    # Run with a pseudo-TTY allocated to stdout. The binary
    # must exit 2 before producing any stdout output.
    # `-t` gives the container a pty on stdout (what we want
    # to trigger the TTY guard). `2>/dev/null` discards the
    # *host-side* stderr so diagnostic noise doesn't pollute
    # the test runner; the exit code is the real signal.
    tty_out=""
    rc=0
    tty_out=$(docker run --rm -t --entrypoint="/usr/bin/$bin" release-image \
        2>/dev/null) || rc=$?
    [ "$rc" -eq 2 ] || {
        echo "FAIL: $bin did not exit 2 on TTY (got $rc)" >&2
        exit 1
    }
    [ -z "$tty_out" ] || {
        echo "FAIL: $bin emitted output before TTY refusal" >&2
        exit 1
    }

    # ── JSON stdout (P9) ─────────────────────────────────
    # Run with stdout piped. The success-path output must be
    # valid JSON. Each helper has a different shape of
    # "minimum invocation that exercises the success path":
    #
    #   health-check  — point at a TCP listener that exists.
    #                   The image's own /health_check endpoint
    #                   is the obvious choice but requires the
    #                   full app to be running.  A sidecar
    #                   listener is not viable in a single
    #                   docker-run invocation, so we accept
    #                   the connection-refused path (exit ≠ 0)
    #                   here and rely on integration tests for
    #                   the 200 path.  The format check still
    #                   fires if the binary emits anything on
    #                   stdout.
    #
    #   auth-keypair  — no args needed; runs to completion
    #                   and emits a JSON object on stdout.
    #                   Must exit 0.
    #
    #   config-probe  — needs a complete-enough Settings to
    #                   pass deserialisation. The image ships
    #                   defaults at
    #                   /usr/share/torrust/default/config/;
    #                   the SQLite container default plus
    #                   overrides for the mandatory fields
    #                   (connect_url, tracker.token —
    #                   stripped from the TOML by §5.1) is
    #                   the smallest spec that exercises the
    #                   success path. Must exit 0.
    case $bin in
        *health-check)
            # Connection-refused path: success-path JSON is
            # not produced, so we only assert "if any output,
            # it must be JSON". Full success-path coverage
            # lives in the helper's own integration tests.
            out=$(docker run --rm --entrypoint="/usr/bin/$bin" \
                release-image "http://localhost:1/nope" 2>/dev/null) || true
            if [ -n "$out" ]; then
                printf '%s' "$out" | jq empty || {
                    echo "FAIL: $bin stdout is not valid JSON" >&2
                    exit 1
                }
            fi
            ;;
        *auth-keypair)
            out=$(docker run --rm --entrypoint="/usr/bin/$bin" \
                release-image 2>/dev/null)
            # Must succeed and emit a JSON object with the
            # documented shape (§2.2 step 3).
            printf '%s' "$out" | jq -e '.private_key_pem and .public_key_pem' \
                >/dev/null || {
                echo "FAIL: $bin did not emit the expected JSON shape" >&2
                exit 1
            }
            ;;
        *config-probe)
            out=$(docker run --rm --entrypoint="/usr/bin/$bin" \
                -e TORRUST_INDEX_CONFIG_TOML_PATH=/usr/share/torrust/default/config/index.container.sqlite3.toml \
                -e TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL="sqlite::memory:" \
                -e TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="test" \
                release-image 2>/dev/null)
            # Must succeed (exit 0) and emit the documented
            # shape (§6.1). If this assertion starts failing
            # because the schema gained a new mandatory field,
            # the override list above needs the new field —
            # don't relax the assertion.
            printf '%s' "$out" | jq -e '.schema and .database and .auth' \
                >/dev/null || {
                echo "FAIL: $bin did not emit the expected JSON shape" >&2
                exit 1
            }
            ;;
    esac
done
```

**Note on the success-path assertions.** The
`auth-keypair` and `config-probe` arms assert exit 0 *and* a
shape-check on the JSON, not just "stdout parses as JSON".
This is deliberate: an earlier draft tolerated empty stdout
(treating exit ≠ 0 as a no-op), which silently never
exercised the success path — a regression that stripped a
required field from the shipped TOML would have passed the
test. The shape-check via `jq -e '.field and .other'` fails
both on missing fields *and* on empty input (`jq -e` exits
non-zero when the filter result is null or false).

The `health-check` arm remains lenient because exercising
the 200-response path inside a single `docker run` requires
either a running app or a sidecar listener — out of scope
for this acceptance check. The helper's own integration
tests (§2.1 step 4) cover the 200 path.

### 7. Documentation complete

The entry script maintains a canonical env-var manifest as a
comment block near the top of the file. This block is the
single source of truth for Criterion #7 — it captures both
literal references *and* dynamically constructed names (e.g.
`AUTH__PRIVATE_KEY_PEM` built via `${uc_pair}_PEM`) that a
naive grep would miss.

The block format in `share/container/entry_script_sh`:

```sh
# ENTRY_ENV_VARS:
#   TORRUST_INDEX_CONFIG_TOML_PATH
#   TORRUST_INDEX_DATABASE_DRIVER
#   TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PRIVATE_KEY_PEM
#   TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PRIVATE_KEY_PATH
#   TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PUBLIC_KEY_PEM
#   TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PUBLIC_KEY_PATH
#   TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN
#   TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL
#   USER_ID
#   API_PORT
#   IMPORTER_API_PORT
#   DEBUG
#   TZ
#   RUNTIME
# END_ENTRY_ENV_VARS
```

The CI check extracts from that block rather than
grep-scraping code lines:

```sh
set -eu

# Extract the canonical env-var list from the manifest block.
# The sed range matches lines between the two sentinel
# comments; the grep strips the comment prefix and extracts
# bare variable names.
vars=$(sed -n '/^# ENTRY_ENV_VARS:/,/^# END_ENTRY_ENV_VARS/p' \
         share/container/entry_script_sh \
       | grep -oE '[A-Z][A-Z0-9_]+' \
       | sort -u)

[ -n "$vars" ] || {
    echo "FAIL: ENTRY_ENV_VARS block not found in entry_script_sh" >&2
    exit 1
}

missing=0
for v in $vars; do
    grep -q "$v" docs/containers.md || {
        echo "MISSING from docs/containers.md: $v" >&2
        missing=1
    }
done

# The two-file compose relationship must be documented.
grep -q 'compose\.override\.yaml' docs/containers.md || {
    echo "MISSING: compose.override.yaml documentation" >&2
    missing=1
}

[ "$missing" -eq 0 ]
```

The manifest block is maintained by the developer who
modifies the entry script. The CI check fails loudly when
a var appears in the block but not in the docs — and when
the block itself is missing or empty.

### 8. Audit record exists

```sh
set -eu

audit=contrib/dev-tools/su-exec/AUDIT.md

# File exists and is non-empty.
test -s "$audit"

# Contains the required sections. Each grep exits non-zero
# on missing content, failing the script under `set -e`.
grep -qi 'provenance'        "$audit"
grep -qi 'rationale'         "$audit"
grep -qi 'SHA-256'           "$audit"
grep -qE '[0-9]{4}-[0-9]{2}-[0-9]{2}' "$audit"   # at least one dated entry

# CI check: the SHA-256 recorded in the most recent audit
# entry matches the current file. Audit-log entries are
# append-only (oldest first) and each entry must contain a
# structured `SHA-256: <hex>` line. The last such line in
# the "## Audit Log" section is the most recent. Using the
# structured marker (rather than a bare 64-hex grep) avoids
# false matches against commit hashes, historical SHAs in
# free-form notes, or other hex strings.
recorded=$(sed -n '/^## Audit Log/,$ { s/^SHA-256: \([0-9a-f]\{64\}\)$/\1/p; }' "$audit" \
  | tail -1)
actual=$(sha256sum contrib/dev-tools/su-exec/su-exec.c | cut -d' ' -f1)
[ "$recorded" = "$actual" ]
```

### 9. Refuse-if-root guard

```sh
set -eu

# USER_ID=0 must be rejected before the application starts.
output=$(docker run --rm -e USER_ID=0 release-image 2>&1) && {
    echo "FAIL: entry script accepted USER_ID=0" >&2
    exit 1
}
echo "$output" | grep -qi 'root'

# A valid low UID (e.g. 500, common in rootless Podman)
# must be accepted. The container won't fully start without
# config, but the USER_ID guard must pass — verify by
# checking for a *later* error (e.g. config probe failure)
# that proves the script got past the guard.
output=$(docker run --rm -e USER_ID=500 release-image 2>&1) || true
# Should not contain the "refusing to run as root" error.
! echo "$output" | grep -qi 'refusing to run as root'
# Positive: the script should have reached the adduser or
# config-probe stage, proving the USER_ID guard passed.
# Match against concrete error text from later stages —
# adduser output, the probe binary name, or the serde
# "missing field" error the probe emits when config is
# absent.
echo "$output" | grep -qiE 'adduser|torrust-index-config-probe|missing field|connect_url'
```
