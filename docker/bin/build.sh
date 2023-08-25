#!/bin/bash

TORRUST_IDX_BACK_USER_UID=${TORRUST_IDX_BACK_USER_UID:-1000}
TORRUST_IDX_BACK_RUN_AS_USER=${TORRUST_IDX_BACK_RUN_AS_USER:-appuser}

echo "Building docker image ..."
echo "TORRUST_IDX_BACK_USER_UID: $TORRUST_IDX_BACK_USER_UID"
echo "TORRUST_IDX_BACK_RUN_AS_USER: $TORRUST_IDX_BACK_RUN_AS_USER"

docker build \
    --build-arg UID="$TORRUST_IDX_BACK_USER_UID" \
    --build-arg RUN_AS_USER="$TORRUST_IDX_BACK_RUN_AS_USER" \
    --tag torrust-index-backend:local .
