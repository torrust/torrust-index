#!/bin/bash

CURRENT_USER_NAME=$(whoami)
CURRENT_USER_ID=$(id -u)
echo "User name: $CURRENT_USER_NAME"
echo "User   id: $CURRENT_USER_ID"

TORRUST_IDX_BACK_USER_UID=$CURRENT_USER_ID
TORRUST_TRACKER_USER_UID=$CURRENT_USER_ID
export TORRUST_IDX_BACK_USER_UID
export TORRUST_TRACKER_USER_UID

# todo: remove duplicate funtion
wait_for_container_to_be_healthy() {
    local container_name="$1"
    local max_retries="$2"
    local retry_interval="$3"
    local retry_count=0

    while [ $retry_count -lt "$max_retries" ]; do
        container_health="$(docker inspect --format='{{json .State.Health}}' "$container_name")"
        if [ "$container_health" != "{}" ]; then
            container_status="$(echo "$container_health" | jq -r '.Status')"
            if [ "$container_status" == "healthy" ]; then
                echo "Container $container_name is healthy"
                return 0
            fi
        fi

        retry_count=$((retry_count + 1))
        echo "Waiting for container $container_name to become healthy (attempt $retry_count of $max_retries)..."
        sleep "$retry_interval"
    done

    echo "Timeout reached, container $container_name is not healthy"
    return 1
}

# Install tool to create torrent files.
# It's needed by some tests to generate and parse test torrent files.
cargo install imdl || exit 1

# Install app (no docker) that will run the test suite against the E2E testing 
# environment (in docker).
cp .env.local .env || exit 1

# TEST USING MYSQL
echo "Running E2E tests using MySQL ..."

# Start E2E testing environment
./contrib/dev-tools/container/e2e/mysql/e2e-env-up.sh || exit 1

wait_for_container_to_be_healthy torrust-mysql-1 10 3
# todo: implement healthchecks for tracker and index and wait until they are healthy
#wait_for_container torrust-tracker-1 10 3
#wait_for_container torrust-idx-back-1 10 3
sleep 20s

# Just to make sure that everything is up and running
docker ps

# Database credentials
MYSQL_USER="root"
MYSQL_PASSWORD="root_secret_password"
MYSQL_HOST="localhost"
MYSQL_DATABASE="torrust_index_e2e_testing"

# Create the MySQL database for the index. Assumes MySQL client is installed.
echo "Creating MySQL database $MYSQL_DATABASE for for E2E testing ..."
mysql -h $MYSQL_HOST -u $MYSQL_USER -p$MYSQL_PASSWORD -e "CREATE DATABASE IF NOT EXISTS $MYSQL_DATABASE;"

# Run E2E tests with shared app instance
TORRUST_INDEX_E2E_SHARED=true TORRUST_INDEX_E2E_PATH_CONFIG="./share/default/config/index.container.mysql.toml" cargo test || exit 1

# Stop E2E testing environment
./contrib/dev-tools/container/e2e/mysql/e2e-env-down.sh || exit 1

