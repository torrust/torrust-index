# ADR-T-009: Container Infrastructure Hardening

**Status:** Accepted — Implemented (Phases 1 & 2)
**Date:** 2026-04-18

## Context

The container infrastructure centres on a multi-stage
[`Containerfile`](../Containerfile), a
[`compose.yaml`](../compose.yaml), a POSIX entry script
([`share/container/entry_script_sh`](../share/container/entry_script_sh)),
a vendored `su-exec` binary
([`contrib/dev-tools/su-exec/su-exec.c`](../contrib/dev-tools/su-exec/su-exec.c)),
default configurations under `share/default/config/`, and a set of
E2E orchestration scripts under `contrib/dev-tools/container/`.
Documentation lives in
[`docs/containers.md`](../docs/containers.md).

### Current Architecture

The Containerfile uses six logical stages:

| Stage | Base | Purpose |
|---|---|---|
| `chef` | `rust:trixie` | Install `cargo-chef` + `cargo-nextest` via `cargo-binstall` |
| `tester` | `rust:slim-trixie` | Runtime for extracted nextest archives |
| `gcc` | `gcc:trixie` | Compile `su-exec.c` |
| `recipe` → `dependencies*` → `build*` | `chef` | `cargo-chef` prepare → cook → build (debug & release) |
| `test*` | `tester` | Extract nextest archive, run full test suite, copy binaries |
| `runtime` → `debug` / `release` | `gcr.io/distroless/cc-debian13:debug` | Minimal runtime with busybox subset |

The build pipeline is well-structured: dependency caching via
`cargo-chef`, test execution *inside* the image build (ensuring
shipped binaries pass the test suite), and a distroless runtime
base.  The entry script creates a non-root `torrust` user, installs
default config and database files, generates RSA auth key pairs on
first boot, and drops privileges via `su-exec`.

### Identified Problems

The issues below are grouped by category with severity ratings.

#### S1 — Security: Hardcoded Credentials

**Severity:** High

The compose file and default config files contain hardcoded
credentials in plaintext:

- `compose.yaml` defines `MYSQL_ROOT_PASSWORD=root_secret_password`
  and `MYSQL_PASSWORD=db_user_secret_password` as literal values.
- `share/default/config/index.container.mysql.toml` embeds
  `root:root_secret_password` in the `connect_url`.
- `TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN` defaults to
  `MyAccessToken` in the compose file.
- All E2E scripts repeat `MyAccessToken` and
  `root_secret_password`.

The compose file's MySQL healthcheck references
`/run/secrets/db-password` (a Docker secrets path), yet no
`secrets:` block is defined — suggesting an incomplete migration
toward secrets-based credential management.

> **Residual debt (post-implementation):** The `compose.yaml`
> credentials are now annotated as dev-only, but literal passwords
> remain in `share/default/config/index.container.mysql.toml`,
> `index.public.e2e.container.mysql.toml`, and all E2E scripts.
> Migrating these to `.env` substitution or Docker secrets is
> deferred to Phase 3.

#### S2 — Security: Entry Script Leaks Environment

**Severity:** Medium

The entry script begins with `set -x` unconditionally, causing
every command — including environment variable expansions that may
contain tokens and database passwords — to be printed to stderr.
This output is typically captured in container logs.

#### S3 — Security: Ports Bound to All Interfaces

**Severity:** Medium

All ports in `compose.yaml` bind to `0.0.0.0` (the default when
only `host:container` is specified).  For a development compose
file this exposes MySQL (3306), the tracker UDP (6969), tracker
HTTP (7070), tracker API (1212), and SMTP mock (1025/1080) to the
local network.

#### S4 — Correctness: MySQL Healthcheck Is Broken

**Severity:** Medium

The MySQL service healthcheck command is:
```yaml
test: ['CMD-SHELL', 'mysqladmin ping -h 127.0.0.1 --password="$$(cat /run/secrets/db-password)" --silent']
```
No Docker secret named `db-password` is configured in the compose
file.  `cat /run/secrets/db-password` fails silently inside
`CMD-SHELL`, and `mysqladmin` falls back to no password, which
coincidentally works for `ping` (it tests connectivity, not auth).
The healthcheck "passes" but for the wrong reason — the command
is fundamentally broken and would be misleading during debugging.

#### S5 — Correctness: Importer API Port Not Exposed

**Severity:** Low

`Containerfile` defines `ARG IMPORTER_API_PORT=3002` and uses it
in the release `HEALTHCHECK`, but:
- `EXPOSE` only lists `${API_PORT}/tcp` (3001).
- `compose.yaml` does not map port 3002.

The importer health check runs inside the container (so it works),
but the port is invisible to orchestrators that inspect `EXPOSE`
metadata and inaccessible from outside the container.

#### S6 — Correctness: Stale Legacy Scripts

**Severity:** Low

Two scripts under `contrib/dev-tools/container/` are out of date:

- **`build.sh`** passes `--build-arg UID` but `Containerfile`
  defines `ARG USER_ID`, not `ARG UID`.  It also omits
  `--file Containerfile` (relying on a `Dockerfile` that no longer
  exists).
- **`run.sh`** mounts `$(pwd)/storage:/app/storage` but the
  current runtime uses `/var/lib/torrust/index`.  It also reads
  `config.toml` (no longer the default config name).

Both scripts predate the current Containerfile layout and will
silently produce broken results.

#### S7 — Maintainability: Unpinned `cargo-binstall` Bootstrap

**Severity:** Low

The `cargo-binstall` installer is fetched via `curl | bash` from
GitHub with no version pin or checksum verification:
```dockerfile
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
    https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
```
A compromised or incompatible upstream release would break (or
subvert) builds without any signal.  This pattern appears in both
the `chef` and `tester` stages.

#### S8 — Maintainability: Deprecated MySQL Auth Plugin

**Severity:** Low

`compose.yaml` passes
`--default-authentication-plugin=mysql_native_password` to MySQL
8.0.  This flag is deprecated in MySQL 8.0.34+ and removed in
MySQL 8.4.  The current `image: mysql:8.0` floats on the 8.0
minor track, which may eventually include a version that warns or
ignores this flag.

#### S9 — Documentation: `USER_UID` vs `USER_ID` Typo

**Severity:** Cosmetic

`docs/containers.md` references `USER_UID` as the environment
variable, but the actual variable used in the `Containerfile` and
entry script is `USER_ID`.

#### S10 — Operational: No Restart Policy

**Severity:** Informational

No `restart:` policy is set on any service in `compose.yaml`.
This is appropriate for a development compose file, but if the
same file is used as a deployment template (as the documentation
suggests), services will not restart after crashes.

#### S11 — Operational: Debug Image Has No Healthcheck

**Severity:** Informational

The `debug` stage does not include a `HEALTHCHECK` and does not
ship the `health_check` binary.  The compose file always targets
`release`, so this only matters when running debug images manually.
The omission is intentional (noted in a comment in the
Containerfile) but undocumented externally.

#### S12 — Operational: Busybox Subset Is Minimal

**Severity:** Informational

The runtime copies only `sh`, `cat`, `ls`, `env` from busybox.
Common debugging tools (`id`, `whoami`, `ps`, `grep`, `wget`) are
absent.  This is a deliberate security posture (smaller attack
surface) but can frustrate operational debugging.

## Options Considered

### Option A: Targeted Fixes Only

Address only the bugs and security issues (S1–S6, S9) without
changing the overall design.

**Changes:**
- Replace hardcoded credentials with `.env` defaults and document
  that they must be changed for non-dev use.
- Gate `set -x` behind `[ "${DEBUG:-}" = "1" ]`.
- Fix the MySQL healthcheck to use the literal password.
- Add `EXPOSE ${IMPORTER_API_PORT}/tcp`.
- Delete or mark `build.sh`/`run.sh` as deprecated.
- Fix `USER_UID` → `USER_ID` in docs.

**Pros:** Smallest diff, no process changes.
**Cons:** Does not address pinning, deprecation, or restart
policies.

### Option B: Comprehensive Hardening

Option A plus all maintainability and operational improvements
(S7–S12).

**Changes (in addition to Option A):**
- Pin `cargo-binstall` to a release tag or verify a checksum.
- Replace `mysql_native_password` with `caching_sha2_password` and
  pin `mysql:8.0.x` to a specific minor.
- Add `restart: unless-stopped` to index/tracker in compose (with
  a comment directing operators to adjust).
- Map port 3002 in compose.
- Bind dev-only ports to `127.0.0.1`.
- Add a `compose.override.yaml` example for production overrides.
- Document the busybox subset and the debug image healthcheck
  omission.

**Pros:** Addresses every identified issue.
**Cons:** Larger changeset, may force compose re-creation for
existing developers.

### Option C: Restructure Into Dev/Prod Compose Profiles

Option B plus: split `compose.yaml` into a base file for
production-like use and `compose.override.yaml` for development
extras (mailcatcher, open ports, debug volumes).

**Changes (in addition to Option B):**
- `compose.yaml` becomes production-ready: secrets, restart
  policies, 127.0.0.1-bound ports, no mailcatcher.
- `compose.override.yaml` (auto-loaded by Docker Compose) adds
  mailcatcher, open ports, `tty: true`, debug passthroughs.
- Alternatively, use Compose profiles (`--profile dev`) to
  include development services.
- E2E scripts updated to reference profiles or overrides.

**Pros:** Clean separation. Operators can deploy from `compose.yaml`
directly without stripping dev concerns.
**Cons:** Largest diff. Requires updating all E2E scripts and
documentation. Risk of drift between dev and prod configs.

## Decision

**Option B: Comprehensive Hardening.**

### Rationale

Option A leaves known debt (unpinned tooling, deprecated flags,
missing ports) that will eventually break or cause friction.
Option C is architecturally cleaner but introduces significant
churn for the E2E test matrix and documentation; it can be pursued
as a follow-up once Option B stabilises the foundation.

Option B fixes every concrete issue, improves security posture, and
is achievable in a single focused pass without restructuring the
compose workflow.

## Implementation Plan

### Phase 1 — Security & Correctness Fixes

Addresses: S1, S2, S3, S4, S5, S6, S9.

1. **Entry script: gate `set -x`.**
   Replace unconditional `set -x` with:
   ```sh
   [ "${DEBUG:-}" = "1" ] && set -x
   ```

2. **Compose: fix MySQL healthcheck.**
   Replace the broken secrets reference with the literal password
   (matching `MYSQL_ROOT_PASSWORD`) and add a `TODO` comment to
   migrate to Docker secrets:
   ```yaml
   test: ['CMD-SHELL', 'mysqladmin ping -h 127.0.0.1 --password="$$MYSQL_ROOT_PASSWORD" --silent']
   ```

3. **Compose: bind dev-only ports to localhost.**
   Change port mappings for MySQL, tracker, and mailcatcher to
   `127.0.0.1:HOST:CONTAINER` format.  Keep the index API on
   `0.0.0.0` (or configurable) since it is the primary service.

4. **Compose: add importer port mapping.**
   Add `127.0.0.1:3002:3002` to the index service ports
   (localhost-bound since the importer is an internal service).

5. **Containerfile: expose importer port.**
   Add `EXPOSE ${IMPORTER_API_PORT}/tcp`.

6. **Compose: add credential comments.**
   Add comments above `MYSQL_ROOT_PASSWORD` and the tracker token
   default documenting that these are development-only values and
   must be changed for any public deployment.

7. **Delete stale scripts.**
   Remove `contrib/dev-tools/container/build.sh` and
   `contrib/dev-tools/container/run.sh`.

8. **Docs: fix `USER_UID` → `USER_ID`.**
   Correct the variable name in `docs/containers.md`.

### Phase 2 — Maintainability & Operational Improvements

Addresses: S7, S8, S10, S11, S12.

1. **Pin `cargo-binstall`.**
   Replace the `main`-branch installer URL with a tagged release.
   Example:
   ```dockerfile
   RUN curl -L --proto '=https' --tlsv1.2 -sSf \
       https://raw.githubusercontent.com/cargo-bins/cargo-binstall/v1.18.1/install-from-binstall-release.sh | bash
   ```
   Alternatively, download the binary directly and verify its
   SHA-256.

2. **Pin MySQL image to a minor version.**
   Change `mysql:8.0` to a specific minor (e.g. `mysql:8.0.45`).
   Replace `--default-authentication-plugin=mysql_native_password`
   with `--authentication-policy=mysql_native_password` (supported
   from 8.0.27+), or migrate to `caching_sha2_password` and update
   connection strings.

3. **Add restart policies.**
   Add `restart: unless-stopped` to the `index` and `tracker`
   services.  Add a comment noting this can be changed to `always`
   or `no` per deployment needs.

4. **Document debug image healthcheck omission.**
   Add a note in `docs/containers.md` explaining that the debug
   target intentionally omits the healthcheck and `health_check`
   binary.

5. **Document the busybox subset.**
   Add a note in `docs/containers.md` listing which busybox
   commands are available (`sh`, `cat`, `ls`, `env`) and explaining
   the rationale (minimal attack surface on distroless).

### Phase 3 — Follow-Up (Out of Scope)

These items are candidates for future work, not part of this ADR:

- **Compose profiles or split files** (Option C) — separating dev
  and prod concerns cleanly.
- **Docker secrets integration** — replacing environment-variable
  credentials with mounted secrets for MySQL and tracker tokens.
- **Multi-platform builds** — `docker buildx` for `linux/arm64`.
- **Image signing** — signing published images with `cosign` or
  Notary v2.

## Consequences

- Developers on existing setups will need to recreate compose
  services after the port-binding changes (addresses bind to
  `127.0.0.1` instead of `0.0.0.0`).
- The removal of `build.sh` and `run.sh` may break any workflow
  that still references them (unlikely given their stale state).
- Pinning `cargo-binstall` and MySQL introduces a maintenance
  burden: versions must be bumped periodically.
- The entry script will no longer produce verbose output by
  default; operators must set `DEBUG=1` to troubleshoot startup.

## Implementation Status

**Phases 1 and 2 are complete.** All fixes described above have
been applied to the codebase as of the acceptance date:

- Entry script gated behind `DEBUG=1`. (S2)
- MySQL healthcheck uses `$$MYSQL_ROOT_PASSWORD`. (S4)
- Dev-only ports bound to `127.0.0.1`. (S3)
- Importer port exposed in Containerfile and mapped in compose. (S5)
- Stale `build.sh` and `run.sh` deleted. (S6)
- `USER_UID` typo corrected in docs. (S9)
- `cargo-binstall` pinned to `v1.18.1`. (S7)
- MySQL image pinned to `8.0.45`; auth flag updated. (S8)
- Restart policies added to index and tracker services. (S10)
- Debug healthcheck omission and busybox subset documented in
  `docs/containers.md`. (S11, S12)
- Credential comments added to compose. (S1, partial — see
  residual debt note under S1.)
- `.dockerignore` renamed to `.containerignore` for Podman
  compatibility.
- Entry script: fixed `USER_ID` guard from `&&` (always-false) to
  `||` (correct short-circuit).
- Removed stale `TORRUST_TRACKER_USER_UID` export from E2E
  scripts.
- Base images upgraded from `bookworm` to `trixie`
  (`cc-debian12` → `cc-debian13`).
- Removed redundant `--tests --benches --examples` flags from
  `cargo chef cook` / `cargo nextest archive` (covered by
  `--all-targets`).

Phase 3 items remain open for future work.
