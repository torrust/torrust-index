#!/bin/bash

# This script is only intended to be used for E2E testing environment.

## Index

# Database credentials
MYSQL_USER="root"
MYSQL_PASSWORD="root_secret_password"
MYSQL_HOST="127.0.0.1"
MYSQL_DATABASE=$TORRUST_INDEX_DATABASE

# Create the MySQL database for the index. Assumes MySQL client is installed.
# The docker compose configuration already creates the database the first time
# the container is created.
echo "Creating MySQL database '$MYSQL_DATABASE' for for E2E testing ..."
MYSQL_PWD=$MYSQL_PASSWORD mysql -h $MYSQL_HOST -u $MYSQL_USER -e "CREATE DATABASE IF NOT EXISTS $MYSQL_DATABASE;"

## Tracker

# Generate the Tracker sqlite database directory and file if it does not exist
mkdir -p ./storage/tracker/lib/database

if ! [ -f "./storage/tracker/lib/database/${TORRUST_TRACKER_DATABASE}.db" ]; then
    echo "Creating tracker database '${TORRUST_TRACKER_DATABASE}.db'"
    sqlite3 "./storage/tracker/lib/database/${TORRUST_TRACKER_DATABASE}.db" "VACUUM;"
fi
