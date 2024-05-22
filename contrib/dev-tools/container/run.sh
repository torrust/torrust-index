#!/bin/bash

USER_ID=${USER_ID:-1000}
TORRUST_INDEX_CONFIG_TOML=$(cat config.toml)

docker run -it \
    --user="$USER_ID" \
    --publish 3001:3001/tcp \
    --env TORRUST_INDEX_CONFIG_TOML="$TORRUST_INDEX_CONFIG_TOML" \
    --volume "$(pwd)/storage":"/app/storage" \
    torrust-index
