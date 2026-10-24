#!/bin/sh
set -e
cargo build --release --target-dir=/tmp/codecrafters-build-git-rust --manifest-path $(dirname $0)/../Cargo.toml
