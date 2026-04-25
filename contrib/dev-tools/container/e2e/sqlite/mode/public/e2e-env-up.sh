#!/bin/bash

# `BUILDAH_FORMAT=docker` keeps the Containerfile's `HEALTHCHECK`
# directive when the build runs under podman: the default OCI image
# format silently drops it (`WARN ... HEALTHCHECK is not supported
# for OCI image format and will be ignored`), which would leave
# `torrust-index-1` permanently un-healthy and break the
# `wait_for_container_to_be_healthy.sh` gate downstream.
export BUILDAH_FORMAT=docker

# Pre-create the bind-mount source directories. Docker's daemon will
# auto-create missing host paths, but podman does not — and with the
# `:Z` SELinux relabel suffix in `compose.yaml`, a missing source path
# fails the container start with `getxattr ... no such file or
# directory` rather than the friendlier "bind source path does not
# exist" error. Creating them up front keeps the script portable.
mkdir -p \
    ./storage/index/lib/database \
    ./storage/index/log \
    ./storage/index/etc \
    ./storage/tracker/lib/database \
    ./storage/tracker/log \
    ./storage/tracker/etc

TORRUST_INDEX_CONFIG_TOML=$(cat ./share/default/config/index.public.e2e.container.toml) \
    docker compose build

# Pull externally-sourced service images up front. We deliberately
# avoid `docker compose up --pull=always`: under podman-compose,
# `--pull` is a boolean flag (paired with a separate `--pull-always`),
# so the Compose-v2 `--pull=always` syntax is rejected. A standalone
# `docker compose pull` is accepted by both implementations; the
# `|| true` keeps the script resilient to transient registry hiccups
# and to images that only exist locally (e.g. `localhost/torrust_index`).
docker compose pull || true

USER_ID=${USER_ID:-1000} \
    TORRUST_INDEX_CONFIG_TOML=$(cat ./share/default/config/index.public.e2e.container.toml) \
    TORRUST_INDEX_DATABASE="e2e_testing_sqlite3" \
    TORRUST_INDEX_DATABASE_DRIVER="sqlite3" \
    TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="MyAccessToken" \
    TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL="sqlite:///var/lib/torrust/index/database/e2e_testing_sqlite3.db?mode=rwc" \
    TORRUST_TRACKER_CONFIG_TOML=$(cat ./share/default/config/tracker.public.e2e.container.sqlite3.toml) \
    TORRUST_TRACKER_DATABASE="e2e_testing_sqlite3" \
    TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER="sqlite3" \
    TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN="MyAccessToken" \
    docker compose up --detach --remove-orphans
