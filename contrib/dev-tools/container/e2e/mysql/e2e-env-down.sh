#!/bin/bash

TORRUST_INDEX_CONFIG=$(cat ./share/default/config/index.container.mysql.toml) \
    TORRUST_TRACKER_CONFIG=$(cat ./share/default/config/tracker.container.mysql.toml) \
    docker compose down

