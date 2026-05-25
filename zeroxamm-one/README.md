# zeroxamm-one

Simple constant-product AMM on Solana using Anchor.

## What it does

- **initialize_pool** — create a pool with two token vaults (PDAs)
- **add_liquidity** — deposit token A and B into the pool vaults
- **swap** — trade one token for another using `x * y = k` math with slippage protection

## Math

```
amount_out = (amount_in * reserve_out) / (reserve_in + amount_in)
```

Example: 100 A in, pool has 1000 A / 1000 B  
→ amount_out = (100 * 1000) / (1000 + 100) = 90 B

## Build

```bash
anchor build
```

## Test

```bash
cargo test
```

All tests use LiteSVM (no validator needed).

## Tests

- `test_initialize_pool` — pool + vault PDAs created correctly
- `test_add_liquidity` — tokens deposited, reserves updated
- `test_swap_a_to_b` — 100 A → 90 B with 1000/1000 pool
- `test_swap_slippage_fails` — tx reverts when min_amount_out not met

## Program ID

```
BwYSdX5KxrcJzxcBhJ3zSveeJ1Cae9AgN8BDLHvY6E3v
```
