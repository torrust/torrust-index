#!/bin/bash

TORRUST_INDEX_USER_UID=${TORRUST_INDEX_USER_UID:-1000}
TORRUST_IDX_BACK_RUN_AS_USER=${TORRUST_IDX_BACK_RUN_AS_USER:-appuser}

echo "Building docker image ..."
echo "TORRUST_INDEX_USER_UID: $TORRUST_INDEX_USER_UID"
echo "TORRUST_IDX_BACK_RUN_AS_USER: $TORRUST_IDX_BACK_RUN_AS_USER"

docker build \
    --build-arg UID="$TORRUST_INDEX_USER_UID" \
    --build-arg RUN_AS_USER="$TORRUST_IDX_BACK_RUN_AS_USER" \
    --target debug --tag torrust-index-backend:debug .
