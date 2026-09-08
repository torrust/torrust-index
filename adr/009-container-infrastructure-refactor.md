# ADR-T-009: Container Infrastructure Refactor

**Status:** Implemented
**Date:** 2026-04-19
**Supersedes:** Earlier `ADR-T-009` draft ("Container Infrastructure Hardening") whose tactical S-N items were merged without a written ADR file. Those items are summarised in [Prior Work](#prior-work) and are not re-litigated here.
**Relates to:** [ADR-T-007](007-jwt-system-refactor.md) (auth key generation performed by the entry script), [ADR-T-010](010-global-command-line-output-contract.md) (global command-line output contract).

---

## Context

The container infrastructure consists of:

- A multi-stage [`Containerfile`](../Containerfile) that builds, tests, and packages the index.
- A single [`compose.yaml`](../compose.yaml) that orchestrates the index together with `tracker`, `mysql`, and `mailcatcher`.
- A POSIX entry script ([`share/container/entry_script_sh`](../share/container/entry_script_sh)) that prepares the runtime, generates auth keys on first boot, and drops privileges via vendored `su-exec` ([`contrib/dev-tools/su-exec/su-exec.c`](../contrib/dev-tools/su-exec/su-exec.c)).
- Default configurations under [`share/default/config/`](../share/default/config/) shipped inside the image at `/usr/share/torrust/default/config/`.
- A small `health_check` binary (`src/bin/health_check.rs`, relocated by this ADR) invoked by the runtime `HEALTHCHECK`.
- E2E orchestration scripts under [`contrib/dev-tools/container/e2e/`](../contrib/dev-tools/container/e2e/) and operator documentation in [`docs/containers.md`](../docs/containers.md).

The previous round of work (see [Prior Work](#prior-work)) brought the infrastructure to a defensible baseline by fixing a long list of concrete bugs. What remained was *structural*: several pieces of the design carried assumptions that no longer matched how the project is used, and continuing to layer fixes onto those assumptions kept producing the same shapes of bug. This ADR records the structural decisions and how they were implemented; the [Appendix](#appendix-diagnostic-detail) catalogues the diagnostic items (`R1`–`R10`) that motivated each one.

### Prior Work

The previous draft of ADR-T-009 ("Container Infrastructure Hardening") catalogued and resolved a set of tactical issues labelled S1 through S12 — entry-script tracing gated on `DEBUG=1`, MySQL healthcheck repair, dev-only port rebinding, restart policies, base-image upgrade `cc-debian12` → `cc-debian13`, and similar. Those changes are merged and are not re-litigated here.

---

## Vision

After this ADR the container subsystem is composed of three layers with deliberately separate concerns.

**The image** is two parallel artifacts built from one Containerfile: a `release` image on a lean distroless base whose only privileged-user utilities are a curated, root-only busybox subset and `su-exec`; and a `debug` image on the same base's `:debug` variant that retains user-accessible developer affordances. In the release image, the unprivileged `torrust` user — under which the application actually runs — has no usable shell and no access to root utilities; the debug image retains user-accessible developer affordances by design. A separate workspace crate, with no transitive HTTP/TLS dependencies, supplies the health-check binary.

**The configuration** is the operator's responsibility, not the image's. Shipped TOML defaults declare structure but no credentials, no `connect_url`, no environment-coupled hostnames, and no environment-coupled paths (notably auth-key paths, which the entry script owns and exports). The schema makes `database.connect_url` and `tracker.token` mandatory, so a missing value fails at parse time with a precise serde error rather than silently falling back to a hidden default. The entry script reads the same `TORRUST_INDEX_CONFIG_OVERRIDE_*` env vars the application reads, and is the single source of truth for any path it materialises (notably auth-key paths) — coordination with the application happens by the script *setting* the override before exec, not by two files agreeing on a constant.

**The orchestration** is two compose files: a production-shaped baseline that references credentials via bare `${VAR}` and binds dev ports to localhost, and an auto-loaded `compose.override.yaml` that re-introduces the dev sandbox (mailcatcher, permissive defaults, tty). A `make up-prod` wrapper validates required env vars before invoking compose; plain `docker compose up` remains the zero-friction dev workflow.

---

## Principles

The decisions below follow from a small set of invariants the container subsystem commits to:

- **P1.** Shipped defaults contain no credentials and no environment-coupled values.
- **P2.** Runtime configuration is runtime; build configuration is build. Neither leaks into the other.
- **P3.** In the release image, the unprivileged runtime user has no usable shell and no access to root utilities. (The debug image deliberately retains user-accessible affordances — see [D4](#d4--two-parallel-runtime-bases-root-only-utilities-in-release).) Privilege drop is irreversible from the application's side under the documented runtime configuration (no `CAP_SETUID`, GID set excludes 0).
- **P4.** The schema enforces required fields. Bootstrap does not re-validate what serde has already proven.
- **P5.** Where two components must agree on a value (path, port, credential), exactly one of them owns it and tells the other; they do not independently maintain a shared constant.
- **P6.** The compose baseline is production-shaped; dev affordances are an additive override layer, never a subtraction from the baseline.
- **P7.** Vendored security-sensitive code is treated as code we own, with a current internal audit record.
- **P8.** Helper binaries implement the TTY-refusal rule now defined globally by [ADR-T-010](010-global-command-line-output-contract.md): commands that emit stdout result data refuse to write it directly to a terminal.
- **P9.** Helper binaries implement the stdout/stderr contract now defined globally by [ADR-T-010](010-global-command-line-output-contract.md). This ADR keeps one helper-specific dependency consequence: every helper binary links the same baseline crates without exception or per-crate justification: `clap` (argv), `tracing` + `tracing-subscriber` with `env-filter` and `json` features (stderr diagnostics), `serde` + `serde_json` (stdout wire format). These are not enumerated in per-crate allowlists. A shared `torrust-index-cli-common` library crate provides the scaffolding: JSON `clap` wrapping, TTY refusal, JSON tracing, JSON panic diagnostics, stdout JSON emission, command runners, and a common `BaseArgs` with `--debug`.

---

## Options Considered

### Option A — Status quo plus targeted patches

Patch the highest-severity items (R1, R2, R3) and leave the rest. Smallest diff. Leaves R4–R10 to keep producing tactical bugs that re-derive the same structural problems.

### Option B — Focused refactor (this ADR)

Treat the container infrastructure as a single subsystem and align it around the principles above. Each decision is locally small but the set is coherent, and each can land independently. Touches many files; requires coordinated CI and documentation updates.

### Option C — Full containerisation reset

Rebuild around a different base (Chainguard, Alpine, or from-scratch with statically linked binaries) and restructure the Containerfile from scratch. Maximum freedom; discards a large amount of working, tested infrastructure for marginal gain. Nothing in the identified problems requires changing the base image.

---

## Decision

**Adopt Option B.**

Option A leaves known structural debt that has already proven willing to come back as new tactical bugs. Option C is disproportionate. Option B keeps the parts that work (multi-stage build, `cargo-chef` caching, in-build `nextest`, distroless runtime, vendored `su-exec`) and corrects the structural pieces that don't.

The decisions that constitute Option B follow.

---

## Decisions

### D1 — Split compose into baseline + override

**Follows from:** P6.
**Addresses:** [R1](#r1--composeyaml-conflates-dev-sandbox-and-deployment-template).

`compose.yaml` is restructured as a production-shaped baseline (no `mailcatcher`, no `tty`, credentials referenced as bare `${VAR}`, dev-only ports on `127.0.0.1`). `compose.override.yaml` is auto-loaded by Compose v2 and carries the dev sandbox (mailcatcher service, permissive `${VAR:-default}` substitutions, tty). A `make up-prod` target validates required credential env vars and runs compose with `--file compose.yaml` (override excluded). `make up-dev` is plain `docker compose up`.

The bare-`${VAR}` rule applies to credentials and environment-coupled hostnames, not to operator selectors that have a sensible cross-environment default (`TORRUST_INDEX_DATABASE_DRIVER` etc.) — those keep their `${VAR:-sqlite3}` defaults so plain `docker compose up` continues to work.

#### Compose baseline restructure

Changes to `compose.yaml`:

- Remove `mailcatcher` (the service and the `index.depends_on: [..., mailcatcher, ...]` reference).
- Remove `tty: true` from `index` / `tracker`.
- Reference credentials via bare `${VAR}` (no default, no `:?required` assertion).
- Bind all ports to `127.0.0.1` except the index API.

The bare-`${VAR}` rule applies to *credentials* (`..._TRACKER__TOKEN`, `..._DATABASE__CONNECT_URL`, MySQL root password, etc.) and to environment-coupled hostnames (`..._MAIL__SMTP__SERVER`). The tracker service's `TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN` follows the same rule: bare `${VAR}` in the prod baseline, with a `:-MyAccessToken` fallback added in `compose.override.yaml` for the dev sandbox. `make up-prod` validates it alongside the index-side credentials.

Intra-compose service-name DNS (`tracker`, `mysql`, `mailcatcher`) is excluded: these are compose-network identifiers resolved by Docker's embedded DNS, not operator-supplied hostnames.

**Why not `${VAR:?required}`.** Compose interpolates each file independently before merging; a `:?required` assertion in the base file fails *during base parse*, before the override's defaults can be considered. Validation is therefore deferred to the `make up-prod` wrapper.

**Defence in depth against empty-string substitution.** Bare `${VAR}` with no fallback means a developer who runs `docker compose -f compose.yaml up` (explicitly bypassing the override) gets empty-string substitution rather than a compose-level error. Three layers catch this before it causes silent runtime misbehaviour:

1. **Config probe ([D3](#d3--single-source-of-truth-for-auth-key-paths))** — the principled gate. Runs inside the container at startup regardless of how compose was invoked. An empty `connect_url` fails `url::Url::parse` (exit 3); an empty `tracker.token` is rejected explicitly (exit 4). This layer cannot be bypassed.
2. **MySQL entrypoint** — the official MySQL image refuses to start with an empty root password.
3. **`make up-prod` wrapper** — fail-fast convenience. Validates required env vars *before* container start so the operator gets a single clear error rather than waiting for each container to boot and fail individually.

An audit of all mail/SMTP references across `compose.yaml` and `src/` was performed using two complementary greps (casual/legacy spellings and the override-prefix form `TORRUST_INDEX_CONFIG_OVERRIDE_MAIL__`), confirming no `index.environment:` block in the prod-shaped baseline still names `mailcatcher`.

#### Override file

`compose.override.yaml` is auto-loaded by Compose v2 and carries:

- `mailcatcher` service, re-attached to `index.depends_on` using long-form (Compose v2 merges `depends_on` additively only in long-form; short-form silently replaces).
- `tty: true` on relevant services.
- Permissive credential defaults via `${VAR:-...}`.
- Optional dev-only port exposures.

```yaml
services:
  index:
    depends_on:
      mailcatcher:
        condition: service_started
  mailcatcher:
    image: docker.io/dockage/mailcatcher:0.8.2
    # ...
```

#### Make targets

A top-level `Makefile` (new; the only prior `Makefile` was the unrelated `contrib/dev-tools/su-exec/Makefile`) provides:

- **`make up-dev`** — plain `docker compose up` (override auto-loaded, dev defaults apply). No validation.
- **`make up-prod`** — validates required env vars are set, then runs `docker compose --file compose.yaml up -d --wait` (override excluded). Produces a clear error on missing variables.

```make
.PHONY: up-dev up-prod _validate-prod-env

COMPOSE_FILE ?= compose.yaml

up-dev:
	docker compose up

_validate-prod-env:
	@sh -uc '\
	  : "$${USER_ID:?required (numeric host UID owning ./storage)}" && \
	  : "$${TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN:?required}" && \
	  : "$${TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL:?required}" && \
	  : "$${TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN:?required}" && \
	  if grep -q "^[[:space:]]*mysql:" $(COMPOSE_FILE); then \
	    : "$${MYSQL_ROOT_PASSWORD:?required}"; \
	  fi'

up-prod: _validate-prod-env
	docker compose --file $(COMPOSE_FILE) up -d --wait
```

The MySQL check is name-coupled to the compose service (`mysql:`) — acceptable because it is defence-in-depth only. The `-d --wait` flags are deliberate: `up` without `-d` blocks indefinitely, and `--wait` makes Compose return non-zero if a service fails its healthcheck within the default timeout. The `COMPOSE_FILE` make-variable is overridable so acceptance tests (and operators with a non-default layout) can point at an alternate file.

#### Bring-up issues found and fixed

Phase 8 was validated end-to-end against `podman-compose 1.5.0` / `podman 5.8.2`. The bring-up exposed five latent bugs in earlier phases, all fixed inline:

1. **Test binaries baking compile-time paths.** `env!("CARGO_BIN_EXE_…")` and `CARGO_MANIFEST_DIR` broke under cargo-nextest's archive → `--extract-to` → `--target-dir-remap` flow. **Fix:** drive contracts through the library API (probe) and `include_str!` + `tempfile` (entry-script tests).
2. **Missing `addgroup` in curated busybox applets.** Busybox `adduser -D` only writes `/etc/passwd`; `getgrnam("torrust")` then failed. **Fix:** add `addgroup` to the curated symlink loop, change the entry script to `addgroup -g "$USER_ID" torrust` followed by `adduser -D -s /bin/sh -u "$USER_ID" -G torrust torrust`, guard both with idempotent `grep`-of-`/etc/{passwd,group}` checks.
3. **`jq` shipped without shared libs.** The runtime stages copied only `/usr/bin/jq`; the binary then aborted with `libjq.so.1: cannot open shared object file`. **Fix:** copy `libjq.so.1` and `libonig.so.5` from the donor alongside the binary, with an `ldd`-based allow-list assertion so a future donor-base upgrade fails the build instead of producing a broken image.
4. **Empty-string env vars treated as configuration TOML.** `Info::from_env` returned `Some("")` for exported-but-empty variables. **Fix:** `Info::from_env` now filters empty strings; `compose.yaml` uses Compose's bare-name pass-through form (`- TORRUST_INDEX_CONFIG_TOML`, no `=`) for optional inline-TOML envs.
5. **Unqualified Docker Hub image names.** `mysql:8.0.45`, `dockage/mailcatcher:0.8.2`, and `torrust/tracker:develop` relied on short-name resolution. **Fix:** fully qualify all as `docker.io/...`.

---

### D2 — Strip credentials from defaults; mandatory `connect_url` and `tracker.token`

**Follows from:** P1, P4.
**Addresses:** [R2](#r2--credentials-embedded-in-shipped-default-configs).

Every file under `share/default/config/` loses its literal `connect_url`, `token`, and `[mail.smtp]` values. The single rule "no credentials, no `connect_url`, no environment-coupled hostnames in shipped defaults" replaces the previous per-driver mix; SQLite `connect_url` values (no credentials, but environment-coupled) are stripped for consistency rather than carved out.

`Database::connect_url` and `Tracker::token` become mandatory at the schema level. A missing value fails at deserialisation with a precise `missing field` error from serde, naming the section. No `check_mandatory_options` branch is added for these fields: the invariant lives in the type, not in a runtime check that readers must trust ran.

The trade-off is acknowledged: a zero-config `docker run torrust-index` no longer produces a working SQLite instance. The simpler enforcement rule is worth the regression because the zero-config path mostly produced confusion when operators later tried to migrate to MySQL and discovered they had been running on an undocumented SQLite default.

#### Credential and environment-coupled value strip

All six files under `share/default/config/` were touched:

| File | Values stripped |
|------|----------------|
| `index.container.toml` | `connect_url`, `token`, `[auth]` paths, `[mail.smtp]` |
| `index.development.sqlite3.toml` | `token` |
| `index.private.e2e.container.sqlite3.toml` | `connect_url`, `token`, `[auth]` paths, `[mail.smtp]` |
| `index.public.e2e.container.toml` | `connect_url`, `token`, `[auth]` paths, `[mail.smtp]` |
| `tracker.private.e2e.container.sqlite3.toml` | `token` |
| `tracker.public.e2e.container.sqlite3.toml` | `token` |

The `[auth]` path entries (`private_key_path`, `public_key_path`) in the container-oriented files are environment-coupled values (they encode the container's volume layout). D2's rule strips them for the same reason it strips `connect_url`: the entry script owns these paths and exports them via `TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__*` ([D3](#d3--single-source-of-truth-for-auth-key-paths)).

The two `tracker.*` files ship from the index repo because they are consumed by the e2e compose flow; they use the tracker service's own config schema, not the index's `config::v2::tracker::Tracker` struct. The schema-level mandatory-token change applies only to the index's `Tracker` struct; the tracker TOMLs are touched here solely for the P1 credential-stripping rule.

`index.development.sqlite3.toml` is not container-only — it is the starting-point template for `cargo run`-based development. After this change, a developer who copies it verbatim must supply `connect_url` and `token` via env var or add them to their local copy.

#### Making `database.connect_url` mandatory

Dropped `#[serde(default = "...")]` on `connect_url` in `packages/index-config/src/v2/database.rs` and removed `impl Default for Database`. Kept the field typed as `Url`.

**Audit of `Database::default` consumers (verified).** A grep across `src/` and `tests/` found exactly one production call site: `packages/index-config/src/v2/mod.rs`, where `default_database()` served as the serde default for the enclosing `TorrustIndex.database` field. No test code called `config::v2::database::Database::default()`. Dropping `impl Default for Database` forced a single parallel change: remove the `#[serde(default = "default_database")]` attribute on `TorrustIndex.database` so an absent `[database]` block fails the same way as an absent `connect_url`.

Sub-options considered and rejected:
- **(a)** Also change the field to `Option<Url>` and add a `check_mandatory_options` branch. Conflates "mandatory" with "type change" and forces every `&Url` consumer to handle the `Option`.
- **(b)** Keep the serde default and reject the known sentinel value. Brittle; the invariant lives far from the type.
- **(c)** Two-stage `RawDatabase` → `Database` validation. Adds a phantom type whose only job is to be unwrapped once.

#### Making `tracker.token` mandatory

Previously, `Tracker::default_token()` returned `ApiToken::new("MyAccessToken")` via `#[serde(default)]`, so stripping `token = "MyAccessToken"` from shipped TOMLs merely moved the credential from the TOML to Rust source.

Dropped `#[serde(default = "Tracker::default_token")]` on `Tracker::token` and removed `Tracker::default_token()`, same pattern as `connect_url`. This keeps one rule for credentials ("no defaults") rather than two ("no defaults in TOML, but a sentinel default in Rust that a probe must know about"). The config probe's exit-4 gate for empty tokens remains as defence in depth — it covers a real gap because `ApiToken`'s `#[derive(Deserialize)]` constructs the inner `String` directly, bypassing the `assert!(!key.is_empty())` guard in `ApiToken::new`; `token = ""` in TOML would silently produce an empty token unless the probe rejects it.

**Audit of `Tracker::default` consumers (verified).** Exactly one production call site: `packages/index-config/src/v2/mod.rs`, where `Settings::default_tracker()` → `Tracker::default()` served as the serde default. Dropping `impl Default for Tracker` forced removal of the `#[serde(default = "default_tracker")]` attribute on `Settings.tracker`.

**Interaction with `check_mandatory_options`.** The existing `load_settings()` validated `"tracker.token"` as a mandatory option via `figment.find_value()`. Once the `#[serde(default)]` was removed, `figment.extract()` produces a serde `missing field` error for the same case, making the entry redundant. Removed `"tracker.token"` from the `mandatory_options` array.

#### Test-fixture consolidation

Removing `impl Default for Settings` (and the `Database` / `Tracker` `Default` impls under it) deleted the single ambient fixture that tests across both crates relied on. A `#[doc(hidden)] pub mod test_helpers` in `torrust-index-config` exposes:

- `PLACEHOLDER_TOML` — the canonical "minimal but legal" TOML (every mandatory field present, nothing more).
- `placeholder_settings() -> Settings` — loads that TOML through `load_settings`, panicking on failure.

The module is `#[doc(hidden)]` (so it does not appear in the public API surface) but `pub` (so integration test binaries in either crate can reach it). Consumers:

- `packages/index-config/src/tests/mod.rs` re-exports `PLACEHOLDER_TOML` under its historical `MINIMUM_VALID_TOML` alias.
- The root crate's `Configuration::for_tests` loads from the same constant.
- `tests/environments/isolated.rs::ephemeral` calls `placeholder_settings()`.
- `tests/e2e/config.rs` intentionally does *not* use the helper: it tests the real shipped sample plus env-var overrides.

---

### D3 — Single source of truth for auth-key paths

**Follows from:** P5.
**Addresses:** [R3](#r3--entry-script-path-assumptions-conflict-with-config-overrides).

The `Auth` config struct exposes both `*_PEM` and `*_PATH` fields per key, and both `Auth::default_private_key_path` and `default_public_key_path` return `None`. There is therefore no schema-level default path the entry script could be byte-equal to; previously the script wrote keys to its own hardcoded location while the application resolved to the in-memory ephemeral fallback, silently disagreeing.

The fix makes the entry script the single source of truth: when no `*_PEM` and no `*_PATH` is configured for a given key, the script generates the key at its built-in location *and exports* `TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__<PAIR>_PATH` to that same location before `exec`'ing the application. The two components agree by construction rather than by maintenance discipline.

The script makes the per-key decision independently, enforces mutual exclusion within a single key (PEM + PATH for the same key is a configuration error), enforces pair-completeness (matching the application's existing invariant in `src/jwt.rs`), and enforces cross-pair source consistency (both keys must use the same delivery mechanism).

The script does not poll env vars to discover the configuration. A small `torrust-index-config-probe` helper loads `Settings` through the parser extracted in [`D5`](#d5--helper-binaries-as-separate-workspace-crates) and prints the resolved auth-key sources. The script calls the helper once and dispatches on its output, so script and application share the parser by construction.

#### Config probe helper (`torrust-index-config-probe`)

A workspace crate `packages/index-config-probe/` (binary `torrust-index-config-probe`) loads the same `Settings` the application loads and emits the container-relevant resolved values as a JSON object on stdout.

**Dependencies.** `torrust-index-config` (path dependency) and `torrust-index-cli-common` (ADR-T-010 scaffolding). The helper inherits the parsing surface (`figment`, `toml`, `serde`, `serde_with`, `url`, `camino`, `derive_more`, `thiserror`, `tracing`) via `torrust-index-config`; it adds direct `url` and `percent-encoding` deps for sqlite-URL path-extraction logic. `figment` is declared with `default-features = false` and an explicit feature allowlist (`toml`, `env`) in `torrust-index-config`'s `Cargo.toml` so a future feature flip cannot smuggle `tokio` in transitively.

**Contract.**

```text
Usage: torrust-index-config-probe

Loads the application's configuration through the same torrust-index-config
loader the application uses, honouring TORRUST_INDEX_CONFIG_TOML,
TORRUST_INDEX_CONFIG_TOML_PATH, and every TORRUST_INDEX_CONFIG_OVERRIDE_*
env var. No CLI flags override the config-file path — callers set
TORRUST_INDEX_CONFIG_TOML_PATH in the environment before invoking the
probe, the same mechanism the application uses.

Refuses to run when stdout is a TTY (exit 2, per ADR-T-010).

`--help`, `--version`, argv errors, TTY refusal, and panic diagnostics are JSON
control-plane records on stderr. They do not emit stdout result data.

On success (exit 0), emits one JSON object + trailing newline on stdout:

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
```

**Field semantics:**

| Field | Meaning |
|-------|---------|
| `schema` | Always 1. Incremented on breaking changes. |
| `database.driver` | URL scheme extracted from `connect_url`. One of `sqlite` or `mysql` — modelled internally as a `Driver` enum. Not the Containerfile's `TORRUST_INDEX_DATABASE_DRIVER` env var (which takes `sqlite3` / `mysql`). |
| `database.path` | For sqlite: the file path (absolute, relative, or `:memory:`). For non-sqlite: null. |
| `auth.*.pem_set` | Raw presence (non-empty after resolution) before PEM-overrides-PATH precedence. Both `None` and `Some("")` fold to `false` because a bare `${VAR}` in compose that substitutes to empty is indistinguishable from "unset" by the time the container starts. |
| `auth.*.path_set` | Same for the path field. |
| `auth.*.source` | Winner after precedence: `"pem"`, `"path"`, or `"none"`. |
| `auth.*.path` | Resolved path if source is `"path"`; null otherwise. |

PEM material is *never* emitted, only its presence (`"pem_set": true`). The probe's stdout is safe to log.

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Recognised, well-formed configuration. |
| 1 | Unhandled panic or unexpected I/O on stdout. |
| 2 | Stdout is a TTY (ADR-T-010), or clap argv-parse failure. |
| 3 | Config-load failure (missing field, parse error, IO error). The underlying error message is forwarded verbatim to stderr via tracing. |
| 4 | Security-critical field present but empty. Currently: `tracker.token`. |
| 5 | Unrecognised database scheme. |

**URL resolution behaviour.** The helper does minimal decoding — just enough to dispatch on scheme and extract a path for the entry script's seeding decisions. `Settings::database.connect_url` is typed as `url::Url`, so `url::Url::parse` runs at deserialisation time. For `sqlite` URLs, the helper handles scheme-specific edge cases explicitly (e.g. `sqlite://data.db?mode=rwc` puts `data.db` in the host slot, not the path; `sqlite::memory:` is opaque). For non-sqlite schemes the helper emits `null` for `database.path`.

| Spelling | `database.driver` | `database.path` |
|----------|--------------------|------------------|
| `sqlite://data.db?mode=rwc` | `"sqlite"` | `"data.db"` (relative) |
| `sqlite:///var/lib/torrust/index.db` | `"sqlite"` | `"/var/lib/torrust/index.db"` |
| `sqlite::memory:` | `"sqlite"` | `":memory:"` |
| `sqlite:///srv/My%20Data/x.db` | `"sqlite"` | `"/srv/My Data/x.db"` |
| `mysql://user:pass@host:3306/db` | `"mysql"` | `null` |
| `mariadb://...` | exit 5 | (stderr: "unsupported scheme: mariadb") |
| `postgres://...` | exit 5 | (stderr: "unsupported scheme: postgres") |
| (`connect_url` missing) | exit 3 | (stderr: serde "missing field" message) |

The hierarchical-path branch percent-decodes via `decode_utf8_lossy`; a non-UTF-8 byte sequence would be replaced with `U+FFFD`. Container deployments overwhelmingly use UTF-8 paths; this is the v1-schema trade-off.

**Default-config-path single source of truth.** The probe binary calls `Info::from_env(DEFAULT_CONFIG_TOML_PATH)` — the JSON-safe sibling of `Info::new` added to `torrust-index-config`. `Info::from_env` reads `TORRUST_INDEX_CONFIG_TOML[_PATH]` exactly like `Info::new` does but skips the diagnostic `println!`s that would corrupt the probe's stdout-only contract. The default path is the `pub const DEFAULT_CONFIG_TOML_PATH` re-exported from `torrust-index-config`.

#### Entry-script integration

The entry script runs under `set -eu`. A line-by-line audit was performed before introduction:

- `inst()` is safe: an `if` with no `else` returns zero when the condition is false (POSIX §2.9.4.1).
- `chown -R` / `chmod -R` on volumes are preceded by `mkdir -p`, making failure unlikely; ordering was verified airtight.
- `$RUNTIME` / `$USER_ID` / `$TORRUST_INDEX_DATABASE_DRIVER` were guarded with `${VAR:-}` forms.

The entry script `set -eu` line appears after the existing `DEBUG=1` → `set -x` line, and immediately after, the script sources the shell library:

```sh
. /usr/local/lib/torrust/entry_script_lib_sh
```

**Temporal contract.** The probe runs exactly once, *before* the script exports any `TORRUST_INDEX_CONFIG_OVERRIDE_*` env vars of its own. The probe's output therefore reflects the operator's true configuration (TOML + operator-supplied env vars) with no script-injected values.

**Runtime execution order:**

1. Probe invocation.
2. `jq` field extraction.
3. Schema version gate.
4. Post-probe PEM/PATH mutual-exclusion check.
5. Pair-completeness check.
6. Cross-pair source consistency check.
7. Three-way auth-key dispatch.
8. Volumes-only directory guard.
9. Key materialisation.
10. Database seeding dispatch.

The probe-consumption section:

```sh
probe_json=$(/usr/bin/torrust-index-config-probe) || exit $?

probe_schema=$(printf '%s' "$probe_json" | jq -r '.schema')
if [ "$probe_schema" != "1" ]; then
    echo "ERROR: config probe emitted schema=$probe_schema" \
         "but this entry script expects schema=1" >&2
    exit 1
fi

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
```

**Post-probe mutual-exclusion check:**

```sh
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

**Pair-completeness and cross-pair source consistency:**

```sh
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

if [ "$private_has" -eq 1 ] && [ "$public_has" -eq 1 ] \
   && [ "$auth_private_key_source" != "$auth_public_key_source" ]; then
    echo "ERROR: private key source is '$auth_private_key_source'" \
         "but public key source is '$auth_public_key_source';" \
         "mixed PEM/PATH across the key pair is not supported." >&2
    exit 1
fi
```

**Three-way auth-key dispatch per key (cases 1/2/3):**

```sh
for pair in private_key public_key; do
    src_var="auth_${pair}_source"
    pth_var="auth_${pair}_path"
    eval "src=\"\$$src_var\""
    eval "pth=\"\$$pth_var\""
    uc_pair=$(printf '%s' "$pair" | tr '[:lower:]' '[:upper:]')

    case $src in
        pem)  continue ;;
        path) eval "${pair}_path=\"\$pth\"" ;;
        none)
            case $pair in
                private_key) default=/etc/torrust/index/auth/private.pem ;;
                public_key)  default=/etc/torrust/index/auth/public.pem  ;;
            esac
            eval "${pair}_path=\"\$default\""
            export "TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__${uc_pair}_PATH=$default"
            ;;
    esac
done
```

**Volumes-only directory guard** then **key materialisation:**

```sh
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
            mkdir -p "$d"; chown torrust:torrust "$d"; chmod 0700 "$d" ;;
        *)
            echo "ERROR: auth key path $d is outside the volumes" \
                 "the entry script manages." >&2
            exit 1 ;;
    esac
done

if [ -n "${private_key_path:-}" ] && [ -n "${public_key_path:-}" ]; then
    if [ ! -s "$private_key_path" ] || [ ! -s "$public_key_path" ]; then
        keypair_json=$(/usr/bin/torrust-index-auth-keypair)
        printf '%s' "$keypair_json" | jq -r .private_key_pem > "$private_key_path"
        printf '%s' "$keypair_json" | jq -r .public_key_pem  > "$public_key_path"
        chown torrust:torrust "$private_key_path" "$public_key_path"
        chmod 0400 "$private_key_path" "$public_key_path"
    fi
fi
```

**Database seeding dispatch:**

```sh
case $database_driver in
    sqlite)  seed_sqlite "$database_path" ;;
    mysql|mariadb) ;; # No file to seed.
    *)
        echo "ERROR: unexpected database.driver='$database_driver'" \
             "from config probe" >&2
        exit 1 ;;
esac
```

Where `seed_sqlite` handles the five outcomes:

- Empty path → error (probe bug).
- `:memory:` → skip silently with info-level note.
- Relative path → warn and skip (no `WORKDIR` set, CWD is `/`).
- Absolute, non-empty file → leave alone.
- Absolute, zero-byte or missing → apply volumes-only auto-mkdir, delegate to `inst()`.

**Note on `eval` usage.** The auth-key loops use a small number of scoped `eval`s to dereference computed variable names. Every right-hand side is double-quoted inside the `eval` string so paths containing spaces survive expansion. Every `eval`'d expansion references a variable populated by the preceding `jq` extraction. The alternative — duplicating each loop body once per key pair — is equivalent for two pairs.

#### Sourced shell library and host-side tests

The entry-script helper functions (`inst`, `key_configured`, `validate_auth_keys`, `seed_sqlite`) are extracted into a sourced library at `share/container/entry_script_lib_sh`. A test-only workspace crate `packages/index-entry-script/` (`torrust-index-entry-script`) drives the helpers via `sh` subprocess and asserts exit codes / stderr contents. The library has no top-level side effects (only function definitions), so sourcing is safe both inside the container and inside the host-side tests.

The crate ships **no runtime code** of its own; it is a `[lib]` whose `tests/` exercise the shell helpers end-to-end. Test coverage:

- `validate_auth_keys` — every branch of the three invariants (mutual exclusion, pair completeness, cross-pair source consistency).
- `seed_sqlite` — every outcome that does not require root or writing to a managed volume (`:memory:` skip, relative-path warn, absolute-non-empty untouched, outside-volumes error, empty-path error).

Both runtime stages copy the library to `/usr/local/lib/torrust/entry_script_lib_sh` with mode `0444 root:root`. The entry script `. /usr/local/lib/torrust/entry_script_lib_sh` at the top, immediately after `set -eu`.

#### `TORRUST_INDEX_DATABASE_DRIVER` scope narrowing

`TORRUST_INDEX_DATABASE_DRIVER` no longer drives runtime database decisions. The entry script's database dispatch uses the config probe's `database.driver` field (derived from `connect_url`'s URL scheme). Note the taxonomy difference: the env var uses `sqlite3` / `mysql`; the probe emits `sqlite` / `mysql` / `mariadb`.

The env var was originally introduced as a Containerfile-level selector for which driver-suffixed default TOML (`index.container.sqlite3.toml` / `index.container.mysql.toml`) to seed at first boot. Phase 9 then consolidated those two samples into a single driver-agnostic `index.container.toml` (Phase 5 had already made `database.connect_url` mandatory, so the file no longer encodes a driver choice). The env var is retained as an input-validation gate so a typo (`postgres`, `Sqlite`, …) fails fast at container start rather than silently propagating an unsupported value into operator-facing scripts; both supported values now resolve to the same template path.

---

### D4 — Two parallel runtime bases; root-only utilities in release

**Follows from:** P3.
**Addresses:** [R6](#r6--both-debug-and-release-inherit-the-debug-distroless-base).

The previous Containerfile built both `release` and `debug` on the `:debug` distroless base, leaving the full busybox tree at `/busybox/` reachable by absolute path even from the unprivileged user — defeating the curated `/bin/` subset.

After this change, `release` builds on the lean distroless `cc-debian13` and ships a single `/bin/busybox` (mode `0700 root:root`) with applet symlinks for the entry script's needs only; `debug` retains the `:debug` base with `/busybox/` on PATH so the unprivileged user has the full applet set.

A completely shell-less release image is not viable: the entry script is POSIX shell and runs as PID 1 before privilege drop. The smaller-diff alternative — keeping `release` on `:debug` but `chmod 0700`'ing `/busybox/` — was rejected because the lean base has independent value (smaller image, fewer files for Trivy/Grype to scan, `/busybox/` directory absent entirely). A long-term alternative — reimplementing the entry-script first-boot work as a small Rust binary — is the right direction but out of scope; it is recorded in [Carry-Over](#carry-over-items).

#### Containerfile restructure

The shared runtime ingredients are factored into a base-agnostic `FROM scratch` stage, then layered onto two parallel runtime bases.

```dockerfile
## ── Runtime asset bundle (base-agnostic) ─────────────────────
FROM gcr.io/distroless/cc-debian13:debug AS busybox_donor

FROM busybox_donor AS busybox_preflight
RUN ["/busybox/sh", "-c", \
     "test -f /busybox/busybox \
      && /busybox/busybox --help >/dev/null \
      && /busybox/busybox install --help 2>&1 | grep -q -- '-D'"]

FROM busybox_preflight AS etc_seed
RUN ["/busybox/sh", "-c", \
     "mkdir -p /seed/etc && \
      printf 'root:x:0:0:root:/:/bin/sh\\n'    > /seed/etc/passwd && \
      printf 'root:x:0:\\n'                    > /seed/etc/group  && \
      : > /seed/etc/profile"]

FROM busybox_donor AS adduser_preflight
COPY --from=etc_seed /seed/etc/passwd /etc/passwd
COPY --from=etc_seed /seed/etc/group  /etc/group
RUN ["/busybox/sh", "-c", \
     "/busybox/adduser -D -s /bin/sh -u 59999 testuser \
      && /busybox/grep -q '^testuser:' /etc/passwd \
      && /busybox/test -d /home/testuser"]

FROM scratch AS runtime_assets
COPY --from=etc_seed --chmod=0644 --chown=0:0 /seed/etc/ /etc/
COPY --from=busybox_preflight --chmod=0700 --chown=0:0 \
    /busybox/busybox /bin/busybox
COPY --from=gcc --chmod=0700 --chown=0:0 \
    /usr/local/bin/su-exec  /bin/su-exec
COPY --chmod=0555 --chown=0:0 \
    ./share/container/entry_script_sh  /usr/local/bin/entry.sh

## ── Preflight gate ───────────────────────────────────────────
FROM scratch AS preflight_gate
COPY --from=busybox_preflight /etc/passwd /tmp/.busybox-ok
COPY --from=adduser_preflight /etc/passwd /tmp/.adduser-ok

## ── Runtime base: release ────────────────────────────────────
FROM gcr.io/distroless/cc-debian13 AS runtime_release
# `gcr.io/distroless/cc-debian13` is usrmerged: `/bin` is a
# symlink to `/usr/bin`, so a recursive `COPY / /` from a
# scratch-built bundle whose layout uses `/bin/...` fails with
# *cannot copy to non-directory*. Copy each curated path
# individually into `/usr/bin/` and let the base's `/bin →
# /usr/bin` symlink forward bare-name lookups.
COPY --from=runtime_assets /etc/passwd  /etc/passwd
COPY --from=runtime_assets /etc/group   /etc/group
COPY --from=runtime_assets /etc/profile /etc/profile
COPY --from=runtime_assets /bin/busybox            /usr/bin/busybox
COPY --from=runtime_assets /bin/su-exec            /usr/bin/su-exec
COPY --from=runtime_assets /usr/local/bin/entry.sh /usr/local/bin/entry.sh
COPY --from=preflight_gate /tmp/.adduser-ok /tmp/.preflight-sentinel
ENV PATH=/usr/local/bin:/bin:/usr/bin:/sbin
RUN ["/usr/bin/busybox", "sh", "-c", \
     "for a in sh adduser addgroup install mkdir dirname chown chmod tr mktemp cat printf rm echo grep; do \
        /usr/bin/busybox ln -s busybox /usr/bin/$a; \
      done && rm -f /tmp/.preflight-sentinel"]

## ── Runtime base: debug ──────────────────────────────────────
FROM gcr.io/distroless/cc-debian13:debug AS runtime_debug
COPY --from=etc_seed --chmod=0644 --chown=0:0 /seed/etc/ /etc/
COPY --from=preflight_gate /tmp/.adduser-ok /tmp/.preflight-sentinel
COPY --from=runtime_assets /bin/su-exec /bin/su-exec
COPY --chmod=0555 --chown=0:0 \
    ./share/container/entry_script_sh  /usr/local/bin/entry.sh
RUN ["/busybox/sh", "-c", \
     "/busybox/ln -s /busybox/sh /bin/sh && rm -f /tmp/.preflight-sentinel"]
ENV PATH=/usr/local/bin:/busybox:/bin:/usr/bin:/sbin
```

The existing `test` and `test_debug` stages are preserved with tightened permissions: the application binary (`torrust-index`) keeps `0755`; root-phase-only helper binaries (`torrust-index-health-check`, `torrust-index-auth-keypair`, `torrust-index-config-probe`) are tightened to `0500 root:root`.

```dockerfile
RUN chmod 0755 /app/bin/torrust-index && \
    chown 0:0 /app/bin/torrust-index-health-check \
              /app/bin/torrust-index-auth-keypair \
              /app/bin/torrust-index-config-probe && \
    chmod 0500 /app/bin/torrust-index-health-check \
               /app/bin/torrust-index-auth-keypair \
               /app/bin/torrust-index-config-probe
```

The two final targets:

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
COPY --from=jq_donor --chmod=0500 --chown=0:0 /jq/jq           /usr/bin/jq
COPY --from=jq_donor --chmod=0444 --chown=0:0 /jq/libjq.so.1   /usr/lib/x86_64-linux-gnu/libjq.so.1
COPY --from=jq_donor --chmod=0444 --chown=0:0 /jq/libonig.so.5 /usr/lib/x86_64-linux-gnu/libonig.so.5
ENTRYPOINT ["/usr/local/bin/entry.sh"]
HEALTHCHECK --interval=5s --timeout=5s --start-period=3s --retries=3 \
  CMD /usr/bin/torrust-index-health-check "http://localhost:${API_PORT}/health_check" \
   && /usr/bin/torrust-index-health-check "http://localhost:${IMPORTER_API_PORT}/health_check"
CMD ["/usr/bin/torrust-index"]

## ── Final: debug ─────────────────────────────────────────────
FROM runtime_debug AS debug
ENV TORRUST_INDEX_CONFIG_TOML_PATH=/etc/torrust/index/index.toml \
    TORRUST_INDEX_DATABASE_DRIVER=sqlite3 \
    USER_ID=1000 API_PORT=3001 IMPORTER_API_PORT=3002 \
    TZ=Etc/UTC ENV=/etc/profile RUNTIME=debug
EXPOSE 3001/tcp 3002/tcp
VOLUME ["/var/lib/torrust/index","/var/log/torrust/index","/etc/torrust/index"]
COPY --from=test_debug /app/ /usr/
COPY --from=jq_donor --chmod=0500 --chown=0:0 /jq/jq           /usr/bin/jq
COPY --from=jq_donor --chmod=0444 --chown=0:0 /jq/libjq.so.1   /usr/lib/x86_64-linux-gnu/libjq.so.1
COPY --from=jq_donor --chmod=0444 --chown=0:0 /jq/libonig.so.5 /usr/lib/x86_64-linux-gnu/libonig.so.5
ENTRYPOINT ["/usr/local/bin/entry.sh"]
HEALTHCHECK --interval=5s --timeout=5s --start-period=3s --retries=3 \
  CMD /usr/bin/torrust-index-health-check "http://localhost:${API_PORT}/health_check" \
   && /usr/bin/torrust-index-health-check "http://localhost:${IMPORTER_API_PORT}/health_check"
# Default CMD matches release so the debug image is a drop-in
# replacement for it. Operators reach an interactive shell with
# `docker run … sh` (or any other curated applet) at run time.
CMD ["/usr/bin/torrust-index"]
```

The `debug` image differs from `release` in two ways: (1) `runtime_debug` is layered on `gcr.io/distroless/cc-debian13:debug` and leaves the donor's full `/busybox/` tree in place and on PATH so the `torrust` user has user-accessible developer affordances; (2) `ENV=/etc/profile` is set so busybox `sh` sources it on interactive non-login invocations. `HEALTHCHECK`, the `torrust-index-health-check` binary, and the default `CMD` are identical to release — the debug image is a drop-in replacement that operators can swap in when they need an interactive break-glass shell, reachable with `docker run … sh` (or any other curated applet) at run time. The entry script runs unchanged on both.

#### Curated applet reference

The release-base symlink loop covers every applet the entry script invokes by bare name. Shell built-ins (`test`, `[`, `read`, `eval`, `case`, `cd`, `exec`, `set`, `trap`, `export`, `.`) do not need applet symlinks.

| Applet | Used by |
|--------|---------|
| `sh` | Entry script interpreter |
| `adduser` | Create `torrust` user at first boot |
| `addgroup` | Create `torrust` group before `adduser` |
| `install` | `inst()` helper — seed config/database templates |
| `mkdir` | Create volume subdirectories, auth-key dirs |
| `dirname` | Resolve parent of auth-key and database paths |
| `chown` | Fix ownership on volumes, auth keys, seeded files |
| `chmod` | Fix permissions on volumes, auth keys |
| `tr` | Auth-key loops (`uc_pair` upper-case conversion) |
| `mktemp` | Temporary file for auth-key generation |
| `cat` | MOTD assembly, profile sourcing |
| `printf` | MOTD lines |
| `rm` | Clean up temp files |
| `echo` | Error messages, MOTD profile hook |
| `grep` | Idempotency guards in §D7's `addgroup` / `adduser` block; reserved for future entry-script extensions and ad-hoc operator break-glass debugging via `docker exec -u root`. |

`su-exec` is a standalone binary at `/usr/bin/su-exec` (release) or `/bin/su-exec` (debug), not a busybox applet.

**CI reconciliation.** A CI step extracts the applet list from the Containerfile's symlink-loop `for` statement and compares it against a grep of bare-name external commands in the entry script. The check is advisory (warns on mismatch, does not block the build) because the grep is necessarily heuristic.

#### Design notes

- **One binary, many names.** Busybox dispatches on `argv[0]`, so all symlinks to `/bin/busybox` invoke the corresponding applet. The asset bundle contains exactly one busybox binary (~1 MB).
- **Root-only permissions.** Every applet the entry script needs is reachable at `/bin/<name>`; the unprivileged `torrust` user gets `EACCES` on `/bin/busybox` (and therefore every symlink to it) after `su-exec` drops privileges.
- **The `torrust` user has no usable login shell in release.** `adduser -s /bin/sh` records `/bin/sh` as the user's login shell, but `/bin/sh → /bin/busybox` is `0700 root:root`, so `su torrust` / `docker exec -u 1000 … sh` cannot spawn one. In the debug image, `/busybox/sh` is on PATH and user-accessible.
- **Security assumption.** The root-only mode relies on the unprivileged user's GID set not including 0 and on no `cap_dac_*` capabilities being granted. Both hold under the documented compose/run flow.
- **PATH is pinned in both bases.** In release, PATH does not include `/busybox/`; in debug, `/busybox/` appears on PATH before `/bin/`.
- **`docker exec -u root … sh` still works** for emergency production debugging on the release image. This is the documented break-glass procedure.
- **`runtime_assets` is `FROM scratch`**, base-agnostic, used only by the release base.
- **`preflight_gate`** aggregates all donor-validation stages behind a single stage. Both runtime bases COPY a sentinel from it, creating an explicit BuildKit dependency edge that prevents any preflight from being pruned.
- **Distroless `nonroot` UID is not preserved.** `runtime_assets` overwrites `/etc/passwd` with a root-only seed. Privilege drop uses `su-exec` to a runtime-created `torrust` user instead.
- **Base `nsswitch.conf` is preserved.** `etc_seed` seeds only `passwd`, `group`, and `profile`; `nsswitch.conf` is inherited from the base unchanged, preserving `hosts: files dns`.
- **No `/busybox/` directory in release.** The lean distroless base does not ship it; the R6 bypass concern is eliminated entirely.
- **Distroless ships per-architecture images**, so the busybox binary extracted from the `:debug` donor is native to the build platform; this survives a future `docker buildx` multi-platform rollout without changes.
- **`/etc/profile` is seeded as empty** so the debug image's `ENV ENV=/etc/profile` points at a real path. Operators who bind-mount a richer profile get the expected behaviour.

---

### D5 — Helper binaries as separate workspace crates

**Follows from:** P2, P8, P9, and the global command-line output contract later extracted as ADR-T-010.
**Addresses:** [R4](#r4--health_check-pulls-in-reqwest-for-a-localhost-get).

Every helper binary is extracted into its own workspace crate under `packages/index-*/` and follows the command-line output contract now defined globally by ADR-T-010. A shared `packages/index-cli-common/` library crate (`torrust-index-cli-common`) provides the scaffolding so each binary's `main` is only domain logic and command-boundary wiring.

The crate boundary makes the "no HTTP/TLS deps" property a manifest-level invariant: a future contributor cannot accidentally re-introduce `reqwest` because the crate's `Cargo.toml` simply does not list it. `reqwest` remains in the workspace for the importer and tracker clients; the goal is to prune it from the *helper binaries'* dep closures, not from the workspace.

#### Helper crate roster

| Crate | Path | Domain deps (beyond ADR-T-010 helper baseline) |
|---|---|---|
| `torrust-index-cli-common` | `packages/index-cli-common/` | *(library — no binary)* |
| `torrust-index-health-check` | `packages/index-health-check/` | *(none — stdlib networking)* |
| `torrust-index-auth-keypair` | `packages/index-auth-keypair/` | `rsa` (re-exports `pkcs8`) |
| `torrust-index-config-probe` | `packages/index-config-probe/` | `torrust-index-config` (path dep); plus direct `url` and `percent-encoding` deps for sqlite-URL path-extraction |
| `torrust-index-entry-script` | `packages/index-entry-script/` | *(test-only `[lib]` crate — no binary; ships host-side integration tests for the sourced shell library)* |

The dep-closure exclusion check ([Acceptance Criterion #5](#5-helper-binary-dep-closures)) is one regex applied uniformly to all helper binaries — no per-crate allowlists. `torrust-index-entry-script` is excluded from that check by construction: it produces no binary.

#### Shared CLI scaffolding (`index-cli-common`)

**Public API:**

```rust
/// Parse argv with clap, emitting JSON help/version/usage records on stderr.
pub fn parse_args_or_exit<T: clap::Parser>() -> T;

/// Refuse to run if stdout is a terminal (ADR-T-010).
/// Emits a JSON control-plane record to stderr and exits with code 2.
pub fn refuse_if_stdout_is_tty(binary_name: &str);

/// Initialise JSON stderr tracing with RUST_LOG / --debug precedence.
pub fn init_json_tracing_with_debug(debug: bool, default_level: tracing::Level);

/// Install the JSON-only panic hook.
pub fn install_json_panic_hook(command_name: &str);

/// Serialise `value` as one JSON object + trailing newline to stdout.
pub fn emit<T: serde::Serialize>(value: &T) -> std::io::Result<()>;

/// Run a stdout-producing single-JSON-object command.
pub fn run_stdout_json_command<Output, CommandError, Run>(
    command_name: &str,
    debug: bool,
    default_level: tracing::Level,
    run: Run,
) -> std::process::ExitCode
where
    Output: serde::Serialize,
    CommandError: std::fmt::Display,
    Run: FnOnce() -> Result<Output, CommandError>;

/// Run a side-effect command that does not emit stdout result data.
pub fn run_no_stdout_command<CommandError, Run>(
    command_name: &str,
    debug: bool,
    default_level: tracing::Level,
    run: Run,
) -> std::process::ExitCode
where
    CommandError: std::fmt::Display,
    Run: FnOnce() -> Result<(), CommandError>;

/// Common `--debug` flag. Flatten into each binary's `Args` via `#[command(flatten)]`.
#[derive(clap::Args)]
pub struct BaseArgs {
    #[arg(long)]
    pub debug: bool,
}
```

**Dependencies.** The ADR-T-010 helper baseline and nothing else: `clap`, `tracing`, `tracing-subscriber` (with `env-filter` and `json` features), `serde`, `serde_json`.

For helpers whose errors map to ADR-T-010's baseline exit classes, `main`
reduces to:

```rust
fn main() -> std::process::ExitCode {
    install_json_panic_hook("torrust-index-<name>");
    let args = parse_args_or_exit::<Args>();
    run_stdout_json_command("torrust-index-<name>", args.base.debug, Level::INFO, || run(&args))
}
```

#### Health-check rewrite

Moved from `src/bin/health_check.rs` to `packages/index-health-check/`. Rewritten with `std::net::TcpStream` + minimal HTTP/1.1 GET (~30 lines), with `set_read_timeout` / `set_write_timeout` for a short connect/read window. No async runtime.

Stdout result JSON on success:
```json
{"schema": 1, "target": "http://localhost:3001/health_check", "status": 200, "elapsed_ms": 4}
```

On failure, stdout is empty; the exit code is the sole branch signal for callers (Docker, the entry script). Tests cover non-2xx response, connection refused, read timeout, and malformed status line using a `TcpListener` on an ephemeral port.

#### Auth-keypair generator extraction

Moved from `src/bin/generate_auth_keypair.rs` to `packages/index-auth-keypair/`. Domain dep is `rsa` (which re-exports `pkcs8`).

Stdout result JSON:
```json
{"schema": 1, "private_key_pem": "-----BEGIN PRIVATE KEY-----\n...", "public_key_pem": "-----BEGIN PUBLIC KEY-----\n..."}
```

This eliminated the `sed` post-processing in the previous documented usage. Consumers use `jq -r .private_key_pem` (shell) or `serde_json::from_reader::<KeypairOutput>` (Rust). The existing TTY guard migrated to the shared ADR-T-010 infrastructure, unifying on exit code 2 (was exit 1).

The entry script's keygen invocation changed from `torrust-generate-auth-keypair` to `torrust-index-auth-keypair`, and the consumer migrated from `sed` PEM-block extraction to `jq` in the same change (`sed` cannot recover usable PEM from the new single-line JSON output).

This required `jq` in the runtime image. A dedicated `jq_donor` stage in the Containerfile installs `jq` from a pristine `rust:slim-trixie` base:

```dockerfile
FROM rust:slim-trixie AS jq_donor
RUN apt-get update && \
    apt-get install -y --no-install-recommends jq && \
    rm -rf /var/lib/apt/lists/*
```

Both runtime bases copy the binary (and its shared libs — `libjq.so.1`, `libonig.so.5`) from that stage as `0500 root:root`. `jq` is invoked only during the entry script's root-phase (before `su-exec` drops privileges). An `ldd`-based allow-list assertion in the `jq_donor` stage catches future transitive dep changes at build time.

Tests verify the generated JSON output round-trips through `serde_json` and the PEM blocks are parseable by `rsa::RsaPrivateKey::from_pkcs8_pem` / `rsa::RsaPublicKey::from_public_key_pem`.

#### Config crate extraction (`torrust-index-config`)

The `torrust-index-config-probe` helper must call the same `figment` + `serde` parser the application uses, otherwise it reintroduces the disagreement the entry-script contract exists to eliminate. The application's parser was entangled with the root crate's runtime types; a probe binary depending on the root crate would inherit `tokio`/`reqwest`/TLS.

The *parsing* surface of `src/config/` was extracted into `packages/index-config/` (crate `torrust-index-config`) whose non-stdlib deps are `serde`, `serde_json`, `serde_with`, `figment`, `toml`, `url`, `camino`, `derive_more`, `thiserror`, `tracing`, and `lettre` (with `default-features = false`, `builder` + `serde` features only — no async/TLS/transport machinery).

**What moved:**

- All of `src/config/v2/` (the schema modules).
- `src/config/validator.rs`.
- From `src/config/mod.rs`: `Settings` / `Info` / `Metadata` / `Version` / `Tls` / `Error` types, `load_settings`, `check_mandatory_options`, the `CONFIG_OVERRIDE_*` constants, and the `ENV_VAR_CONFIG_TOML*` constants.

**What stayed in the root crate:**

- The `Configuration` wrapper struct holding `tokio::sync::RwLock<Settings>` and its `async` accessors. This is application runtime state, not parsing.

**`Tsl` → `Tls` clean-break rename.** The original `Tsl` spelling was a typo. Renamed to `Tls` / `tls` as part of the extraction with no backwards-compatibility alias:

- **Type:** `Tsl` → `Tls` (definition, all imports).
- **Field:** `Network::tsl` → `Network::tls` (definition and call sites).
- **Serde wire key:** TOML `[net.tsl]` → `[net.tls]`, JSON `"tsl"` → `"tls"`.
- **Local variables:** `opt_net_tsl` → `opt_net_tls`, `tsl_config` → `tls_config`, etc.
- **Shipped defaults:** commented-out `#[net.tsl]` → `#[net.tls]`.
- **Doc-comments:** "TSL" → "TLS".
- **Internal helpers:** `Network::default_tsl()` → `Network::default_tls()`.

The grep-verified call-site count was around twenty Rust sites plus one TOML and one doc-comment JSON example.

**Inward dependencies resolved:**

1. **`DynError`** — defined a `pub type DynError = Arc<dyn std::error::Error + Send + Sync>` alias inside the new crate (one line) so the config crate has no dependency on the web layer.
2. **`PermissionOverride`, `Role`, `Action`, `Effect`** — value types (`#[derive(Deserialize)]` structs/enums) with no service-layer dependencies. Moved into the new crate under `permissions::`; re-exported from `src/services/authorization/` for backwards compatibility.
3. **`Tsl`** — sibling-module import after extraction; no cross-crate work needed.

**Compatibility shim.** `src/config/mod.rs` became a thin re-export: `pub use torrust_index_config::*;` plus the `Configuration` wrapper. Every existing `use crate::config::Settings;` kept compiling.

**Acceptance verified:** `cargo tree -p torrust-index-config -e normal --prefix none` excludes `tokio`, `reqwest`, `sqlx`, `hyper`, `rustls`, `native-tls`, `openssl`. `grep -rE 'Tsl|\.tsl' src/ share/` returns zero hits.

---

### D6 — Drop build-time `ARG` for runtime concerns

**Follows from:** P2.
**Addresses:** [R5](#r5--build-time-arg-for-runtime-concerns).

`API_PORT` and `IMPORTER_API_PORT` lost their build-time `ARG` declarations and kept only their `ENV` defaults, which the listener and the healthcheck honour at runtime. `EXPOSE` continues to freeze the default port into image metadata at build time (a known limitation of `EXPOSE` itself, documented for operators); it does not affect actual port binding.

---

### D7 — Refuse-if-root entry-script guard

**Follows from:** P3.
**Addresses:** [R7](#r7--entry-script-user_id--1000-guard-encodes-the-wrong-property).

The entry script's `USER_ID >= 1000` guard was replaced by an "is numeric" + `-eq 0` check. The old rule encoded the wrong property: it rejected valid configurations — rootless Podman with subuid remapping, low-UID CI runners, BSD-derived hosts — without stating its intent.

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

The `adduser` invocation was also changed to busybox short-option form:

```sh
# before
adduser --disabled-password --shell "/bin/sh" --uid "$USER_ID" "torrust"
# after
addgroup -g "$USER_ID" torrust
adduser -D -s /bin/sh -u "$USER_ID" -G torrust torrust
```

This works uniformly against busybox `adduser` on both runtime bases. The `adduser_preflight` build stage exercises this exact invocation against the shadow-less `etc_seed` layout so a future busybox bump surfaces the failure at build time, not first boot. Both commands are guarded with idempotent `grep`-of-`/etc/{passwd,group}` checks so a container restart is a no-op.

---

### D8 — Vendored `su-exec` gains an internal audit record

**Follows from:** P7.
**Addresses:** [R10](#r10--vendored-su-exec-has-no-internal-audit-record).

Upstream `su-exec` has not released since ~2017; framing the problem as "document a refresh procedure" is wrong-shaped. The vendored file is treated as code we own. `contrib/dev-tools/su-exec/AUDIT.md` records:

- **Provenance.** Upstream URL, commit/tag, date vendored, SHA-256 of `su-exec.c`.
- **Choice rationale.** Why `su-exec` over `gosu` (Go runtime, larger binary) or `setpriv` (util-linux dependency, not on the lean distroless base).
- **Audit log.** Append-only table (oldest first): reviewer, date, repo commit, SHA-256 of `su-exec.c`, scope, conclusions. Each entry contains a structured `SHA-256: <hex>` line (CI-parseable).
- **Re-audit triggers.** File-change trigger (CI-enforced: SHA-256 mismatch fails the build until a fresh audit entry is added). CVE trigger (manual review duty).

**Why no calendar trigger.** The vendored file is ~105 lines of pure POSIX C (`setgroups` → `setgid` → `setuid` → `execvp`) with no networking, no crypto, and no dependencies beyond libc. Code that doesn't change can't become vulnerable through inaction. A 365-day hard CI gate that blocks every PR for an unchanged ~105-line file would be high cost for zero value.

There is deliberately no "refresh procedure" section. If a re-vendor is ever needed, it will be a manual diff-and-review exercise.

---

### D9 — Build hygiene and test-stage coupling

**Follows from:** P2.
**Addresses:** [R8](#r8--containerignore-sends-excess-context-to-the-builder), [R9](#r9--test-stages-and-build-stages-are-entangled).

`.containerignore` adds `adr/` and `docs/` to the existing exclusions.

**Not excluded:**
- `packages/render-text-as-image/` — a workspace member and path dependency that must remain in the build context. (See [Carry-Over](#carry-over-items) for the long-term plan.)
- `tests/fixtures/` and `migrations/<other-driver>/` — not excluded without a full CI matrix run confirming no test references them.

The in-build test stages remain coupled to the image build (no "skip tests" build path is introduced). The trade-off is documented in `docs/containers.md`: the strong correctness guarantee ("no image without green tests") is real, and the alternative would inevitably be used in production.

---

## Implementation Sequence

The implementation landed in nine phases with the following dependencies:

| Phase | Title | Decisions |
|-------|-------|-----------|
| 1 | Build hygiene | D6, D9 (build-context part) |
| 2 | Helper binaries | D5 (health-check, auth-keypair, cli-common) |
| 3 | Extract `index-config` crate | Foundation for D3/D5 |
| 4 | Runtime base split | D4, D7 |
| 5 | Schema & credential strip | D2 |
| 6 | Config probe | D3 (probe half) |
| 7 | Entry-script contract | D3 (script half) |
| 8 | Compose split | D1 |
| 9 | Documentation & audit | D8, D9 (docs part) |

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

Phases 1, 2, and 4 were mutually independent and could land in any order. Phase 3 preceded Phase 5 (Phase 5 edits files Phase 3 moves). Phase 6 needed Phase 3 (to depend on the extracted config crate) and Phase 5 (so the mandatory-`connect_url` schema change was reflected). Phase 7 consumed outputs of Phases 2, 4, 5, and 6.

---

## Consequences

### Operational

- `release` images ship a single root-only `/usr/bin/busybox` (reachable as `/bin/busybox` via the base's usrmerge symlink) with curated applet symlinks under `/usr/bin/<applet>`. The unprivileged `torrust` user cannot invoke any of them. Operators who need a user-accessible shell use the `debug` image or sidecar containers.
- The `debug` image is a drop-in replacement for `release`: same `HEALTHCHECK`, same default `CMD` (`/usr/bin/torrust-index`), same `torrust-index-health-check` binary on disk. It differs only in the runtime base (`gcr.io/distroless/cc-debian13:debug`) which retains a user-accessible `/busybox/` tree on PATH, giving the `torrust` user developer affordances. Operators reach an interactive break-glass shell with `docker run … sh` (or any other curated applet) instead of needing a separate orchestrator profile.
- `docker compose up` continues to work for dev (override auto-loaded). Production deployments use `make up-prod` or pass `--file compose.yaml` explicitly.
- The entry-script env-var contract widened; operators see more knobs documented in `docs/containers.md`.
- Bare-metal developers using `share/default/config/index.development.sqlite3.toml` as a starting template are also affected by the credential strip (D2): they must supply `connect_url` and `token` via env var or add them to their local copy.

### Schema

- `database.connect_url` is mandatory. Existing TOMLs that omit the field fail to load with a serde `missing field 'connect_url'` error.
- A config that omits the `[database]` section entirely also fails (the enclosing `serde(default)` was removed in the same change).
- The `[net.tsl]` config key and `"tsl"` JSON API key are renamed to `[net.tls]` / `"tls"` (clean break — the original spelling was a typo). Existing operator TOMLs and API consumers must update.
- `tracker.token` is mandatory at the schema level (same pattern as `database.connect_url`).
- Within the container, setting both `_PEM` and `_PATH` for the same auth key is a startup error (D3). Operators who previously configured both (relying on the application's silent PEM-overrides-PATH precedence) must remove one.
- Within the container, using different delivery mechanisms across the key pair (e.g. private via PEM, public via PATH) is a startup error (D3).

### Security

- `release` no longer contains `/busybox/`; the curated busybox subset and `su-exec` are `0700 root:root`, inaccessible to the application process after privilege drop. Combined with pruning HTTP/TLS deps from the helper binaries, this materially reduces the attack surface.
- Eliminating credentials from all shipped defaults closes the "forgot to override" footgun.
- No `USER` directive is set in the image. The entry script runs as root (it needs root for `adduser`, `chown`, and key generation) and drops to the `torrust` user via `su-exec` before `exec`'ing the application. A stray `docker run --entrypoint=<binary> release-image` therefore executes as root. This is a deliberate trade-off: the entry script's first-boot work requires root, and a `USER` directive would force every operator to `--user root` it away. The `0700 root:root` busybox/su-exec permissions limit what the unprivileged user can do *after* privilege drop. Operators who need defence against accidental root execution should enforce `runAsNonRoot` / `allowPrivilegeEscalation: false` at the orchestrator level.
- Auth-key PEM material passed via `TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__*_PEM` is readable by any process that can read `/proc/<pid>/environ`. Distroless removes most local-attack gadgets; operators who need stricter handling should mount keys as files (path overrides). Docker secrets is the long-term direction.

### Maintenance

- Vendored `su-exec` has an internal audit record with CI-enforced freshness checks.
- Two compose files instead of one, but each is simpler than the previous single file.

---

## Acceptance Criteria

### 1. Runtime base split (D4)

`release`-tagged images contain no `/busybox/` directory; `/bin/busybox` and `/bin/su-exec` are mode `0700 root:root`; applet symlinks dereference to `/bin/busybox` and therefore return EACCES for `--user 1000` invocations.

```sh
set -eu

docker run --rm --entrypoint=/bin/sh release-image \
    -c '! test -e /busybox'

docker run --rm --user 1000 --entrypoint=/bin/sh release-image -c 'echo pwned'
# Should fail: permission denied.

docker run --rm --user 1000 --entrypoint=/bin/busybox release-image sh -c 'echo pwned'
# Should fail: permission denied.

docker run --rm --user 1000 --entrypoint=/bin/su-exec release-image root sh
# Should fail: permission denied.
```

### 2. Credentials stripped (D2)

`share/default/config/*.toml` contain no `connect_url`, `token`, or `mail` keys, and no literal dev credentials.

```sh
set -eu

! grep -rE '(secret_password|MyAccessToken)' share/default/config/
! grep -rE '^[[:space:]]*(connect_url|token)[[:space:]]*=' share/default/config/
! grep -rE 'mailcatcher' share/default/config/
! grep -rE '^\[mail(\.|\])' share/default/config/
```

### 3. Dev compose works (D1)

`docker compose up` (or `make up-dev`) starts a working dev environment without operator intervention.

```sh
set -eu
docker compose up -d --wait --wait-timeout 60
curl -sf http://localhost:3001/health_check
docker compose down
```

### 4. Prod compose validates (D1)

`make up-prod` fails with a clear error when required credential env vars are unset, before invoking compose.

```sh
set -eu

unset TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN \
      TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL \
      MYSQL_ROOT_PASSWORD 2>/dev/null

output=$(make up-prod 2>&1) && {
    echo "FAIL: make up-prod succeeded with no credentials set" >&2; exit 1
}
echo "$output" | grep -qE 'TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN|TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL'

# When validation passes, compose's own exit code must propagate.
export TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN=x
export TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL=sqlite::memory:
export TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN=x
export MYSQL_ROOT_PASSWORD=x
export USER_ID=1000

empty_compose=$(mktemp --suffix=.yaml)
trap 'rm -f "$empty_compose"' EXIT

output=$(make up-prod COMPOSE_FILE="$empty_compose" 2>&1) && {
    echo "FAIL: make up-prod succeeded with empty compose file" >&2; exit 1
}
```

### 5. Helper-binary dep closures (D5)

No helper binary crate's normal-edge dependency closure contains an HTTP client, async runtime, or TLS stack. One exclusion regex applied uniformly — no per-crate allowlists.

```sh
set -eu

forbidden='^(reqwest|tokio|tokio-[a-z0-9_-]+|hyper|hyper-[a-z0-9_-]+|rustls|rustls-[a-z0-9_-]+|native-tls|openssl|openssl-[a-z0-9_-]+)( |$)'

for crate in torrust-index-health-check torrust-index-auth-keypair torrust-index-config-probe; do
    cargo tree -p "$crate" -e normal --prefix none \
      | grep -Eq "$forbidden" && {
        echo "FAIL: $crate pulls in a forbidden dependency" >&2; exit 1
      }
done
exit 0
```

### 6. Helper JSON + TTY contract (ADR-T-010)

Every helper binary, when invoked with stdout attached to a TTY, exits with code 2 before producing any stdout. When invoked with stdout piped, every helper emits exactly one JSON object followed by one trailing newline on stdout, and JSON/NDJSON control-plane or tracing records on stderr. Help, version, argv errors, TTY refusal, and panic diagnostics are JSON control-plane records on stderr. This is the helper-binary acceptance slice of the global contract later extracted as ADR-T-010.

```sh
set -eu

for bin in torrust-index-health-check \
           torrust-index-auth-keypair \
           torrust-index-config-probe; do

    # TTY refusal
    tty_out=""
    rc=0
    tty_out=$(docker run --rm -t --entrypoint="/usr/bin/$bin" release-image \
        2>/dev/null) || rc=$?
    [ "$rc" -eq 2 ] || { echo "FAIL: $bin did not exit 2 on TTY (got $rc)" >&2; exit 1; }
    [ -z "$tty_out" ] || { echo "FAIL: $bin emitted output before TTY refusal" >&2; exit 1; }

    # stdout result JSON
    case $bin in
        *health-check)
            out=$(docker run --rm --entrypoint="/usr/bin/$bin" \
                release-image "http://localhost:1/nope" 2>/dev/null) || true
            if [ -n "$out" ]; then
                printf '%s' "$out" | jq empty || {
                    echo "FAIL: $bin stdout is not valid JSON" >&2; exit 1; }
            fi ;;
        *auth-keypair)
            out=$(docker run --rm --entrypoint="/usr/bin/$bin" release-image 2>/dev/null)
            printf '%s' "$out" | jq -e '.private_key_pem and .public_key_pem' >/dev/null || {
                echo "FAIL: $bin did not emit the expected JSON shape" >&2; exit 1; } ;;
        *config-probe)
            out=$(docker run --rm --entrypoint="/usr/bin/$bin" \
                -e TORRUST_INDEX_CONFIG_TOML_PATH=/usr/share/torrust/default/config/index.container.toml \
                -e TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL="sqlite::memory:" \
                -e TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="test" \
                release-image 2>/dev/null)
            printf '%s' "$out" | jq -e '.schema and .database and .auth' >/dev/null || {
                echo "FAIL: $bin did not emit the expected JSON shape" >&2; exit 1; } ;;
    esac
done
```

### 7. Documentation complete

`docs/containers.md` describes every env var the entry script reads and the relationship between `compose.yaml` and `compose.override.yaml`.

The entry script maintains a canonical env-var manifest block (`ENTRY_ENV_VARS:` / `END_ENTRY_ENV_VARS`). A CI check extracts from the manifest and verifies every variable appears in `docs/containers.md`:

```sh
set -eu

vars=$(sed -n '/^# ENTRY_ENV_VARS:/,/^# END_ENTRY_ENV_VARS/p' \
         share/container/entry_script_sh \
       | grep -oE '[A-Z][A-Z0-9_]+' | sort -u)

[ -n "$vars" ] || { echo "FAIL: ENTRY_ENV_VARS block not found" >&2; exit 1; }

missing=0
for v in $vars; do
    grep -q "$v" docs/containers.md || { echo "MISSING: $v" >&2; missing=1; }
done

grep -q 'compose\.override\.yaml' docs/containers.md || {
    echo "MISSING: compose.override.yaml documentation" >&2; missing=1; }

[ "$missing" -eq 0 ]
```

### 8. Audit record exists (D8)

`contrib/dev-tools/su-exec/AUDIT.md` contains provenance, choice rationale, at least one dated full-file audit entry (with a structured `SHA-256: <hex>` line), and CI-enforced re-audit triggers.

```sh
set -eu

audit=contrib/dev-tools/su-exec/AUDIT.md

test -s "$audit"
grep -qi 'provenance'        "$audit"
grep -qi 'rationale'         "$audit"
grep -qi 'SHA-256'           "$audit"
grep -qE '[0-9]{4}-[0-9]{2}-[0-9]{2}' "$audit"

recorded=$(sed -n '/^## Audit Log/,$ { s/^SHA-256: \([0-9a-f]{64}\)$/\1/p; }' "$audit" \
  | tail -1)
actual=$(sha256sum contrib/dev-tools/su-exec/su-exec.c | cut -d' ' -f1)
[ "$recorded" = "$actual" ]
```

### 9. Refuse-if-root guard (D7)

The entry script rejects `USER_ID=0` with a clear error and accepts valid low-UID values.

```sh
set -eu

output=$(docker run --rm -e USER_ID=0 release-image 2>&1) && {
    echo "FAIL: accepted USER_ID=0" >&2; exit 1
}
echo "$output" | grep -qi 'root'

output=$(docker run --rm -e USER_ID=500 release-image 2>&1) || true
! echo "$output" | grep -qi 'refusing to run as root'
echo "$output" | grep -qiE 'adduser|torrust-index-config-probe|missing field|connect_url'
```

---

## Carry-Over Items

Tracked for visibility; not part of this refactor:

- Docker secrets integration for credential management.
- `docker buildx` multi-platform builds (`linux/arm64`).
- Image signing with `cosign`.
- Pin base images (`gcr.io/distroless/cc-debian13` and `:debug`) by digest rather than tag for reproducible builds and supply-chain integrity.
- Reimplement the entry script's first-boot work as a small Rust binary (`torrust-index-entry`), eliminating vendored `su-exec` (privilege drop via direct `setgroups`/`setgid`/`setuid` syscalls), the shell-based IFS/heredoc parsing of probe output, and most of the curated busybox applet set. The `torrust-index-config` extraction, the ADR-T-010 helper conventions, and the `torrust-index-config-probe` helper are deliberate prerequisites: they pull the parsing surface out of the root crate, establish the stderr-tracing / stdout-JSON contract all helpers share, and prove the script-↔-Rust integration shape before committing to the full rewrite. The entry binary would depend on `torrust-index-config` and `torrust-index-auth-keypair` directly, eliminating the serialisation boundary entirely.
- Promote `packages/render-text-as-image/` to a published crate and drop the root crate's `path = "packages/..."` override; once that lands, the directory can safely be added to `.containerignore`.

---

## Appendix: Diagnostic Detail

The `R-N` items below are the structural problems that motivated each decision. They are kept here for traceability; each decision in the body cites the items it addresses.

### R1 — `compose.yaml` conflates dev sandbox and deployment template

The single file mixed dev-only (`mailcatcher`, `tty`, hardcoded dev credentials, dev-only ports) and prod-shaped (`restart: unless-stopped`, MySQL healthcheck wiring) concerns. Operators who treated it as a deployment template had to edit it in place; developers paid for prod-shaped semantics they didn't need. **Severity: High.**

### R2 — Credentials embedded in shipped default configs

Multiple TOMLs under `share/default/config/` carried literal passwords and dev-only tokens that got baked into the image. The `compose.yaml` credentials were annotated as dev-only; the TOML defaults — the ones embedded in the image artifact — were not. **Severity: High.**

### R3 — Entry-script path assumptions conflict with config overrides

The script hardcoded `/etc/torrust/index/auth/{private,public}.pem` for key generation and the SQLite default-database path. Operators were documented as being able to override these via `TORRUST_INDEX_CONFIG_OVERRIDE_*`; when they did, the script still wrote to the hardcoded location and the application silently used different (or no) keys. The `Auth` schema exposed four relevant fields (`*_PEM` and `*_PATH` per key) and fell back to an in-memory ephemeral key when no source was configured — a fallback that was *never* the intended container outcome. **Severity: Medium.**

### R4 — `health_check` pulls in `reqwest` for a localhost GET

`src/bin/health_check.rs` issued a single `GET /health_check` against localhost using `reqwest` + `tokio` + a TLS stack — its own comment said to "avoid third-party libraries because they ... introduce new attack vectors". A stdlib TCP + minimal HTTP/1.1 GET (~30 lines) eliminates hundreds of transitive deps from the binary's link graph. **Severity: Medium.**

### R5 — Build-time `ARG` for runtime concerns

`API_PORT` / `IMPORTER_API_PORT` were build-time `ARG`s; a consumer who wanted to change them had to rebuild the image. Ports are runtime configuration. Volume paths were hardcoded literals everywhere, so the parameterisation was also inconsistent. **Severity: Medium.**

### R6 — Both `debug` and `release` inherit the `:debug` distroless base

`release` built on `cc-debian13:debug`, inheriting the full busybox at `/busybox/`. The curated `/bin/` subset was bypassed by absolute-path invocation of any applet under `/busybox/`. The "minimal attack surface" property documented in `docs/containers.md` was weaker than it looked. **Severity: Medium.**

### R7 — Entry-script `USER_ID >= 1000` guard encodes the wrong property

The actual property was "do not run as root". The `< 1000` rule rejected valid configurations (rootless Podman with subuid remapping, low-UID CI runners, BSD-derived hosts) without stating its intent. **Severity: Low.**

### R8 — `.containerignore` sends excess context to the builder

`adr/` and `docs/` were in the build context and were not read by any stage; they slowed `cargo chef prepare`'s analysis and bloated the daemon's context tarball. **Severity: Low.**

### R9 — Test stages and build stages are entangled

The `test` and `test_debug` stages both gated the image build on test success *and* produced the binaries copied into `runtime`. Any flaky test blocked every image build until fixed. The decision was to keep the coupling and document it, since the alternative ("skip tests" path) would inevitably be used in production. **Severity: Low.**

### R10 — Vendored `su-exec` has no internal audit record

The vendored file had provenance metadata next to it but no record of which upstream commit it corresponded to, why `su-exec` was chosen over `gosu`/`setpriv`, or whether anyone had read the ~105 lines of C and concluded what. Upstream is effectively unmaintained, so the right framing was "code we own with a current audit", not "refresh procedure". **Severity: Low.**