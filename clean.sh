#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo clean --manifest-path ./Cargo.toml
