#!/bin/bash
set -xeo pipefail

DB_USER=${MONGO_USER:-"mongo"}
DB_PASSWORD=${MONGO_PASSWORD:-"password"}
DB_PORT=${MONGO_PORT:-27017}

# Launch postgres using Docker
container=`docker run \
  -e MONGO_INITDB_ROOT_USERNAME=${DB_USER} \
  -e MONGO_INITDB_ROOT_PASSWORD=${DB_PASSWORD} \
  -p "${DB_PORT}":27017 \
  -d mongo`

echo "container=${container}"

