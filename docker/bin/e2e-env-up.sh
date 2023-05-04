#!/bin/bash

TORRUST_IDX_BACK_USER_UID=${TORRUST_IDX_BACK_USER_UID:-1000} \
    docker compose build

TORRUST_IDX_BACK_USER_UID=${TORRUST_IDX_BACK_USER_UID:-1000} \
    TORRUST_IDX_BACK_CONFIG=$(cat config-idx-back.local.toml) \
    TORRUST_TRACKER_CONFIG=$(cat config-tracker.local.toml) \
    TORRUST_TRACKER_API_TOKEN=${TORRUST_TRACKER_API_TOKEN:-MyAccessToken} \
    docker compose up -d
