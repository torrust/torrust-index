#!/bin/bash

# This script is only intended to be used for E2E testing environment.

## Index

# Generate the Index sqlite database directory and file if it does not exist
mkdir -p ./storage/index/lib/database

if ! [ -f "./storage/index/lib/database/${TORRUST_INDEX_DATABASE}.db" ]; then
    echo "Creating index database '${TORRUST_INDEX_DATABASE}.db'"
    sqlite3 "./storage/index/lib/database/${TORRUST_INDEX_DATABASE}.db" "VACUUM;"
fi

## Tracker

# Generate the Tracker sqlite database directory and file if it does not exist
mkdir -p ./storage/tracker/lib/database

if ! [ -f "./storage/tracker/lib/database/${TORRUST_TRACKER_DATABASE}.db" ]; then
    echo "Creating tracker database '${TORRUST_TRACKER_DATABASE}.db'"
    sqlite3 "./storage/tracker/lib/database/${TORRUST_TRACKER_DATABASE}.db" "VACUUM;"
fi
