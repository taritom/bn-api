#!/usr/bin/env bash
ulimit -S -s 65536
ulimit -s 65536
ulimit -a 
cd ..
cargo install cargo-audit --force && cargo audit
