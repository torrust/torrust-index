#!/bin/bash

TORRUST_INDEX_CONFIG=$(cat ./share/default/config/index.container.sqlite3.toml) \
    TORRUST_TRACKER_CONFIG=$(cat ./share/default/config/tracker.container.sqlite3.toml) \
    docker compose down
