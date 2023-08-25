#!/bin/bash

# Generate the default settings file if it does not exist
if ! [ -f "./config.toml" ]; then
    cp ./config-index.local.toml ./config.toml
fi

# Generate storage directory if it does not exist
mkdir -p "./storage/database"

# Generate the sqlite database for the index backend if it does not exist
if ! [ -f "./storage/database/data.db" ]; then
    # todo: it should get the path from config.toml and only do it when we use sqlite
    touch ./storage/database/data.db
    echo ";" | sqlite3 ./storage/database/data.db
fi

# Generate the sqlite database for the tracker if it does not exist
if ! [ -f "./storage/database/tracker.db" ]; then
    touch ./storage/database/tracker.db
    echo ";" | sqlite3 ./storage/database/tracker.db
fi
