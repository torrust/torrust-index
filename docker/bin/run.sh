#!/bin/bash

TORRUST_INDEX_CONFIG=$(cat config.toml)

docker run -it \
    --publish 3001:3001/tcp \
    --env TORRUST_INDEX_CONFIG="$TORRUST_INDEX_CONFIG" \
    --volume "$(pwd)/source":"/var/lib/torrust" \
    --entrypoint "torrust-index-backend" \
    torrust-index-backend:debug
