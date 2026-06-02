#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

export CARGO_TARGET_DIR="$(pwd)/target"
FIXTURES="tests/fixtures"

if [[ ! -f "$FIXTURES/mpl_core.so" ]]; then
  echo "Missing $FIXTURES/mpl_core.so — run:"
  echo "  solana program dump CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d $FIXTURES/mpl_core.so -u mainnet-beta"
  exit 1
fi

if [[ ! -f "$FIXTURES/spl_token.so" ]]; then
  echo "Fetching SPL Token program into $FIXTURES/spl_token.so ..."
  solana program dump TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA "$FIXTURES/spl_token.so" -u mainnet-beta
fi

anchor keys sync
anchor build --ignore-keys --no-idl

echo "Running LiteSVM integration tests..."
cargo test -p zeroxnft-staking --manifest-path programs/zeroxnft-staking/Cargo.toml -- --nocapture
