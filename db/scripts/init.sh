#!/usr/bin/env bash

# Run this script to update the database to the latest migration version
diesel -V 2> /dev/null || cargo install diesel_cli --no-default-features --features postgres
diesel setup
diesel migration run
