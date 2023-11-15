#!/bin/bash

USER_ID=${USER_ID:-1000}
TORRUST_INDEX_RUN_AS_USER=${TORRUST_INDEX_RUN_AS_USER:-appuser}

echo "Building docker image ..."
echo "USER_ID: $USER_ID"
echo "TORRUST_INDEX_RUN_AS_USER: $TORRUST_INDEX_RUN_AS_USER"

docker build \
    --build-arg UID="$USER_ID" \
    --build-arg RUN_AS_USER="$TORRUST_INDEX_RUN_AS_USER" \
    -t torrust-index .
