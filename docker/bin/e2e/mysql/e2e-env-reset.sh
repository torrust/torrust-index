#!/bin/bash

# Delete the databases and recreate them.

docker compose down

# Index Backend

# Database credentials
MYSQL_USER="root"
MYSQL_PASSWORD="root_secret_password"
MYSQL_HOST="localhost"
MYSQL_DATABASE="torrust_index_backend_e2e_testing"

# Create the MySQL database for the index backend. Assumes MySQL client is installed.
echo "Creating MySQL database $MYSQL_DATABASE for E2E testing ..."
mysql -h $MYSQL_HOST -u $MYSQL_USER -p$MYSQL_PASSWORD -e "DROP DATABASE IF EXISTS $MYSQL_DATABASE; CREATE DATABASE $MYSQL_DATABASE;"

# Tracker

# Delete tracker database
rm -f ./storage/database/torrust_tracker_e2e_testing.db

# Generate lib/torrust directory if it does not exist
mkdir -p "./storage/database"

# Generate the sqlite database for the tracker if it does not exist
if ! [ -f "./storage/database/torrust_tracker_e2e_testing.db" ]; then
    touch ./storage/database/torrust_tracker_e2e_testing.db
    echo ";" | sqlite3 ./storage/database/torrust_tracker_e2e_testing.db
fi

./docker/bin/e2e/mysql/e2e-env-up.sh
