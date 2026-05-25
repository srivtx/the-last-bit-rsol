# Where amount_out Comes From

Constant-product AMM (zeroxamm-one). Paper math only — no Solana yet.

---

## 1. What the words mean

| Name | Meaning |
|------|--------|
| x | Reserve of the token you **sell** (token A) — `reserve_in` before the swap |
| y | Reserve of the token you **buy** (token B) — `reserve_out` before the swap |
| dx | How much A the user puts in — `amount_in` |
| dy | How much B the user gets — `amount_out` |

**amount_out** = how much of the **output** token the pool pays the user in this swap.

---

## 2. The one rule: constant product

The pool keeps:

```
x * y = k
```

Before the swap: `k = x * y`.

After the user adds **dx** of A and removes **dy** of B, the pool holds:

- **x + dx** of A
- **y - dy** of B

We want the same invariant (simple model):

```
(x + dx) * (y - dy) = x * y
```

That is not a separate rule — it **is** "x*y = k" written for after the trade.

---

## 3. Solve for dy (algebra)

Step 1 — isolate how much B stays in the pool:

```
y - dy = (x * y) / (x + dx)
```

Step 2 — solve for dy:

```
dy = y - (x * y) / (x + dx)
```

Step 3 — common denominator:

```
dy = (y * (x + dx) - x * y) / (x + dx)
dy = (y * dx) / (x + dx)
```

**The formula:**

```
dy = (dx * y) / (x + dx)
```

In code:

```text
amount_out = (amount_in * reserve_out) / (reserve_in + amount_in)
```

| Math | Code |
|------|------|
| dx | `amount_in` |
| x | `reserve_in` |
| y | `reserve_out` |
| dy | `amount_out` |

---

## 4. Example: 1000 / 1000 pool, 100 A in

```
x  = 1000
y  = 1000
dx = 100

dy = (100 * 1000) / (1000 + 100)
   = 100000 / 1100
   = 90   (integer division — drop the fraction)
```

So **amount_out = 90** B.

After the swap (conceptually):

```
Pool A: 1100
Pool B: 910   (1000 - 90)

1100 * 910 = 1001000
1000 * 1000 = 1000000

1001000 >= 1000000  →  pool keeps a tiny rounding edge
```

---

## 5. Intuition (no algebra)

- You add **A** → the pool has **more A** → A is relatively **cheaper** → you get **less B per A**.
- Bigger **dx** → worse rate for you → **(x + dx)** in the denominator does that.

**One sentence:** `amount_out` is how much B the pool pays you; the formula is "keep x*y the same and solve for how much B left."

---

## 6. Integer math on Solana

Tokens are **whole units** (lamports / smallest decimals).

```text
amount_out = (amount_in * reserve_out) / (reserve_in + amount_in)
```

Division **rounds down** → user gets **less than or equal to** the perfect real number → **dust stays with the pool (LPs)**. That is the "tiny edge," not the 0.3% swap fee (that comes later with `fee_bps`).

Some programs also do: if `amount_out > 0` then `amount_out -= 1` for extra safety.

---

## 7. min_amount_out (later, in the program)

User sets: "I only accept this swap if I get **at least** this much B."

```text
require!(amount_out >= min_amount_out);
```

If the pool would give 90 but you required 95, the transaction **fails** (slippage protection).

---

## 8. Paper practice (Sit 0)

Pool always **1000 A, 1000 B**. Use:

```text
amount_out = (amount_in * 1000) / (1000 + amount_in)
```

Integer division only.

| amount_in | amount_out |
|-----------|------------|
| 100 | 90 |
| 10 | 9 |
| 500 | 333 |

**Done when you can say:** 100 in → **90** out, without looking.

---

## 9. What comes next

1. **Sit 1:** Rust `get_amount_out` + test `100 → 90`
2. **Later:** `fee_bps` on input before this formula
3. **Much later (V-AMM):** StableSwap + volatility — not zeroxamm-one

---

## 10. Alternate form (same math, watch rounding)

Some write:

```text
y_new = (x * y) / (x + dx)
dy    = y - y_new
```

If you floor `y_new` first, you can get **91** instead of **90** for the same inputs. Pick **one** convention in code and test it. zeroxamm-one should use:

```text
amount_out = (amount_in * reserve_out) / (reserve_in + amount_in)
```
