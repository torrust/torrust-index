#!/bin/bash

# Delete the databases and recreate them.

docker compose down

rm -f ./storage/database/torrust_index_backend_e2e_testing.db
rm -f ./storage/database/torrust_tracker_e2e_testing.db

# Generate storage directory if it does not exist
mkdir -p "./storage/database"

# Generate the sqlite database for the index backend if it does not exist
if ! [ -f "./storage/database/torrust_index_backend_e2e_testing.db" ]; then
    # todo: it should get the path from config.toml and only do it when we use sqlite
    touch ./storage/database/torrust_index_backend_e2e_testing.db
    echo ";" | sqlite3 ./storage/database/torrust_index_backend_e2e_testing.db
fi

# Generate the sqlite database for the tracker if it does not exist
if ! [ -f "./storage/database/torrust_tracker_e2e_testing.db" ]; then
    touch ./storage/database/torrust_tracker_e2e_testing.db
    echo ";" | sqlite3 ./storage/database/torrust_tracker_e2e_testing.db
fi

./docker/bin/e2e/sqlite/e2e-env-up.sh
