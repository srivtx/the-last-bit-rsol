# zeroxnft-staking

Metaplex **Core** NFT staking for Assignment 1 (Week 5). Built on the official [Anchor staking guide](https://developers.metaplex.com/core/guides/anchor/anchor-staking-example), with:

- **`claim_rewards`** — claim SPL rewards while the NFT stays staked (frozen)
- **Collection `Attributes` plugin** — tracks `staked_count` across the collection
- **`initialize`** — staking config PDA + reward vault

## Stack

| Component | Version |
|-----------|---------|
| Anchor CLI | 1.0.x |
| anchor-lang / anchor-spl | 0.31.1 |
| mpl-core (Rust) | 0.11.x (`anchor` feature) |
| @metaplex-foundation/mpl-core (tests) | ^1.0.2 |

Program ID (localnet): `2cna6db1apehVYZiQpBUtBHYNWP6fPWzUaDXvxcNtWiF`

## Instructions

| Instruction | Purpose |
|-------------|---------|
| `initialize` | Create `StakeConfig` + reward token vault for a collection |
| `stake` | Set asset `staked` timestamp, freeze NFT, `staked_count += 1` |
| `claim_rewards` | Pay `elapsed * reward_per_second` SPL; reset stake clock; NFT stays frozen |
| `unstake` | Accrue time, clear `staked`, thaw + remove freeze, `staked_count -= 1` |

## Prerequisites

- Solana CLI, Anchor 1.0+, Node/yarn
- Local validator: `solana-test-validator` (or `anchor test` starts one)

## Build

```bash
cd zeroxnft-staking
anchor build
```

## Test

```bash
yarn install
anchor test
```

For submission, capture a screenshot of the terminal showing all tests passing.

## Docs

See [`../nftstaking_docs/`](../nftstaking_docs/) for architecture and attribute design.

## Clean artifacts

From repo root:

```bash
./scripts/clean.sh
```
