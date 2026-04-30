#!/bin/bash

# See the public-mode counterpart for the rationale behind
# `BUILDAH_FORMAT=docker` and `--pull=always` (rather than
# `--pull always`).
export BUILDAH_FORMAT=docker

# See the public-mode counterpart for why we pre-create these.
mkdir -p \
    ./storage/index/lib/database \
    ./storage/index/log \
    ./storage/index/etc \
    ./storage/tracker/lib/database \
    ./storage/tracker/log \
    ./storage/tracker/etc

TORRUST_INDEX_CONFIG_TOML=$(cat ./share/default/config/index.private.e2e.container.sqlite3.toml) \
    docker compose build

# See the public-mode counterpart for why we pull separately
# rather than passing `--pull=always` to `docker compose up`.
docker compose pull || true

USER_ID=${USER_ID:-1000} \
    TORRUST_INDEX_CONFIG_TOML=$(cat ./share/default/config/index.private.e2e.container.sqlite3.toml) \
    TORRUST_INDEX_DATABASE="e2e_testing_sqlite3" \
    TORRUST_INDEX_DATABASE_DRIVER="sqlite3" \
    TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="MyAccessToken" \
    TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL="sqlite:///var/lib/torrust/index/database/e2e_testing_sqlite3.db?mode=rwc" \
    TORRUST_TRACKER_CONFIG_TOML=$(cat ./share/default/config/tracker.private.e2e.container.sqlite3.toml) \
    TORRUST_TRACKER_DATABASE="e2e_testing_sqlite3" \
    TORRUST_TRACKER_CONFIG_OVERRIDE_CORE__DATABASE__DRIVER="sqlite3" \
    TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN="MyAccessToken" \
    docker compose up --detach --remove-orphans
