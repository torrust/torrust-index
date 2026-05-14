# Containers (Docker or Podman)

## Demo environment

The pre-built public image still runs with one command, but
after ADR-T-009 §D2 it requires two operator-supplied
secrets at startup:

With Docker:

```sh
docker run -it \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="MyAccessToken" \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL="sqlite:///var/lib/torrust/index/database/sqlite3.db?mode=rwc" \
    torrust/index:latest
```

or with Podman:

```sh
podman run -it \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="MyAccessToken" \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL="sqlite:///var/lib/torrust/index/database/sqlite3.db?mode=rwc" \
    torrust/index:latest
```

A bare `docker run -it torrust/index:latest` (the
zero-config invocation that worked in earlier releases) now
fails to start with a serde `missing field` error — this is
intentional, see [ADR-T-009 §D2](../adr/009-container-infrastructure-refactor.md).

## Requirements

- Tested with recent versions of Docker or Podman.

## Volumes

The [Containerfile](../Containerfile) defines three volumes:

```Dockerfile
VOLUME ["/var/lib/torrust/index","/var/log/torrust/index","/etc/torrust/index"]
```

When instancing the container image with the `docker run` or `podman run` command, we map these volumes to the local storage:

```s
./storage/index/lib -> /var/lib/torrust/index
./storage/index/log -> /var/log/torrust/index
./storage/index/etc -> /etc/torrust/index
```

> NOTE: You can adjust this mapping for your preference, however this mapping is the default in our guides and scripts.

### Pre-Create Host-Mapped Folders

Please run this command where you wish to run the container:

```sh
mkdir -p ./storage/index/lib/ ./storage/index/log/ ./storage/index/etc/
```

### Matching Ownership ID's of Host Storage and Container Volumes

It is important that the `torrust` user has the same uid `$(id -u)` as the host mapped folders. In our [entry script](../share/container/entry_script_sh), installed to `/usr/local/bin/entry.sh` inside the container, switches to the `torrust` user created based upon the `USER_ID` environmental variable.

When running the container, you may use the `--env USER_ID="$(id -u)"` argument that gets the current user-id and passes to the container.

`USER_ID` must be a non-negative integer and must not be `0`
(the entry script refuses to run as root). Any positive UID
is accepted — including low-UID values produced by rootless
Podman with subuid remapping, low-UID CI runners, or
BSD-derived hosts. The previous `USER_ID >= 1000` rule was
dropped because it rejected several of these legitimate
configurations without stating its intent.

### Mapped Tree Structure

Using the standard mapping defined above produces this following mapped tree:

```s
storage/index/
├── lib
│   ├── database
│   │   └── index.sqlite3.db => /var/lib/torrust/index/database/index.sqlite3.db [auto populated, sqlite3 only]
│   └── tls
│       ├── localhost.crt  => /var/lib/torrust/index/tls/localhost.crt [user supplied]
│       └── localhost.key  => /var/lib/torrust/index/tls/localhost.key [user supplied]
├── log                    => /var/log/torrust/index (future use)
└── etc
    ├── auth
    │   ├── private.pem    => /etc/torrust/index/auth/private.pem [auto generated on first boot]
    │   └── public.pem     => /etc/torrust/index/auth/public.pem  [auto generated on first boot]
    └── index.toml        => /etc/torrust/index/index.toml [auto populated]
```

> NOTE: you only need the `tls` directory and certificates in case you have enabled SSL.
>
> The `auth/` directory and RSA key pair are auto-generated on first boot by the
> container entry script. Sessions persist across restarts as long as the
> `/etc/torrust/index` volume is retained. To use your own keys, either
> pre-populate the volume before first boot or overwrite the generated files and
> restart. Per ADR-T-009 §D3 the script is the single source of truth for the
> auth-key paths: when neither `..._AUTH__*_PEM` nor `..._AUTH__*_PATH` is
> configured, the script generates the pair at the locations above and exports
> the corresponding `..._AUTH__*_PATH` overrides so the application sees the
> same paths.
>
> The SQLite database file is only auto-populated when the resolved
> `database.connect_url` (via `TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL`
> or the mounted `index.toml`) names an absolute SQLite path under one of the
> managed volumes. MySQL/MariaDB connections are not seeded — the application
> connects directly. See [Entry Script Contract](#entry-script-contract).

## Building the Container

### Test-Stage Gate

The image build is **gated on the workspace test suite passing**.
The `Containerfile` defines intermediate `test` / `test_debug`
stages that compile the workspace's test archive and run it
with `cargo nextest run` before the runtime image stages
(`runtime_release` / `runtime_debug`) are assembled. A red
test run blocks image production until the underlying issue
is fixed.

This is a deliberate trade-off:

- **What you get.** A red `develop`/`main` test run cannot
  silently ship as a published image. There is no
  build-time `--skip-tests` escape hatch — and that omission
  is intentional. Any such hatch would, in time, end up wired
  into a release pipeline "just for this one urgent fix" and
  would defeat the gate's whole purpose.
- **What it costs.** A flaky or environment-dependent test
  blocks image production even when the application is
  unaffected. Treat this as a forcing function, not as
  collateral damage: stabilise or quarantine the test rather
  than reaching for an opt-out.
- **Escalation path.** When a known-flaky test blocks an
  urgent build, the supported response is to `#[ignore]`
  the specific test on a tracked branch with a linked
  issue, rebuild, then follow up by fixing the test and
  removing the `#[ignore]`. There is no privileged rebuild
  path that bypasses the gate.

See [ADR-T-009 §D9](../adr/009-container-infrastructure-refactor.md)
for the design rationale.

### Clone and Change into Repository

```sh
# Inside your dev folder
git clone https://github.com/torrust/torrust-index.git; cd torrust-index
```

### (Docker) Setup Context

Before starting, if you are using docker, it is helpful to reset the context to the default:

```sh
docker context use default
```

### (Docker) Build

```sh
# Release Mode
docker build --target release --tag torrust-index:release --file Containerfile .

# Debug Mode
docker build --target debug --tag torrust-index:debug --file Containerfile .
```

### (Podman) Build

Podman defaults to writing OCI-format manifests. The OCI image-spec
has no field for `HEALTHCHECK`, so building without `--format docker`
drops the directive (and prints a `WARN[…] HEALTHCHECK is not
supported for OCI image format` line). Pass `--format docker` so the
healthcheck survives in the manifest:

```sh
# Release Mode
podman build --format docker --target release --tag torrust-index:release --file Containerfile .

# Debug Mode
podman build --format docker --target debug --tag torrust-index:debug --file Containerfile .
```

`docker build` defaults to Docker-format manifests, so the flag is
only needed for Podman / Buildah. Running OCI-format images on Docker
or Podman works regardless of which format they were built with;
the format only matters for Docker-specific manifest extensions like
`HEALTHCHECK`.

## Running the Container

### Basic Run

The minimum invocation supplies the two mandatory overrides
introduced by ADR-T-009 §D2 (`tracker.token` and
`database.connect_url`). Without them, the config probe
fails the schema check and the entry script aborts startup
before privilege drop. See [Entry Script Contract](#entry-script-contract)
for the full boot sequence.

#### (Docker) Run Basic

```sh
# Release Mode
docker run -it \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="MySecretToken" \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL="sqlite:///var/lib/torrust/index/database/index.sqlite3.db?mode=rwc" \
    torrust-index:release

# Debug Mode
docker run -it \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="MySecretToken" \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL="sqlite:///var/lib/torrust/index/database/index.sqlite3.db?mode=rwc" \
    torrust-index:debug
```

#### (Podman) Run Basic

```sh
# Release Mode
podman run -it \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="MySecretToken" \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL="sqlite:///var/lib/torrust/index/database/index.sqlite3.db?mode=rwc" \
    torrust-index:release

# Debug Mode
podman run -it \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="MySecretToken" \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL="sqlite:///var/lib/torrust/index/database/index.sqlite3.db?mode=rwc" \
    torrust-index:debug
```

### Arguments

The arguments need to be placed before the image tag. i.e.

`run [arguments] torrust-index:release`

#### Environmental Variables:

Environmental variables are loaded through the `--env`, in the format `--env VAR="value"`.

The following environmental variables can be set:

- `TORRUST_INDEX_CONFIG_TOML_PATH` - The in-container path to the index configuration file, (default: `"/etc/torrust/index/index.toml"`).
- `TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN` - **Required.** Tracker admin token. Per ADR-T-009 §D2 the shipped TOMLs no longer carry a default value for this field, so the operator must supply it via this env var (or pre-populate the in-volume `index.toml`). Startup fails with `missing field 'token'` otherwise.
- `TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL` - **Required.** Database connection URL (e.g. `sqlite:///var/lib/torrust/index/database/index.sqlite3.db?mode=rwc` or a `mysql://...` URL). Same rule as `TRACKER__TOKEN`: shipped TOMLs no longer carry a default, so absent both env var and operator-supplied TOML the application fails to start with `missing field 'connect_url'`.
- `TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PRIVATE_KEY_PATH` - Path to an RSA private key PEM file for JWT signing. Optional: without this, ephemeral auto-generated keys are used (sessions will not survive restarts).
- `TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PUBLIC_KEY_PATH` - Path to an RSA public key PEM file for JWT verification. Required when `PRIVATE_KEY_PATH` is set.
- `TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PRIVATE_KEY_PEM` - Inline RSA private key PEM string (alternative to file path). Optional: for persistent sessions.
- `TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PUBLIC_KEY_PEM` - Inline RSA public key PEM string (alternative to file path). Required when `PRIVATE_KEY_PEM` is set.
- `TORRUST_INDEX_DATABASE_DRIVER` - **Input-validation gate only** (options: `sqlite3`, `mysql`, default `sqlite3`). Per ADR-T-009 §7.4, this env var was originally introduced to choose which driver-suffixed default TOML to seed into `/etc/torrust/index/` on first boot. Phase 9 then collapsed those two driver-suffixed defaults into a single driver-agnostic `index.container.toml` (Phase 5 had already made `database.connect_url` mandatory, so the file no longer encodes a driver choice). The env var is retained as an input-validation gate so a typo (`postgres`, `Sqlite`, …) fails fast at container start rather than silently propagating; both supported values now seed the same template. Runtime database decisions are taken from the config probe's `database.driver` field, derived from `database.connect_url`'s URL scheme. Note the taxonomy difference — the env var uses `sqlite3` / `mysql`; the probe (and the application) emit `sqlite` / `mysql`. Operators who scripted around this env var expecting it to control runtime behaviour must update their scripts to supply `TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL` instead.
- `TORRUST_INDEX_CONFIG_TOML` - Load config from this environmental variable instead from a file, (i.e: `TORRUST_INDEX_CONFIG_TOML=$(cat index-index.toml)`).
- `USER_ID` - The user id for the runtime-created `torrust` user. Must be a non-negative integer and must not be `0`. Should match the ownership of the host-mapped volumes (default `1000`).
- `API_PORT` - The port for the index API. This should match the port used in the configuration, (default `3001`).
- `IMPORTER_API_PORT` - The port for the importer API. This should match the port used in the configuration, (default `3002`).
- `TZ` - Container time zone passed through to the runtime (default `Etc/UTC`). Set in the Containerfile `ENV` block; override at `docker run` time if you need wall-clock log timestamps in a specific zone.

> NOTE: `API_PORT` and `IMPORTER_API_PORT` are runtime `ENV` values, not
> build-time `ARG`s. Overriding them at `docker run` / `podman run` time
> with `--env API_PORT=…` correctly reaches the application listener and
> the in-container `HEALTHCHECK`, but the `EXPOSE` directive in the
> `Containerfile` is evaluated at build time and bakes the *defaults*
> (`3001`, `3002`) into image metadata. Tools that read that metadata
> (`docker inspect`, `docker port`) will continue to report the defaults
> regardless of any `--env` override. Use `--publish host:container` to
> map whichever container port the application is actually listening on.

### Sockets

Socket ports used internally within the container can be mapped to with the `--publish` argument.

The format is: `--publish [optional_host_ip]:[host_port]:[container_port]/[optional_protocol]`, for example: `--publish 127.0.0.1:8080:80/tcp`.

The default ports can be mapped with the following:

```s
--publish 0.0.0.0:3001:3001/tcp
```

> NOTE: Inside the container it is necessary to expose a socket with the wildcard address `0.0.0.0` so that it may be accessible from the host. Verify that the configuration that the sockets are wildcard.

### Mapped Volumes

By default the container will install volumes for `/var/lib/torrust/index`, `/var/log/torrust/index`, and `/etc/torrust/index`, however for better administration it good to make these volumes host-mapped.

The argument to host-map volumes is `--volume`, with the format: `--volume=[host-src:]container-dest[:<options>]`.

The default mapping can be supplied with the following arguments:

```s
--volume ./storage/index/lib:/var/lib/torrust/index:Z \
--volume ./storage/index/log:/var/log/torrust/index:Z \
--volume ./storage/index/etc:/etc/torrust/index:Z \
```

Please not the `:Z` at the end of the podman `--volume` mapping arguments, this is to give read-write permission on SELinux enabled systemd, if this doesn't work on your system, you can use `:rw` instead.

## Complete Example

### With Docker

```sh
## Setup Docker Default Context
docker context use default

## Build Container Image
docker build --target release --tag torrust-index:release --file Containerfile .

## Setup Mapped Volumes
mkdir -p ./storage/index/lib/ ./storage/index/log/ ./storage/index/etc/

## Run Torrust Index Container Image
## Note: Without key path env vars, ephemeral auto-generated keys are used.
## For persistent sessions, supply your own RSA key pair:
##   --env TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PRIVATE_KEY_PATH="/var/lib/torrust/index/jwt/private.pem" \
##   --env TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PUBLIC_KEY_PATH="/var/lib/torrust/index/jwt/public.pem" \
docker run -it \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="MySecretToken" \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL="sqlite:///var/lib/torrust/index/database/index.sqlite3.db?mode=rwc" \
    --env USER_ID="$(id -u)" \
    --publish 0.0.0.0:3001:3001/tcp \
    --volume ./storage/index/lib:/var/lib/torrust/index:Z \
    --volume ./storage/index/log:/var/log/torrust/index:Z \
    --volume ./storage/index/etc:/etc/torrust/index:Z \
    torrust-index:release
```

### With Podman

```sh
## Build Container Image
podman build --format docker --target release --tag torrust-index:release --file Containerfile .

## Setup Mapped Volumes
mkdir -p ./storage/index/lib/ ./storage/index/log/ ./storage/index/etc/

## Run Torrust Index Container Image
podman run -it \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="MySecretToken" \
    --env TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL="sqlite:///var/lib/torrust/index/database/index.sqlite3.db?mode=rwc" \
    --env USER_ID="$(id -u)" \
    --publish 0.0.0.0:3001:3001/tcp \
    --volume ./storage/index/lib:/var/lib/torrust/index:Z \
    --volume ./storage/index/log:/var/log/torrust/index:Z \
    --volume ./storage/index/etc:/etc/torrust/index:Z \
    torrust-index:release
```

## Compose Split

Per ADR-T-009 §8, the repository ships two Compose files
with a clear separation between *production-shaped baseline*
and *dev sandbox*:

- [`compose.yaml`](../compose.yaml) — **deployment template.**
  Production-shaped: no `mailcatcher` sidecar, no `tty`,
  external ports bound to `127.0.0.1` (except the index API
  on `:3001`), and credentials referenced as bare `${VAR}`
  with no defaults. This is the file operators copy as a
  starting point for real deployments.
- [`compose.override.yaml`](../compose.override.yaml) —
  **for development.** Auto-loaded by Compose v2 (i.e. by a
  plain `docker compose up` and by `make up-dev`). Adds the
  `mailcatcher` sidecar, allocates TTYs on `index` /
  `tracker`, and supplies permissive `${VAR:-default}`
  defaults for the credentials the baseline leaves blank.

Two top-level [`Makefile`](../Makefile) targets wrap the two
documented invocation paths:

```sh
# Dev sandbox: auto-loads compose.override.yaml.
make up-dev

# Production-shaped: validates required credentials, then
# runs `docker compose --file compose.yaml up -d --wait`
# (override excluded). Required env vars:
#
#   USER_ID                                              (numeric host UID owning ./storage)
#   TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN
#   TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL
#   TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN
#   MYSQL_ROOT_PASSWORD  (only if the local mysql sidecar is in use)
make up-prod
```

`make up-prod` is fail-fast convenience — defence in depth,
not the only line. The container's config probe (ADR-T-009
§6) is the authoritative gate: it rejects empty
`connect_url` (exit 3) and empty `tracker.token` (exit 4)
regardless of how Compose was invoked.

A developer running `docker compose -f compose.yaml up`
(deliberately bypassing the override) gets bare `${VAR}`
substitution to empty strings; the config probe catches this
inside the container, so the worst case is a clear startup
failure rather than silent misbehaviour.

## Runtime Image Notes

### Command-Line Output Contract

The `torrust-index` server process is a no-stdout command. Once the Rust
application starts, its tracing diagnostics are JSON records on stderr. The
configured `[logging].threshold` selects the default filter, and a non-empty
`RUST_LOG` environment variable overrides that default. The server does not
refuse terminal stdout, because it does not emit stdout result data.

Command-reachable server libraries follow that same stream contract. Shutdown
grace-period notices are structured tracing records, and mail-template
initialization or rendering failures are returned to callers for JSON diagnostic
reporting rather than printed or handled by exiting from the mailer library.

Container helper binaries follow the ADR-T-010 stdout/stderr split. When they
emit result data, stdout is exactly one JSON object with a trailing newline and
a top-level `schema` field. Diagnostics, help, version output, argv errors, TTY
refusal, and panic reports are emitted on stderr as JSON records.

The stdout-producing helpers are:

- `torrust-index-auth-keypair`: `schema`, `private_key_pem`, `public_key_pem`.
- `torrust-index-config-probe`: `schema`, `database`, `auth`.
- `torrust-index-health-check`: `schema`, `target`, `status`, `elapsed_ms`.

These helpers refuse to write stdout result data directly to a terminal. Pipe or
redirect the result instead:

```sh
torrust-index-auth-keypair | jq .
torrust-index-config-probe | jq .
torrust-index-health-check http://127.0.0.1:3001/health_check | jq .
```

For helper diagnostics, a non-empty `RUST_LOG` environment variable takes
precedence over `--debug`; otherwise `--debug` raises the helper's default
diagnostic filter to debug. Help and version requests do not emit stdout result
data and therefore do not trigger TTY refusal; they write JSON control-plane
records to stderr and exit with code 0.

The container entry script captures helper stdout internally and does not
forward it to the terminal. Before it execs the application, the entry script is
a no-stdout orchestration command: validation failures, status notices, expected
utility failures, `jq` parsing failures, unexpected shell exits, and `DEBUG=1`
phase records are emitted on stderr as one JSON object per line. The records use
the shared `schema`, `command`, `kind`, `message`, and `fields` shape; the
entry-script command name is `torrust-index-entry-script`.

Expected utility stderr captured by the script is carried inside JSON fields
instead of being forwarded as top-level stream text. Helper stdout remains inside
command substitutions. File writes to `/etc/motd` and `/etc/profile` are not
stream output.

### Healthcheck (both targets)

Both `release` and `debug` ship the same two-probe `HEALTHCHECK`
block, invoking `torrust-index-health-check` against the index API
and the importer API in turn. The healthcheck binary is itself
`0500 root:root`; the `HEALTHCHECK` directive runs as root (no
`--user` flag in the directive), so the unprivileged `torrust` user
cannot invoke it directly.

If you build with Podman without `--format docker`, the directive is
silently dropped at build time (see the build section above) and the
image will report no health status. `docker build` is unaffected.

### Available Shell Commands (Busybox Subset)

The two build targets ship deliberately different shell
footprints.

**`release` target.** Built on the lean
`gcr.io/distroless/cc-debian13` base. Ships a single
`/bin/busybox` binary (mode `0700 root:root`) plus a curated
set of applet symlinks pointing at it. The unprivileged
`torrust` user that the application runs as gets `EACCES`
on `/bin/busybox` (and therefore on every applet symlink)
after privilege drop — the busybox tree is reachable only
by root. The curated symlink set covers exactly the applets
the entry script needs at first boot:

- `sh`, `adduser`, `addgroup`, `install`, `mkdir`, `dirname`,
  `chown`, `chmod`, `tr`, `mktemp`, `cat`, `printf`, `rm`,
  `echo`, `grep`

`su-exec` is a separate root-only binary at `/bin/su-exec`,
not a busybox applet. `jq` is a separate root-only binary at
`/usr/bin/jq` used by the entry script's JSON diagnostics,
config-probe parsing, and auth-keypair bootstrap. None of
these are reachable by the unprivileged `torrust` user.

There is no `/busybox/` directory in the release image — the
full busybox applet tree from the upstream `:debug`
distroless image is deliberately not present, so absolute-path
invocations like `/busybox/ls` cannot bypass the curated
subset.

For emergency operational debugging, `docker exec -u root …
sh` still works on the release image (the curated `/bin/sh`
resolves through PATH to `/bin/busybox`, and `0700 root:root`
permits root invocation). This is the documented break-glass
procedure.

**`debug` target.** Built on `gcr.io/distroless/cc-debian13:debug`,
which ships the upstream full busybox tree at `/busybox/`
with default world-executable permissions. The debug image
leaves that tree in place and puts `/busybox/` on `PATH` so
the unprivileged user retains access to the complete applet
set (`id`, `whoami`, `ps`, `grep`, `wget`, …). This is the
debug image's purpose; use it whenever you need an
interactive shell as the application user.

### Entry Script Debugging

The container entry script does not produce verbose output by default.
To emit JSON phase records for startup troubleshooting, set the `DEBUG`
environment variable:

```sh
--env DEBUG=1
```

Debug records are written to stderr as ADR-T-010 JSON status records with
`fields.level = "debug"`. The script no longer enables shell tracing with
`set -x`, so automation should parse stderr as NDJSON rather than scrape shell
trace lines.

The entry script also runs under `set -eu` (POSIX `errexit` +
`nounset`): any unchecked command failure aborts startup
immediately, and references to unset variables are treated as
errors. This converts a class of silent-misconfiguration bugs
into loud, actionable startup failures.

### Entry Script Contract

Per ADR-T-009 §7, the entry script reads its configuration in
the following order. Each step depends on the values
resolved by previous steps.

The complete list of environment variables the entry script
consults is maintained as a canonical manifest comment block
in [`share/container/entry_script_sh`](../share/container/entry_script_sh),
delimited by the `# ENTRY_ENV_VARS:` and
`# END_ENTRY_ENV_VARS` sentinel lines. CI verifies that
every variable named between those sentinels is documented
in this file (see ADR-T-009 Acceptance Criterion #7);
when adding or removing a variable from the entry script,
update both the manifest block and the env-var section
above.

1. **`USER_ID`** — numeric, non-zero (refuses to run as
   root). Default `1000`. Used to create the unprivileged
   `torrust` user via `adduser` and to chown the volume
   directories.
2. **`TORRUST_INDEX_DATABASE_DRIVER`** — validated as an
   input-only gate (`sqlite3` or `mysql`); both values
   now seed the same driver-agnostic
   `index.container.toml` template into
   `/etc/torrust/index/index.toml` on first boot. Unset
   or unrecognised values abort startup with an explicit
   error. See the env-var entry above for full scope and
   migration notes.
3. **`RUNTIME`** — selects the message-of-the-day banner
   (`runtime`, `debug`, or `release`). Set by the
   Containerfile per image variant.
4. **Config probe (`/usr/bin/torrust-index-config-probe`)**
   — invoked as root after the default TOML is in place.
   The probe is the same loader the application uses, so it
   sees the operator's full TOML + env-var stack. Its JSON
  output (`schema`, `database`, `auth`) is consumed by `jq`
  and drives the remaining steps. The probe runs *before* the script exports any
   `TORRUST_INDEX_CONFIG_OVERRIDE_*` of its own, so its
   output reflects only operator-supplied values.
5. **`TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__{PRIVATE,PUBLIC}_KEY_{PEM,PATH}`**
   — the probe reports raw presence and resolved source for
   each key. The script enforces three invariants post-probe:
   PEM and PATH are mutually exclusive within a single key;
   both keys must be configured or neither (no half-pair);
   and both keys must use the same delivery mechanism (no
   mixed PEM/PATH across the pair). When neither key is
   configured anywhere (probe `source=none`), the script
   applies the container defaults
   (`/etc/torrust/index/auth/private.pem` and
   `.../public.pem`) and **exports the corresponding
   `..._AUTH__*_PATH` override env var** so the application
   sees the same path the script materialises. The script
   is the single source of truth for these defaults; there
   is no constant duplicated between two files to drift
   apart.
6. **`database.connect_url`** (resolved by the probe) —
   the probe's `database.driver` field selects the seeding
   dispatch (`sqlite` seeds the default DB file; `mysql`
   is a no-op since the application connects directly).
   For SQLite the seed is materialised at the resolved path
   only when the parent directory lives under one of the
   managed volumes (`/etc/torrust/index/`,
   `/var/lib/torrust/index/`, `/var/log/torrust/index/`);
   paths outside those roots must be pre-created by the
   operator.
7. **`exec /bin/su-exec torrust ...`** — drops privileges
   and execs the application (or the supplied `CMD`).

#### Required Overrides Between Phases

The default TOML shipped at
`/usr/share/torrust/default/config/index.container.toml`
intentionally leaves `[tracker]` and `[database]` empty so
operators must supply real values. Per ADR-T-009 §D2 the
schema requires `tracker.token` and
`database.connect_url` — the config probe will exit non-zero
(codes 3/4) and the entry script will abort startup if
either is missing. Supply them via:

```sh
--env TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN=...
--env TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL=sqlite:///var/lib/torrust/index/database/index.sqlite3.db?mode=rwc
```

or by mounting a populated `index.toml` at
`/etc/torrust/index/index.toml`. The
[`compose.override.yaml`](../compose.override.yaml) shipped
in the repo wires both overrides for the local dev workflow
(`make up-dev`); see [Compose Split](#compose-split).

#### Runtime `jq` Dependency

Both runtime images ship a root-only `/usr/bin/jq` (mode
`0500 root:root`, sourced from a pristine `rust:slim-trixie`
`jq_donor` build stage in the Containerfile). It is invoked
only during the entry script's pre-`su-exec` phase to emit
properly escaped JSON diagnostics, parse the config probe's
JSON output, and split the auth-keypair helper's JSON output.
If `jq` is unavailable, the script emits a fixed minimal JSON
diagnostic before exiting. The unprivileged `torrust` user has
no access to `/usr/bin/jq` after privilege drop.

Operators who run the helpers manually should use the same pattern: pipe stdout
result data to `jq`, redirect it to a file, or capture it from another process.
Direct terminal stdout is refused by design.

#### Sourced Shell Library

The entry script's reusable POSIX `sh` helpers live in a
separate library shipped at
`/usr/local/lib/torrust/entry_script_lib_sh` (mode
`0444 root:root`, sourced — not exec'd). Splitting them
out lets the workspace test crate
[`packages/index-entry-script/`](../packages/index-entry-script/)
drive helpers such as `validate_auth_keys` and `seed_sqlite`
through a host `sh` subprocess and assert their exit-code and
JSON stderr contracts. The same library owns the entry script's
JSON control-plane emitters, checked utility wrappers, `jq`
read/write helpers, and unexpected-exit trap. It has no top-level
side effects, so sourcing it from either the entry script or a test
harness is safe.
