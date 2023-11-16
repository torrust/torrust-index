#!/bin/bash

CURRENT_USER_NAME=$(whoami)
CURRENT_USER_ID=$(id -u)
echo "User name: $CURRENT_USER_NAME"
echo "User   id: $CURRENT_USER_ID"

USER_ID=$CURRENT_USER_ID
TORRUST_TRACKER_USER_UID=$CURRENT_USER_ID
export USER_ID
export TORRUST_TRACKER_USER_UID

# Install tool to create torrent files.
# It's needed by some tests to generate and parse test torrent files.
cargo install imdl || exit 1

# Install app (no docker) that will run the test suite against the E2E testing 
# environment (in docker).
cp .env.local .env || exit 1
./contrib/dev-tools/container/e2e/sqlite/install.sh || exit 1

# TEST USING SQLITE
echo "Running E2E tests using SQLite ..."

# Start E2E testing environment
./contrib/dev-tools/container/e2e/sqlite/e2e-env-up.sh || exit 1

# Wait for conatiners to be healthy
./contrib/dev-tools/container/functions/wait_for_container_to_be_healthy.sh torrust-mysql-1 10 3 || exit 1
# todo: implement healthchecks for the tracker and wait until it's healthy
#./contrib/dev-tools/container/functions/wait_for_container_to_be_healthy.sh torrust-tracker-1 10 3
./contrib/dev-tools/container/functions/wait_for_container_to_be_healthy.sh  torrust-index-1 10 3 || exit 1
sleep 20s

# Just to make sure that everything is up and running
docker ps

# Run E2E tests with shared app instance
TORRUST_INDEX_E2E_SHARED=true TORRUST_INDEX_E2E_PATH_CONFIG="./share/default/config/index.container.sqlite3.toml" cargo test || exit 1

# Stop E2E testing environment
./contrib/dev-tools/container/e2e/sqlite/e2e-env-down.sh || exit 1
