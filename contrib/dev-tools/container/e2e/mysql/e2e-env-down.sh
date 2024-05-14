#!/bin/bash

TORRUST_INDEX_CONFIG=$(cat ./share/default/config/index.e2e.container.mysql.toml) \
    TORRUST_TRACKER_CONFIG_TOML=$(cat ./share/default/config/tracker.e2e.container.sqlite.toml) \
    docker compose down
