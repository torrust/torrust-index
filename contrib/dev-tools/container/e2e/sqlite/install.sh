#!/bin/bash

# This script is only intended to be used for E2E testing environment.

# Generate storage directory if it does not exist
mkdir -p ./storage/index/lib/database

# Generate the sqlite database if it does not exist
if ! [ -f "./storage/index/lib/database/sqlite3.db" ]; then
    # todo: it should get the path from tracker.toml and only do it when we use sqlite
    sqlite3 ./storage/index/lib/database/sqlite3.db "VACUUM;"
fi

