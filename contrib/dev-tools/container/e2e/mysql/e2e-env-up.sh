#!/bin/bash

TORRUST_INDEX_CONFIG=$(cat ./share/default/config/index.e2e.container.mysql.toml) \
    docker compose build

USER_ID=${USER_ID:-1000} \
    TORRUST_INDEX_CONFIG=$(cat ./share/default/config/index.e2e.container.mysql.toml) \
    TORRUST_INDEX_DATABASE="torrust_index_e2e_testing" \
    TORRUST_INDEX_DATABASE_DRIVER="mysql" \
    TORRUST_INDEX_TRACKER_API_TOKEN="MyAccessToken" \
    TORRUST_INDEX_MYSQL_DATABASE="torrust_index_e2e_testing" \
    TORRUST_TRACKER_CONFIG=$(cat ./share/default/config/tracker.e2e.container.sqlite3.toml) \
    TORRUST_TRACKER_DATABASE="e2e_testing_sqlite3" \
    TORRUST_TRACKER_DATABASE_DRIVER="sqlite3" \
    TORRUST_TRACKER_API_ADMIN_TOKEN="MyAccessToken" \
    docker compose up --detach --pull always --remove-orphans
