#!/usr/bin/env bash
cargo run create -c $DATABASE_URL -f -e superuser@test.com -p password -m 8883
cd ../api
cargo build
cargo run &
export SERVER_PID=$!$1
# Run newman tests
apt-get install nodejs
apt-get install npm
    # Workaround for invalid NPM certificate
npm config set strict-ssl false
npm install -g newman

newman run ../integration-tests/bigneon-tests.postman_collection.json -e ../integration-tests/travis.postman_environment.json
kill -s SIGTERM $SERVER_PID