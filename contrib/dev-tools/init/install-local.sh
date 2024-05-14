#!/bin/bash

# This script is only intended to be used for development environment.

# Generate the Index sqlite database directory and file if it does not exist
mkdir -p ./storage/index/lib/database

if ! [ -f "./storage/index/lib/database/sqlite3.db" ]; then
    echo "Creating index database 'sqlite3.db'"
    sqlite3 "./storage/index/lib/database/sqlite3.db" "VACUUM;"
fi
