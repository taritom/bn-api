#!/usr/bin/env bash
ulimit -S -s 65536
ulimit -s 65536
ulimit -a 
cd ../db
cargo run create -c $DATABASE_URL -f -e superuser@test.com -p password -m 8883
cd ..
cargo test --exclude bigneon_api --all
