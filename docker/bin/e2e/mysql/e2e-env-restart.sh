#!/bin/bash

docker compose down
./docker/bin/e2e/mysql/e2e-env-up.sh
