#!/bin/bash

TORRUST_INDEX_CONFIG=$(cat ./share/default/config/index.container.mysql.toml) \
    docker compose build

USER_ID=${USER_ID:-1000} \
    # Index
    TORRUST_INDEX_CONFIG=$(cat ./share/default/config/index.container.mysql.toml) \
    TORRUST_INDEX_DATABASE_DRIVER="mysql" \
    TORRUST_INDEX_TRACKER_API_TOKEN="MyAccessToken" \
    TORRUST_IDX_BACK_MYSQL_DATABASE="torrust_index_e2e_testing" \
    # Tracker
    TORRUST_TRACKER_CONFIG=$(cat ./share/default/config/tracker.container.mysql.toml) \
    TORRUST_TRACKER_DATABASE_DRIVER="mysql" \
    TORRUST_TRACKER_API_ADMIN_TOKEN="MyAccessToken" \
    docker compose up -d
