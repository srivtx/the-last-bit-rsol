#!/usr/bin/env bash
set -euo pipefail

cargo build-sbf --manifest-path programs/zeroxnft-marketplace/Cargo.toml --sbf-out-dir target/deploy
cargo test -p zeroxnft-marketplace --tests

