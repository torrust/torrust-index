#!/bin/bash

TORRUST_INDEX_USER_UID=${TORRUST_INDEX_USER_UID:-1000}
TORRUST_INDEX_CONFIG=$(cat config.toml)

docker run -it \
    --user="$TORRUST_INDEX_USER_UID" \
    --publish 3001:3001/tcp \
    --env TORRUST_INDEX_CONFIG="$TORRUST_INDEX_CONFIG" \
    --volume "$(pwd)/lib/torrust":"/var/lib/torrust" \
    torrust-index-backend
