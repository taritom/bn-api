#!/usr/bin/env bash
# Ensure we are in the root of the git repo
cd $(git rev-parse --show-toplevel)
ulimit -S -s 65536
ulimit -s 65536
ulimit -a
cargo install cargo-audit --force && cargo audit
