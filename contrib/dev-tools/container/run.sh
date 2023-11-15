#!/bin/bash

USER_ID=${USER_ID:-1000}
TORRUST_INDEX_CONFIG=$(cat config.toml)

docker run -it \
    --user="$USER_ID" \
    --publish 3001:3001/tcp \
    --env TORRUST_INDEX_CONFIG="$TORRUST_INDEX_CONFIG" \
    --volume "$(pwd)/storage":"/app/storage" \
    torrust-index
