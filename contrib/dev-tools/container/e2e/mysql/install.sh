#!/bin/bash

# This script is only intended to be used for E2E testing environment.

# Database credentials
MYSQL_USER="root"
MYSQL_PASSWORD="root_secret_password"
MYSQL_HOST="127.0.0.1"
MYSQL_DATABASE="torrust_index_e2e_testing"

# Create the MySQL database for the index. Assumes MySQL client is installed.
# The docker compose configuration already creates the database the first time
# the container is created.
echo "Creating MySQL database $MYSQL_DATABASE for for E2E testing ..."
MYSQL_PWD=$MYSQL_PASSWORD mysql -h $MYSQL_HOST -u $MYSQL_USER -e "CREATE DATABASE IF NOT EXISTS $MYSQL_DATABASE;"
