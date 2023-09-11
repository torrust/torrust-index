#!/bin/bash

# Generate the default settings file if it does not exist
if ! [ -f "./config.toml" ]; then
    cp ./config.local.toml ./config.toml
fi

# Generate storage directory if it does not exist
mkdir -p "./storage/database"

# Generate the sqlite database for the index backend if it does not exist
if ! [ -f "./storage/database/data.db" ]; then
    sqlite3 ./storage/database/data.db "VACUUM;"
fi

# Generate storage directory if it does not exist
mkdir -p "./storage/tracker/lib/database"

# Generate the sqlite database for the tracker if it does not exist
if ! [ -f "./storage/tracker/lib/database/sqlite3.db" ]; then
    sqlite3 ./storage/tracker/lib/database/sqlite3.db "VACUUM;"
fi
