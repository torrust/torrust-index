#!/bin/bash

TORRUST_INDEX_CONFIG_TOML=$(cat ./share/default/config/index.public.e2e.container.sqlite3.toml) \
    docker compose build

USER_ID=${USER_ID:-1000} \
    TORRUST_INDEX_CONFIG_TOML=$(cat ./share/default/config/index.public.e2e.container.sqlite3.toml) \
    TORRUST_INDEX_DATABASE="e2e_testing_sqlite3" \
    TORRUST_INDEX_DATABASE_DRIVER="sqlite3" \
    TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN="MyAccessToken" \
    TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__SECRET_KEY="MaxVerstappenWC2021" \
    TORRUST_TRACKER_CONFIG_TOML=$(cat ./share/default/config/tracker.public.e2e.container.sqlite3.toml) \
    TORRUST_TRACKER_DATABASE="e2e_testing_sqlite3" \
    TORRUST_TRACKER_CONFIG_OVERRIDE_DB_DRIVER="Sqlite3" \
    TORRUST_TRACKER_CONFIG_OVERRIDE_HTTP_API__ACCESS_TOKENS__ADMIN="MyAccessToken" \
    docker compose up --detach --pull always --remove-orphans
