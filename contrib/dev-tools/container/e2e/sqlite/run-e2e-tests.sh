#!/bin/bash

# Guard: refuse to run unless we are at the top of a `torrust-index`
# git checkout. The script does destructive things (wipes `./storage`,
# overwrites `.env`, builds container images tagged `torrust_index`)
# that are only safe inside the project root, and the bind-mount
# paths in `compose.yaml` are all relative to it.
if ! command -v git >/dev/null 2>&1; then
    echo "error: 'git' is required to verify the working directory" >&2
    exit 1
fi
GIT_TOPLEVEL="$(git rev-parse --show-toplevel 2>/dev/null)" || {
    echo "error: not inside a git repository" >&2
    exit 1
}
if [ "$GIT_TOPLEVEL" != "$PWD" ]; then
    echo "error: must be run from the repository root ($GIT_TOPLEVEL), not $PWD" >&2
    exit 1
fi
# Identify the repo by the presence of the root crate's Cargo.toml
# rather than by remote URL — that keeps forks and local clones
# (with arbitrary remote names) working.
if ! grep -qE '^name = "torrust-index"$' Cargo.toml 2>/dev/null; then
    echo "error: this does not look like the torrust-index repository (Cargo.toml package name mismatch)" >&2
    exit 1
fi

CURRENT_USER_NAME=$(whoami)
CURRENT_USER_ID=$(id -u)
echo "User name: $CURRENT_USER_NAME"
echo "User   id: $CURRENT_USER_ID"

# Make rustup-managed toolchains visible. Distros that ship rust as
# a system package put `cargo` on the default PATH; rustup installs
# (the upstream-recommended path) put it under `~/.cargo/bin`, which
# is only added to PATH by `~/.cargo/env` — and that's only sourced
# in interactive login shells. Prepend it unconditionally so the
# script works regardless of the invoking shell's setup.
if [ -d "$HOME/.cargo/bin" ]; then
    PATH="$HOME/.cargo/bin:$PATH"
    export PATH
fi
if ! command -v cargo >/dev/null 2>&1; then
    echo "error: 'cargo' not found on PATH (looked in \$PATH and \$HOME/.cargo/bin)" >&2
    exit 1
fi

USER_ID=$CURRENT_USER_ID
export USER_ID

export TORRUST_INDEX_DATABASE="e2e_testing_sqlite3"
export TORRUST_TRACKER_DATABASE="e2e_testing_sqlite3"

# Wipe any storage left over from a previous run.
#
# Under rootless podman, files written from inside a container are
# owned by a sub-UID from /etc/subuid (e.g. host-UID 525287 for
# container-UID 1000), which a plain `rm -rf` cannot remove from the
# host shell. `podman unshare` re-enters that user namespace where
# we *are* root and can delete freely. Detect the actual provider
# behind the `docker` CLI rather than trusting the binary name —
# `podman-docker` ships a `docker` shim that wraps podman.
if [ -d ./storage ]; then
    if command -v podman >/dev/null 2>&1 && \
       { ! command -v docker >/dev/null 2>&1 || \
         docker --version 2>/dev/null | grep -qi podman; }; then
        echo "Cleaning ./storage via 'podman unshare' (rootless userns)"
        podman unshare rm -rf ./storage || exit 1
    else
        echo "Cleaning ./storage"
        rm -rf ./storage || exit 1
    fi
fi

# Install tool to create torrent files.
# It's needed by some tests to generate and parse test torrent files.
# Skip the (slow) `cargo install` when `imdl` is already on PATH.
if ! command -v imdl >/dev/null 2>&1; then
    cargo install --locked imdl || exit 1
fi

# Install app (no docker) that will run the test suite against the E2E testing
# environment (in docker).
cp .env.local .env || exit 1
./contrib/dev-tools/container/e2e/sqlite/install.sh || exit 1

# Compose v2 (`docker compose`) names containers with hyphens
# (`torrust-mysql-1`); `podman-compose` v1.x preserves the legacy
# `_` separator (`torrust_mysql_1`). Probe once and let downstream
# wait-for-healthy calls reuse the result.
detect_container_name_sep() {
    local svc=mysql
    if docker ps --format '{{.Names}}' | grep -q "^torrust-${svc}-1$"; then
        echo '-'
    elif docker ps --format '{{.Names}}' | grep -q "^torrust_${svc}_1$"; then
        echo '_'
    else
        # Sensible default for Compose v2.
        echo '-'
    fi
}

# TEST USING SQLITE
echo "Running E2E tests using SQLite ..."

# TEST USING A PUBLIC TRACKER
echo "Running E2E tests with a public tracker ..."

# Start E2E testing environment
./contrib/dev-tools/container/e2e/sqlite/mode/public/e2e-env-up.sh || exit 1

SEP=$(detect_container_name_sep)

# Wait for conatiners to be healthy
./contrib/dev-tools/container/functions/wait_for_container_to_be_healthy.sh "torrust${SEP}mysql${SEP}1" 10 3 || exit 1
./contrib/dev-tools/container/functions/wait_for_container_to_be_healthy.sh "torrust${SEP}tracker${SEP}1" 10 3 || exit 1
./contrib/dev-tools/container/functions/wait_for_container_to_be_healthy.sh "torrust${SEP}index${SEP}1" 10 3 || exit 1

# Just to make sure that everything is up and running
docker ps

# Run E2E tests with shared app instance
#
# The e2e config TOML intentionally omits `tracker.token` and
# `database.connect_url` (operators are expected to supply them via env
# overrides; see ADR-T-009 §D2). Inject host-side overrides so the test
# process can load the same config file the container uses.
TORRUST_INDEX_E2E_SHARED=true \
    TORRUST_INDEX_CONFIG_TOML_PATH="./share/default/config/index.public.e2e.container.toml" \
    TORRUST_INDEX_E2E_DB_CONNECT_URL="sqlite://./storage/index/lib/database/e2e_testing_sqlite3.db?mode=rwc" \
    TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="MyAccessToken" \
    TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL="sqlite://./storage/index/lib/database/e2e_testing_sqlite3.db?mode=rwc" \
    cargo test ||
    {
        ./contrib/dev-tools/container/e2e/sqlite/mode/public/e2e-env-down.sh
        exit 1
    }

# Stop E2E testing environment
./contrib/dev-tools/container/e2e/sqlite/mode/public/e2e-env-down.sh || exit 1

# TEST USING A PRIVATE TRACKER
echo "Running E2E tests with a private tracker ..."

# Start E2E testing environment
./contrib/dev-tools/container/e2e/sqlite/mode/private/e2e-env-up.sh || exit 1

SEP=$(detect_container_name_sep)

# Wait for conatiners to be healthy
./contrib/dev-tools/container/functions/wait_for_container_to_be_healthy.sh "torrust${SEP}mysql${SEP}1" 10 3 || exit 1
./contrib/dev-tools/container/functions/wait_for_container_to_be_healthy.sh "torrust${SEP}tracker${SEP}1" 10 3 || exit 1
./contrib/dev-tools/container/functions/wait_for_container_to_be_healthy.sh "torrust${SEP}index${SEP}1" 10 3 || exit 1

# Just to make sure that everything is up and running
docker ps

# Run E2E tests with shared app instance
#
# Same rationale as above — supply mandatory `tracker.token` and
# `database.connect_url` via env overrides for the host-side test
# process (ADR-T-009 §D2).
TORRUST_INDEX_E2E_SHARED=true \
    TORRUST_INDEX_CONFIG_TOML_PATH="./share/default/config/index.private.e2e.container.sqlite3.toml" \
    TORRUST_INDEX_E2E_DB_CONNECT_URL="sqlite://./storage/index/lib/database/e2e_testing_sqlite3.db?mode=rwc" \
    TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="MyAccessToken" \
    TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL="sqlite://./storage/index/lib/database/e2e_testing_sqlite3.db?mode=rwc" \
    cargo test ||
    {
        ./contrib/dev-tools/container/e2e/sqlite/mode/private/e2e-env-down.sh
        exit 1
    }

# Stop E2E testing environment
./contrib/dev-tools/container/e2e/sqlite/mode/private/e2e-env-down.sh || exit 1
