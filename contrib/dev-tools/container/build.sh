#!/bin/bash

USER_ID=${USER_ID:-1000}

echo "Building docker image ..."
echo "USER_ID: $USER_ID"

docker build \
    --build-arg UID="$USER_ID" \
    -t torrust-index .
