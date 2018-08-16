#!/usr/bin/env bash

set -o allexport
source .env
set +o allexport
# Run this script to update the database to the latest migration version
diesel -V 2> /dev/null || cargo install diesel_cli --no-default-features --features postgres
diesel database reset --database-url=$TEST_DATABASE_ADMIN_URL
diesel setup --database-url=$TEST_DATABASE_ADMIN_URL
diesel migration run --database-url=$TEST_DATABASE_ADMIN_URL
