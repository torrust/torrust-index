#!/bin/bash

docker run \
    --detach \
    --name torrust-index-backend-mysql \
    --env MYSQL_USER=db-user \
    --env MYSQL_PASSWORD=db-passwrod \
    --env MYSQL_ROOT_PASSWORD=db-root-password \
    -p 3306:3306 \
    mysql:8.0.30 # This version is used in tests
