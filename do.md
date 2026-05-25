Then, in order (one sit each)
Order	Sit	Do only this	Stop when
1
Math
New folder or file math with get_amount_out + one Rust test: 1000/1000, in 100, out 91
cargo test green
2
Shell
anchor init zeroxamm-one (copy Anchor/rust toolchain from zeroxescrow-o1)
anchor build works
3
Init
Only initialize_pool: PDAs + empty vaults, no swap
deploy or anchor test sees pool account
4
Fund
You (or a script) deposit 1000 A + 1000 B into vaults, update reserves in add_liquidity OR manual for v0
vault balances match reserves
5
Swap
Only swap: transfer in, get_amount_out, transfer out, update reserves
one swap on devnet/local
6
Safety
min_amount_out + require!(amount_out >= min)
slippage test passes
Do not start sit 2 until sit 1 is green.
Do not add fees, LP tokens, or remove_liquidity until sit 5 works.

What your brain should not do yet
Read the full V-AMM doc
Newton-Raphson / StableSwap
fee_bps, EWMA, cranks
Perfect LP accounting
Compare to Uniswap/Curve
That’s capstone energy. Not week-one energy.

Tie to what you already built
You already did harder Solana shape stuff in:

zeroxpda-calculator → PDAs
zeroxescrow-o1 → Anchor init, accounts
zeroxamm-one adds one new idea: the formula in sit 1. Sit 2–5 is “escrow with two vaults + math.”

If you freeze, use this sentence
“Today I only make get_amount_out return 91 for 100 in, 1000/1000 reserves.”

Nothing else counts as progress today.

After first real swap (later, not now)
v1: 30 bps fee on input
v2: remove_liquidity pro-rata
v3: read V-AMM doc again
First action right now: paper swap (Step 0).
Second action (next sit): Rust test for get_amount_out only.

When you want the repo scaffolded for sit 2, switch to Agent mode and say: “scaffold zeroxamm-one sit 1 math only.”