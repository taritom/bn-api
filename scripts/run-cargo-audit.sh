#!/usr/bin/env bash
cd ..
cargo install cargo-audit --force && cargo audit
