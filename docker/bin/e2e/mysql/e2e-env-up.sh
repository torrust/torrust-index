#!/bin/bash

TORRUST_IDX_BACK_USER_UID=${TORRUST_IDX_BACK_USER_UID:-1000} \
    docker compose build

TORRUST_IDX_BACK_USER_UID=${TORRUST_IDX_BACK_USER_UID:-1000} \
    TORRUST_IDX_BACK_CONFIG=$(cat config-idx-back.mysql.local.toml) \
    TORRUST_IDX_BACK_MYSQL_DATABASE="torrust_index_backend_e2e_testing" \
    TORRUST_TRACKER_CONFIG=$(cat config-tracker.local.toml) \
    TORRUST_TRACKER_DATABASE_DRIVER=${TORRUST_TRACKER_DATABASE_DRIVER:-mysql} \
    TORRUST_TRACKER_API_ADMIN_TOKEN=${TORRUST_TRACKER_API_ADMIN_TOKEN:-MyAccessToken} \
    docker compose up -d

