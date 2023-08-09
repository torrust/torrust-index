#!/bin/bash

docker compose down
./docker/bin/e2e/sqlite/e2e-env-up.sh
