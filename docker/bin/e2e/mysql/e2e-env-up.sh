#!/bin/bash

docker compose build


TORRUST_INDEX_CONFIG=$(cat config-index.mysql.local.toml) \
TORRUST_INDEX_MYSQL_DATABASE="torrust_index_backend_e2e_testing" \
TORRUST_TRACKER_CONFIG=$(cat config-tracker.local.toml) \
TORRUST_TRACKER_API_TOKEN=${TORRUST_TRACKER_API_TOKEN:-MyAccessToken} \
docker compose up -d

