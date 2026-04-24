# ADR-T-009: Container Infrastructure Refactor

**Status:** Implemented (Phases 1–9 complete)
**Date:** 2026-04-19
**Supersedes:** Earlier `ADR-T-009` draft ("Container
Infrastructure Hardening") whose tactical S-N items were
merged without a written ADR file. Those items are summarised
in [Prior Work](#prior-work) and are not re-litigated here.
**Relates to:**
[ADR-T-007](007-jwt-system-refactor.md) (auth key generation
performed by the entry script).
**Implementation plan:**
[adr/009-implementation-plan.md](009-implementation-plan.md)
— phases, file lists, snippets, dependency graph, and
merge-conflict notes live there. This ADR records the
decisions; the plan records how to land them.

## Context

The container infrastructure consists of:

- A multi-stage [`Containerfile`](../Containerfile) that
  builds, tests, and packages the index.
- A single [`compose.yaml`](../compose.yaml) that orchestrates
  the index together with `tracker`, `mysql`, and
  `mailcatcher`.
- A POSIX entry script
  ([`share/container/entry_script_sh`](../share/container/entry_script_sh))
  that prepares the runtime, generates auth keys on first
  boot, and drops privileges via vendored `su-exec`
  ([`contrib/dev-tools/su-exec/su-exec.c`](../contrib/dev-tools/su-exec/su-exec.c)).
- Default configurations under
  [`share/default/config/`](../share/default/config/) shipped
  inside the image at `/usr/share/torrust/default/config/`.
- A small `health_check` binary
  ([`src/bin/health_check.rs`](../src/bin/health_check.rs))
  invoked by the runtime `HEALTHCHECK`.
- E2E orchestration scripts under
  [`contrib/dev-tools/container/e2e/`](../contrib/dev-tools/container/e2e/)
  and operator documentation in
  [`docs/containers.md`](../docs/containers.md).

The previous round of work (see [Prior Work](#prior-work))
brought the infrastructure to a defensible baseline by fixing
a long list of concrete bugs. What remains is *structural*:
several pieces of the design carry assumptions that no longer
match how the project is used, and continuing to layer fixes
onto those assumptions will keep producing the same shapes of
bug. This ADR records the structural decisions; the
[Appendix](#appendix-diagnostic-detail) catalogues the
diagnostic items (`R1`–`R10`) that motivated each one.

### Prior Work

The previous draft of ADR-T-009 ("Container Infrastructure
Hardening") catalogued and resolved a set of tactical issues
labelled S1 through S12 — entry-script tracing gated on
`DEBUG=1`, MySQL healthcheck repair, dev-only port rebinding,
restart policies, base-image upgrade `cc-debian12` →
`cc-debian13`, and similar. Those changes are merged and are
not re-litigated here.

## Vision

After this ADR the container subsystem is composed of three
layers with deliberately separate concerns.

**The image** is two parallel artifacts built from one
Containerfile: a `release` image on a lean distroless base
whose only privileged-user utilities are a curated, root-only
busybox subset and `su-exec`; and a `debug` image on the same
base's `:debug` variant that retains user-accessible developer
affordances. In the release image, the unprivileged `torrust`
user — under which the application actually runs — has no
usable shell and no access to root utilities; the debug image
retains user-accessible developer affordances by design.
A separate workspace crate,
with no transitive HTTP/TLS dependencies, supplies the
health-check binary.

**The configuration** is the operator's responsibility, not
the image's. Shipped TOML defaults declare structure but no
credentials, no `connect_url`, no environment-coupled
hostnames, and no environment-coupled paths (notably
auth-key paths, which the entry script owns and exports).
The schema makes `database.connect_url` mandatory,
so a missing value fails at parse time with a precise serde
error rather than silently falling back to a hidden default.
The entry script reads the same
`TORRUST_INDEX_CONFIG_OVERRIDE_*` env vars the application
reads, and is the single source of truth for any path it
materialises (notably auth-key paths) — coordination with the
application happens by the script *setting* the override
before exec, not by two files agreeing on a constant.

**The orchestration** is two compose files: a
production-shaped baseline that references credentials via
bare `${VAR}` and binds dev ports to localhost, and an
auto-loaded `compose.override.yaml` that re-introduces the dev
sandbox (mailcatcher, permissive defaults, tty). A
`make up-prod` wrapper validates required env vars before
invoking compose; plain `docker compose up` remains the
zero-friction dev workflow.

## Principles

The decisions below follow from a small set of invariants the
container subsystem commits to:

- **P1.** Shipped defaults contain no credentials and no
  environment-coupled values.
- **P2.** Runtime configuration is runtime; build
  configuration is build. Neither leaks into the other.
- **P3.** In the release image, the unprivileged runtime user
  has no usable shell and no access to root utilities.
  (The debug image deliberately retains user-accessible
  affordances — see [D4](#d4--two-parallel-runtime-bases-root-only-utilities-in-release).)
  Privilege drop is irreversible from the application's side
  under the documented runtime configuration (no
  `CAP_SETUID`, GID set excludes 0).
- **P4.** The schema enforces required fields. Bootstrap does
  not re-validate what serde has already proven.
- **P5.** Where two components must agree on a value (path,
  port, credential), exactly one of them owns it and tells the
  other; they do not independently maintain a shared constant.
- **P6.** The compose baseline is production-shaped; dev
  affordances are an additive override layer, never a
  subtraction from the baseline.
- **P7.** Vendored security-sensitive code is treated as code
  we own, with a current internal audit record.
- **P8.** No machine-readable stdout to a TTY. Every helper
  binary that emits structured output (JSON, PEM) on stdout
  refuses to run when stdout is a terminal. The check is
  unconditional — it does not depend on whether the specific
  output is sensitive. Operators who want to see the output
  interactively pipe to `jq`, `less`, or `cat`.
- **P9.** Universal helper conventions. Every helper binary
  links the same baseline crates without exception or
  per-crate justification: `clap` (argv), `tracing` +
  `tracing-subscriber` with `json` feature (stderr
  diagnostics), `serde` + `serde_json` (stdout wire
  format). These are not enumerated in per-crate allowlists.
  On success (exit 0), stdout is one JSON object followed
  by one trailing newline. On failure (exit ≠ 0), stdout is
  empty — the exit code is the sole branch signal for
  callers, and the diagnostic goes to stderr via `tracing`.
  Stderr is always NDJSON `tracing` events regardless of
  exit code. A
  shared `torrust-index-cli-common` library crate provides
  the scaffolding (`refuse_if_stdout_is_tty`,
  `init_json_tracing`, `emit<T: Serialize>`, and a common
  `BaseArgs` with `--debug`).

## Options Considered

### Option A — Status quo plus targeted patches

Patch the highest-severity items (R1, R2, R3) and leave the
rest. Smallest diff. Leaves R4–R10 to keep producing tactical
bugs that re-derive the same structural problems.

### Option B — Focused refactor (this ADR)

Treat the container infrastructure as a single subsystem and
align it around the principles above. Each decision is locally
small but the set is coherent, and each can land
independently. Touches many files; requires coordinated CI and
documentation updates.

### Option C — Full containerisation reset

Rebuild around a different base (Chainguard, Alpine, or
from-scratch with statically linked binaries) and restructure
the Containerfile from scratch. Maximum freedom; discards a
large amount of working, tested infrastructure for marginal
gain. Nothing in the identified problems requires changing the
base image.

## Decision

**Adopt Option B.**

Option A leaves known structural debt that has already proven
willing to come back as new tactical bugs. Option C is
disproportionate. Option B keeps the parts that work
(multi-stage build, `cargo-chef` caching, in-build `nextest`,
distroless runtime, vendored `su-exec`) and corrects the
structural pieces that don't.

The decisions that constitute Option B follow.

### D1 — Split compose into baseline + override

**Follows from:** P6.
**Addresses:** [R1](#r1--composeyaml-conflates-dev-sandbox-and-deployment-template).
**Status:** Landed 2026-04-24 via
[Phase 8](009-implementation-plan.md#phase-8--compose-split-d1).

`compose.yaml` is restructured as a production-shaped baseline
(no `mailcatcher`, no `tty`, credentials referenced as bare
`${VAR}`, dev-only ports on `127.0.0.1`).
`compose.override.yaml` is auto-loaded by Compose v2 and
carries the dev sandbox (mailcatcher service, permissive
`${VAR:-default}` substitutions, tty). A `make up-prod` target
validates required credential env vars and runs compose with
`--file compose.yaml` (override excluded). `make up-dev` is
plain `docker compose up`.

The bare-`${VAR}` rule applies to credentials and
environment-coupled hostnames, not to operator selectors that
have a sensible cross-environment default
(`TORRUST_INDEX_DATABASE_DRIVER` etc.) — those keep their
`${VAR:-sqlite3}` defaults so plain `docker compose up`
continues to work.

### D2 — Strip credentials from defaults; mandatory `connect_url`

**Follows from:** P1, P4.
**Addresses:** [R2](#r2--credentials-embedded-in-shipped-default-configs).
**Status:** Landed 2026-04-24 via
[Phase 5](009-implementation-plan.md#phase-5--schema--credential-strip-d2).

Every file under `share/default/config/` loses its literal
`connect_url`, `token`, and `[mail.smtp]` values. The single
rule "no credentials, no `connect_url`, no environment-coupled
hostnames in shipped defaults" replaces the previous
per-driver mix; SQLite `connect_url` values (no credentials,
but environment-coupled) are stripped for consistency rather
than carved out.

`Database::connect_url` becomes mandatory at the schema level
by dropping `#[serde(default = "...")]` and the surrounding
`impl Default for Database`. A missing value fails at
deserialisation with a precise `missing field 'connect_url'`
error from serde, naming the section. No `check_mandatory_options`
branch is added: the invariant lives in the type, not in a
runtime check that readers must trust ran.

The trade-off is acknowledged: a zero-config
`docker run torrust-index` no longer produces a working
SQLite instance. The simpler enforcement rule is worth the
regression because the zero-config path mostly produced
confusion when operators later tried to migrate to MySQL
and discovered they had been running on an undocumented
SQLite default.

**`tracker.token` sentinel default.** Today
`Tracker::default_token()` in `src/config/v2/tracker.rs`
returns `ApiToken::new("MyAccessToken")` via
`#[serde(default)]`, so stripping `token = "MyAccessToken"`
from shipped TOMLs merely moves the credential from the TOML
to Rust source — the sentinel still takes effect when the
field is absent. The config-resolution probe (§6.1 of the
plan) catches an *empty* token (exit 4) but not the sentinel
default.

**Decision: make `tracker.token` mandatory (option a).**
Drop `#[serde(default = "Tracker::default_token")]` and
remove `Tracker::default_token()`, same pattern as
`connect_url`. This keeps one rule for credentials ("no
defaults") rather than two ("no defaults in TOML, but a
sentinel default in Rust that a probe must know about").
The probe's exit-4 gate for empty tokens remains as defence
in depth; a separate sentinel-rejection branch is no longer
needed because the sentinel is gone. (Note: the exit-4 gate
covers a real gap — `ApiToken`'s `#[derive(Deserialize)]`
constructs the inner `String` directly, bypassing the
`assert!(!key.is_empty())` guard in `ApiToken::new`, so
`token = ""` in TOML silently produces an empty token
unless the probe rejects it.)

### D3 — Single source of truth for auth-key paths

**Follows from:** P5.
**Addresses:** [R3](#r3--entry-script-path-assumptions-conflict-with-config-overrides).
**Status:** Landed 2026-04-24. Phase 6
([`torrust-index-config-probe`](009-implementation-plan.md#phase-6--config-probe-helper))
ships the JSON-shaped resolution surface and Phase 7
([entry-script contract](009-implementation-plan.md#phase-7--entry-script-contract-d3))
wires the script-side mutual-exclusion checks, the
three-way auth-key dispatch (PEM / PATH / container
default), and the
`TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__*_PATH` export so the
entry script is the sole owner of the default key paths.

The `Auth` config struct exposes both `*_PEM` and `*_PATH`
fields per key, and both
`Auth::default_private_key_path` and `default_public_key_path`
return `None` (verified against
[`src/config/v2/auth.rs`](../src/config/v2/auth.rs)). There
is therefore no schema-level default path the entry script
could be byte-equal to; previously the script wrote keys to
its own hardcoded location while the application resolved to
the in-memory ephemeral fallback, silently disagreeing.

The fix is to make the entry script the single source of
truth: when no `*_PEM` and no `*_PATH` is configured for a
given key, the script generates the key at its built-in
location *and exports*
`TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__<PAIR>_PATH` (where
`<PAIR>` is `PRIVATE_KEY` or `PUBLIC_KEY`) to
that same location before `exec`'ing the application. The
two components then agree by construction rather than by
maintenance discipline.

The script makes the per-key decision independently,
enforces mutual exclusion within a single key (PEM + PATH
for the same key is a configuration error — a deliberate
tightening of the application contract at the container
boundary, since
[`src/config/v2/auth.rs`](../src/config/v2/auth.rs)
documents PEM as silently overriding PATH), enforces
pair-completeness (matching the application's existing
invariant in [`src/jwt.rs`](../src/jwt.rs)), and enforces
cross-pair source consistency (both keys must use the same
delivery mechanism — both PEM or both PATH/none — because
the generator emits a matched keypair in one invocation
and cannot produce a single key file).

The mutual-exclusion check covers *all* configuration
sources — env vars, mounted TOML, and mixed-source
combinations. The config-resolution probe (§6.1 of the
plan) emits both raw-presence booleans
(`auth_<pair>_pem_set`, `auth_<pair>_path_set`) and the
resolved source (`auth_<pair>_source`). The script
enforces mutual exclusion on the raw-presence pair
(catching cross-source collisions where, e.g., PEM comes
from a mounted TOML and PATH from an env var) and
dispatches on the resolved source.

The script does not poll env vars to discover the
configuration. Phase 3 of the implementation plan extracts
the application's config parser into a `torrust-index-config`
workspace crate, and a small `torrust-index-config-probe`
helper (§6.1 of the plan) loads `Settings` through that
parser and prints the resolved auth-key sources. The script
calls the helper once and dispatches on its output, so
script and application share the parser by construction —
TOML-only and env-var-only operators are treated identically.

### D4 — Two parallel runtime bases; root-only utilities in release

**Follows from:** P3.
**Addresses:** [R6](#r6--both-debug-and-release-inherit-the-debug-distroless-base).

The previous Containerfile built both `release` and `debug`
on the `:debug` distroless base, leaving the full busybox
tree at `/busybox/` reachable by absolute path even from the
unprivileged user — defeating the curated `/bin/` subset.
After this change, `release` builds on the lean distroless
`cc-debian13` and ships a single `/bin/busybox` (mode
`0700 root:root`) with applet symlinks for the entry script's
needs only; `debug` retains the `:debug` base with
`/busybox/` on PATH so the unprivileged user has the full
applet set. Distroless ships per-architecture images, so the
busybox binary extracted from the `:debug` donor is
native to the build platform; this survives a future
`docker buildx` multi-platform rollout without changes. A `runtime_assets` stage (used by the release
base) bundles the root-only busybox, `su-exec`, entry script,
and account-file seed; the debug base pulls `su-exec` from
the same `gcc` build stage (via `runtime_assets`, so there
is a single compiled artifact), the entry script, and the
`etc_seed` directly, relying on the `:debug` donor for
everything else.

A completely shell-less release image is not viable: the
entry script is POSIX shell and runs as PID 1 before
privilege drop. The smaller-diff alternative — keeping
`release` on `:debug` but `chmod 0700`'ing `/busybox/` — is
rejected because the lean base has independent value
(smaller image, fewer files for Trivy/Grype to scan,
`/busybox/` directory absent entirely). A long-term
alternative — reimplementing the entry-script first-boot work
as a small Rust binary — is the right direction but out of
scope; it is recorded in
[Carry-Over](#carry-over-items).

### D5 — Helper binaries as separate workspace crates

**Follows from:** P2 (a manifest is build-time
configuration, and the property "this binary has no HTTP/TLS
deps" is a build-time invariant), P8 (no stdout to TTY),
P9 (universal helper conventions).
**Addresses:** [R4](#r4--health_check-pulls-in-reqwest-for-a-localhost-get).
**Status:** Landed 2026-04-24. Phases
[2](009-implementation-plan.md#phase-2--health-check--auth-keypair-helpers-d5)
and
[6](009-implementation-plan.md#phase-6--config-probe-helper)
shipped all three helper crates (`torrust-index-health-check`,
`torrust-index-auth-keypair`, `torrust-index-config-probe`)
alongside the shared `torrust-index-cli-common` scaffolding
crate. Wiring them into the entry script is
[Phase 7](009-implementation-plan.md#phase-7--entry-script-contract-d3).

Every helper binary is extracted into its own workspace crate
under `packages/index-*/` and follows P9's universal
conventions: stderr is `tracing` JSON, stdout is one JSON
object (+ trailing newline), exit code is the sole branch
signal for callers, stdout-to-TTY is refused unconditionally,
and the five baseline crates (`clap`, `tracing`,
`tracing-subscriber`, `serde`, `serde_json`) are linked
without per-crate opt-out. A shared
`packages/index-cli-common/` library crate
(`torrust-index-cli-common`) provides the scaffolding so
each binary's `main` is only domain logic.

The crate boundary makes the "no HTTP/TLS deps" property a
manifest-level invariant: a future contributor cannot
accidentally re-introduce `reqwest` because the crate's
`Cargo.toml` simply does not list it. (`cargo tree --bin` is
not a substitute: it scopes to the package that owns the
binary, and the root `torrust-index` package legitimately
depends on `reqwest`/`tokio`/TLS for the server itself.)
`reqwest` remains in the workspace for the importer and
tracker clients; the goal is to prune it from the *helper
binaries'* dep closures, not from the workspace.

All helper crates use the `index-` prefix to disambiguate
from the tracker project's own helpers. The crate *names*
are `torrust-index-*` (matching the produced binary names);
the workspace *paths* are `packages/index-*/`.

**Helper crate roster.** All helper binaries depend on
`torrust-index-cli-common` for their P9 baseline scaffolding
(`refuse_if_stdout_is_tty`, `init_json_tracing`, `emit`,
`BaseArgs`). The "Domain deps" column lists only the
*additional* per-crate dependencies.

| Crate | Path | Domain deps (beyond P9 baseline) |
|---|---|---|
| `torrust-index-cli-common` | `packages/index-cli-common/` | *(library — no binary)* |
| `torrust-index-health-check` | `packages/index-health-check/` | *(none — stdlib networking)* |
| `torrust-index-auth-keypair` | `packages/index-auth-keypair/` | `rsa` (re-exports `pkcs8`) |
| `torrust-index-config-probe` | `packages/index-config-probe/` | `torrust-index-config` (path dep; brings the full parsing surface: `figment`, `toml`, `serde_with`, `serde_json`, `url`, `camino`, `derive_more`, `thiserror`, `lettre` with `default-features = false`); plus direct `url` and `percent-encoding` deps for the sqlite-URL path-extraction logic |
| `torrust-index-entry-script` | `packages/index-entry-script/` | *(test-only `[lib]` — no binary, no runtime code; ships host-side integration tests for the sourced shell library at [`share/container/entry_script_lib_sh`](../share/container/entry_script_lib_sh); `dev-dependencies` only: `tempfile`)* |

Domain-specific deps are the *only* per-crate variation.
The dep-closure exclusion check (Acceptance Criterion #5)
is one regex applied uniformly to all helper binaries —
no per-crate allowlists.

`health_check` moves from `src/bin/health_check.rs` and
is rewritten with `std::net::TcpStream` — no `reqwest`,
no `tokio`, no TLS stack. Its stdout is now a JSON object
(`{"target": "...", "status": 200, "elapsed_ms": 4}`)
instead of silent exit.

`torrust-generate-auth-keypair` moves from
`src/bin/generate_auth_keypair.rs`. Its stdout changes from
raw PEM blocks to a JSON object
(`{"private_key_pem": "...", "public_key_pem": "..."}`),
eliminating the post-processing `sed` calls in the current
documented usage. The entry script (or future Rust entry
binary) consumes the output via `serde_json` or `jq`
instead. See [implementation plan §2](009-implementation-plan.md)
for the extraction steps.

The `torrust-index-config-probe` crate introduced in
[D3](#d3--single-source-of-truth-for-auth-key-paths) (and
detailed in
[implementation plan §6.1](009-implementation-plan.md#61-new-helper-crate-torrust-index-config-probe))
depends on `torrust-index-config` (the parsing crate
extracted in Phase 3) for its purpose — loading the
application's `Settings` through the shared parser. Its
stdout is a JSON object describing the resolved
configuration the entry script needs to dispatch on.

### D6 — Drop build-time `ARG` for runtime concerns

**Follows from:** P2.
**Addresses:** [R5](#r5--build-time-arg-for-runtime-concerns).
**Status:** Landed 2026-04-24 via
[Phase 1](009-implementation-plan.md#phase-1--build-hygiene-d6-d9-build-context-part).

`API_PORT` and `IMPORTER_API_PORT` lose their build-time
`ARG` declarations and keep only their `ENV` defaults, which
the listener and the healthcheck honour at runtime.
`EXPOSE` continues to freeze the default port into image
metadata at build time (a known limitation of `EXPOSE`
itself, documented for operators); it does not affect actual
port binding.

### D7 — Refuse-if-root entry-script guard

**Follows from:** P3.
**Addresses:** [R7](#r7--entry-script-user_id--1000-guard-encodes-the-wrong-property).

The entry script's `USER_ID >= 1000` guard is replaced by an
"is numeric" + `-eq 0` check. The old rule encoded the wrong
property: it rejected valid configurations — rootless Podman
with subuid remapping, low-UID CI runners, BSD-derived hosts
— without stating its intent. The property the script
actually wants to enforce is "do not run as root".

This change is implemented alongside D4 (both edits touch the
same entry-script section); see
[implementation plan §4.1](009-implementation-plan.md).

### D8 — Vendored `su-exec` gains an internal audit record

**Follows from:** P7.
**Addresses:** [R10](#r10--vendored-su-exec-has-no-internal-audit-record).

Upstream `su-exec` has not released since ~2017; framing the
problem as "document a refresh procedure" is wrong-shaped.
The vendored file is treated as code we own. A new
`contrib/dev-tools/su-exec/AUDIT.md` records provenance
(upstream commit + SHA-256), the rationale for choosing
`su-exec` over `gosu`/`setpriv`, a dated audit log, and
re-audit triggers (file change or CVE). The file-change
trigger is CI-enforced; the CVE trigger is a manual review
duty.

**Why no calendar trigger.** The vendored file is ~105 lines
of pure POSIX C (`setgroups` → `setgid` → `setuid` →
`execvp`) with no networking, no crypto, and no dependencies
beyond libc.  Code that doesn't change can't become
vulnerable through inaction — there is no time-dependent
decay in its security posture.  The file-change trigger
catches modifications (the real threat for vendored code),
and the CVE trigger covers external advisories.  A 365-day
hard CI gate that blocks every PR in the repo for an
unchanged ~105-line file would be high cost for zero value.

### D9 — Build hygiene and test-stage coupling

**Follows from:** P2 (build hygiene).
**Addresses:** [R8](#r8--containerignore-sends-excess-context-to-the-builder),
[R9](#r9--test-stages-and-build-stages-are-entangled).
**Status:** Build-context part (`.containerignore`) landed
2026-04-24 via
[Phase 1](009-implementation-plan.md#phase-1--build-hygiene-d6-d9-build-context-part);
the test-stage coupling part is documentation-only and
remains scheduled for
[Phase 9](009-implementation-plan.md#phase-9--documentation--audit-d8-d9-docs-part).

`.containerignore` adds `adr/` and `docs/` to the existing
exclusions (and only those —
`packages/render-text-as-image/` is a workspace member and
path dependency that must remain in the build context;
`tests/fixtures/` and `migrations/<other-driver>/` are read
by the in-build test stages and need a CI matrix run before
any exclusion).

The in-build test stages remain coupled to the image build
(no "skip tests" build path is introduced). The trade-off is
documented in `docs/containers.md` rather than dissolved: the
strong correctness guarantee ("no image without green tests")
is real, and the alternative ("skip tests" path) would
inevitably be used in production.

## Consequences

**Operational.**

- `release` images ship a single root-only `/bin/busybox`
  with curated applet symlinks. The unprivileged `torrust`
  user cannot invoke any of them. Operators who need a
  user-accessible shell use the `debug` image or sidecar
  containers.
- `docker compose up` continues to work for dev (override
  auto-loaded). Production deployments use `make up-prod` or
  pass `--file compose.yaml` explicitly.
- The entry-script env-var contract widens; operators see
  more knobs documented in `docs/containers.md`.
- Bare-metal developers using
  `share/default/config/index.development.sqlite3.toml` as a
  starting template are also affected by the credential strip
  (D2): they must supply `connect_url` and `token` via env
  var or add them to their local copy.

**Schema.**

- `database.connect_url` becomes mandatory. Existing TOMLs
  that omit the field fail to load with a serde
  `missing field 'connect_url'` error. Operators must add
  `connect_url = "sqlite://..."` (or the appropriate MySQL
  URL).
- A config that omits the `[database]` section *entirely*
  also fails (the enclosing `serde(default)` is removed in
  the same change). This is intentional: the same
  "explicit failure forces an explicit choice" rule applies
  at both levels.
- The `[net.tsl]` config key and `"tsl"` JSON API key are
  renamed to `[net.tls]` / `"tls"` (clean break — the
  original spelling was a typo). Existing operator TOMLs
  and API consumers must update.
- `tracker.token` becomes mandatory at the schema level
  (same pattern as `database.connect_url`).
- Within the container, setting both `_PEM` and `_PATH` for
  the same auth key is now a startup error (D3). Operators
  who currently configure both (relying on the application's
  silent PEM-overrides-PATH precedence) must remove one.
  The entry script rejects both before the application
  starts.
- Within the container, using different delivery mechanisms
  across the key pair (e.g. private via PEM, public via
  PATH) is now a startup error (D3). The generator emits
  a matched keypair; it cannot materialise only one side.

**Security.**

- `release` no longer contains `/busybox/`; the curated
  busybox subset and `su-exec` are `0700 root:root`,
  inaccessible to the application process after privilege
  drop. Combined with pruning HTTP/TLS deps from the
  helper binaries (health-check, auth-keypair generator,
  config probe), this materially reduces the attack surface.
- Eliminating credentials from all shipped defaults closes
  the "forgot to override" footgun.
- No `USER` directive is set in the image. The entry script
  runs as root (it needs root for `adduser`, `chown`, and
  key generation) and drops to the `torrust` user via
  `su-exec` before `exec`'ing the application. A stray
  `docker run --entrypoint=<binary> release-image` therefore
  executes as root. This is a deliberate trade-off: the
  entry script's first-boot work requires root, and a `USER`
  directive would force every operator to `--user root` it
  away. The `0700 root:root` busybox/su-exec permissions
  limit what the unprivileged user can do *after* privilege
  drop; they do not prevent a root-level entrypoint
  override. Operators who need defence against accidental
  root execution should enforce `runAsNonRoot` /
  `allowPrivilegeEscalation: false` at the orchestrator
  level (Kubernetes `securityContext`, Compose
  `security_opt`).
- Auth-key PEM material passed via
  `TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__*_PEM` is readable by
  any process that can read `/proc/<pid>/environ` (typically
  same-UID processes or `CAP_SYS_PTRACE` holders).
  Distroless removes most local-attack gadgets; operators
  who need stricter handling should mount keys as files
  (path overrides) and rely on filesystem permissions. Docker
  secrets is the long-term direction
  ([Carry-Over](#carry-over-items)).

**Maintenance.**

- Vendored `su-exec` gains an internal audit record with
  CI-enforced freshness checks.
- Two compose files instead of one, but each is simpler than
  the current single file.

## Acceptance Criteria

The refactor is complete when:

1. **Runtime base split (D4).** `release`-tagged images
   contain no `/busybox/` directory; `/bin/busybox` and
   `/bin/su-exec` are mode `0700 root:root`; applet
   symlinks (e.g. `/bin/sh`) dereference to `/bin/busybox`
   and therefore return EACCES for `--user 1000`
   invocations. (See
   [implementation plan §4](009-implementation-plan.md) for
   the exact `docker run` assertions.)

2. **Credentials stripped (D2).**
   `share/default/config/*.toml` contain no `connect_url`,
   `token`, or `mail` keys, and no literal dev credentials.

3. **Dev compose works (D1).** `docker compose up` (or
   `make up-dev`) starts a working dev environment without
   operator intervention.

4. **Prod compose validates (D1).** `make up-prod` fails
   with a clear error when required credential env vars are
   unset, before invoking compose.

5. **Helper-binary dep closure (D5).** No helper binary
   crate's normal-edge dependency closure contains an HTTP
   client, async runtime, or TLS stack. One exclusion regex
   applied uniformly to all helpers — no per-crate
   allowlists. The P9 baseline (`clap`, `tracing`,
   `tracing-subscriber`, `serde`, `serde_json`) and each
   crate's domain deps (see the roster table in D5) are
   implicitly permitted; the canonical forbidden-dependency
   regex lives in
   [implementation plan Acceptance §5](009-implementation-plan.md#acceptance-criteria--implementation-detail)
   — it is not duplicated here to avoid drift.

6. **Helper JSON + TTY contract (P8, P9).** Every helper
   binary, when invoked with stdout attached to a TTY, exits
   with code 2 before producing any output. When invoked
   with stdout piped, every helper emits exactly one JSON
   object followed by one trailing newline on stdout, and
   `tracing` NDJSON events on stderr.

7. **Documentation complete.** `docs/containers.md`
   describes every env var the entry script reads and the
   relationship between `compose.yaml` and
   `compose.override.yaml`.

8. **Audit record exists (D8).**
   `contrib/dev-tools/su-exec/AUDIT.md` contains provenance,
   choice rationale, at least one dated full-file audit
   entry (each with a structured `SHA-256: <hex>` line —
   the format is contractual; CI parses it), and
   CI-enforced re-audit triggers.

9. **Refuse-if-root guard (D7).** The entry script rejects
   `USER_ID=0` with a clear error and accepts valid low-UID
   values (e.g. `USER_ID=500`).

## Carry-Over Items

Tracked for visibility; not part of this refactor:

- Docker secrets integration for credential management.
- `docker buildx` multi-platform builds (`linux/arm64`).
- Image signing with `cosign`.
- Pin base images (`gcr.io/distroless/cc-debian13` and
  `:debug`) by digest rather than tag for reproducible
  builds and supply-chain integrity.
- Reimplement the entry script's first-boot work as a small
  Rust binary (`torrust-index-entry`), eliminating vendored
  `su-exec` (privilege drop via direct
  `setgroups`/`setgid`/`setuid` syscalls), the shell-based
  IFS/heredoc parsing of probe output, and most of the
  curated busybox applet set. The Phase 3
  `torrust-index-config` extraction, the P9 universal
  helper conventions, and the `torrust-index-config-probe`
  helper are deliberate stepping stones: they pull the
  parsing surface out of the root crate, establish the
  stderr-tracing / stdout-JSON contract all helpers share,
  and prove the script-↔-Rust integration shape before
  committing to the full rewrite. The entry binary would
  depend on `torrust-index-config` and
  `torrust-index-auth-keypair` directly, eliminating the
  serialisation boundary entirely.
- Promote `packages/render-text-as-image/` to a published
  crate and drop the root crate's `path = "packages/..."`
  override; once that lands, the directory can safely be
  added to `.containerignore`.

## Appendix: Diagnostic Detail

The `R-N` items below are the structural problems that
motivated each decision. They are kept here for traceability;
each decision in the body cites the items it addresses.

### R1 — `compose.yaml` conflates dev sandbox and deployment template

The single file mixes dev-only (`mailcatcher`, `tty`,
hardcoded dev credentials, dev-only ports) and prod-shaped
(`restart: unless-stopped`, MySQL healthcheck wiring) concerns.
Operators who treat it as a deployment template must edit it
in place; developers pay for prod-shaped semantics they don't
need. **Severity: High.**

### R2 — Credentials embedded in shipped default configs

Multiple TOMLs under `share/default/config/` carry literal
passwords and dev-only tokens that get baked into the image.
The `compose.yaml` credentials are now annotated as dev-only;
the TOML defaults — the ones embedded in the image artifact —
are not. **Severity: High.**

### R3 — Entry-script path assumptions conflict with config overrides

The script hardcodes `/etc/torrust/index/auth/{private,public}.pem`
for key generation and the SQLite default-database path.
Operators are documented as being able to override these via
`TORRUST_INDEX_CONFIG_OVERRIDE_*`; when they do, the script
still writes to the hardcoded location and the application
silently uses different (or no) keys. The `Auth` schema
exposes four relevant fields (`*_PEM` and `*_PATH` per key)
and falls back to an in-memory ephemeral key when no source
is configured — a fallback that is *never* the intended
container outcome. **Severity: Medium.**

### R4 — `health_check` pulls in `reqwest` for a localhost GET

[`src/bin/health_check.rs`](../src/bin/health_check.rs) issues
a single `GET /health_check` against localhost using
`reqwest` + `tokio` + a TLS stack — its own comment says to
"avoid third-party libraries because they ... introduce new
attack vectors". A stdlib TCP + minimal HTTP/1.1 GET (~30
lines) eliminates hundreds of transitive deps from the
binary's link graph. **Severity: Medium.**

### R5 — Build-time `ARG` for runtime concerns

`API_PORT` / `IMPORTER_API_PORT` are build-time `ARG`s; a
consumer who wants to change them must rebuild the image.
Ports are runtime configuration. Volume paths are hardcoded
literals everywhere, so the parameterisation is also
inconsistent. **Severity: Medium.**

### R6 — Both `debug` and `release` inherit the `:debug` distroless base

`release` builds on `cc-debian13:debug`, inheriting the full
busybox at `/busybox/`. The curated `/bin/` subset is
bypassed by absolute-path invocation of any applet under
`/busybox/`. The "minimal attack surface" property
documented in `docs/containers.md` is weaker than it looks.
**Severity: Medium.**

### R7 — Entry-script `USER_ID >= 1000` guard encodes the wrong property

The actual property is "do not run as root". The `< 1000`
rule rejects valid configurations (rootless Podman with
subuid remapping, low-UID CI runners, BSD-derived hosts)
without stating its intent. **Severity: Low.**

### R8 — `.containerignore` sends excess context to the builder

`adr/` and `docs/` are in the build context and are not read
by any stage; they slow `cargo chef prepare`'s analysis and
bloat the daemon's context tarball. **Severity: Low.**

### R9 — Test stages and build stages are entangled

The `test` and `test_debug` stages both gate the image build
on test success *and* produce the binaries copied into
`runtime`. Any flaky test blocks every image build until
fixed. The decision is to keep the coupling and document it,
since the alternative ("skip tests" path) would inevitably be
used in production. **Severity: Low.**

### R10 — Vendored `su-exec` has no internal audit record

The vendored file has provenance metadata next to it but no
record of which upstream commit it corresponds to, why
`su-exec` was chosen over `gosu`/`setpriv`, or whether
anyone has read the ~105 lines of C and concluded what.
Upstream is effectively unmaintained, so the right framing is
"code we own with a current audit", not "refresh procedure".
**Severity: Low.**
