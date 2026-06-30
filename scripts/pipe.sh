#!/usr/bin/env bash
set -euo pipefail

cargo fmt
cargo clippy -- -D warnings
cargo test
cargo build
