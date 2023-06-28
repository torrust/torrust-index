#!/bin/bash

TORRUST_IDX_BACK_USER_UID=${TORRUST_IDX_BACK_USER_UID:-1000}
TORRUST_IDX_BACK_CONFIG=$(cat config.toml)

docker run -it \
    --user="$TORRUST_IDX_BACK_USER_UID" \
    --publish 3001:3001/tcp \
    --env TORRUST_IDX_BACK_CONFIG="$TORRUST_IDX_BACK_CONFIG" \
    --volume "$(pwd)/storage":"/app/storage" \
    torrust-index-backend
