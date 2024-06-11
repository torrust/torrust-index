#!/bin/bash

TORRUST_INDEX_CONFIG_TOML=$(cat ./share/default/config/index.private.e2e.container.sqlite3.toml) \
TORRUST_TRACKER_CONFIG_TOML=$(cat ./share/default/config/tracker.private.e2e.container.sqlite3.toml) \
    docker compose down
