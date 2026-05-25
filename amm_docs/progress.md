# zeroxamm-one — progress

Capstone path: basic constant-product AMM first, then V-AMM later.

---

## Done

### Sit 0 — Paper math + docs

- [x] Read / derived `amount_out` formula
- [x] `amm_docs/amount_out.md` (plain markdown, no LaTeX)

### Sit 1 — Rust math crate

- [x] `zeroxamm-math/` created (`cargo new --lib`)
- [x] `get_amount_out(amount_in, reserve_in, reserve_out) -> Option<u64>`
- [x] Formula: `(amount_in * reserve_out) / (reserve_in + amount_in)` with `u128` + `checked_*` + `?`
- [x] Tests: `100` → `90`, `10` → `9` (100/100 pool)
- [x] `cargo test` green

**Proof:** `zeroxamm-math/src/lib.rs`

### Sit 2 — Anchor shell

- [x] `zeroxamm-one/` Anchor workspace
- [x] `anchor build` works

### Sit 3 — `initialize_pool`

- [x] `PoolState` + `LEN` in `state.rs`
- [x] `initialize_pool` instruction (pool + authority + vault PDAs)
- [x] `/// CHECK:` on `pool_authority`
- [x] LiteSVM test: `tests/test_initialize_pool.rs`
- [x] `anchor test` green (`test_initialize_pool`)

**Proof:** `programs/zeroxamm-one/src/instructions/initialize_pool.rs`

**Test deps:** `target/deploy/zeroxamm_one.so` + `spl_token.so` (dump once: `solana program dump TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA target/deploy/spl_token.so`)

---

## Current

### Sit 4 — Add liquidity

- [ ] `add_liquidity` instruction
- [ ] Vault balances match `reserve_a` / `reserve_b`

---

## Later (do not skip ahead)

| Sit | Task | Stop when |
|-----|------|-----------|
| 5 | `swap` instruction | one swap works |
| 6 | `min_amount_out` | slippage check passes |

### After zeroxamm-one

- [ ] Swap fee (30 bps on input)
- [ ] `remove_liquidity` pro-rata
- [ ] V-AMM capstone (StableSwap, volatility, cranks)

---

## Concepts locked in

| Term | Meaning |
|------|--------|
| `reserve_in` | Pool balance of token user sells |
| `reserve_out` | Pool balance of token user buys |
| `amount_out` | How much output token user receives |
| `?` | If `None`, bail from function (not `unwrap_or`) |
| `PoolState` | On-chain pool data layout |
| `InitializePool` | Tx account list for init only |
| `pool_authority` | PDA signer for vaults (no data) |
| `payer` | Wallet that pays rent for `init` |

---

## Notes

- Use **90** not 91 for `100` in / `1000` pool (integer division).
- Remove stub `initialize` from `lib.rs` when cleaning up (optional).
