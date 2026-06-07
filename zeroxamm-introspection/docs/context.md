# Agent context — zeroxamm-introspection (program source only)

Use this file as full source context for another agent. **Tests are excluded.** For design and state transitions see [ARCHITECTURE.md](./ARCHITECTURE.md).

## Summary

- **Stack:** Anchor 0.31.1, SPL Token
- **Program ID:** `7xKp2mNqR8vYw3tZfHjL9sDc4eUb6aFg1nXi5oPr7QwE`
- **Purpose:** Constant-product AMM; swap split into `burn_for_swap` (input) + `swap_payout` (output with instruction introspection on prior ix)
- **Instructions:** `initialize`, `initialize_pool`, `add_liquidity`, `burn_for_swap`, `swap_payout`

## File tree (non-test)

```text
zeroxamm-introspection/
├── Anchor.toml
├── Cargo.toml
└── programs/zeroxamm-introspection/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── error.rs
        ├── state.rs
        ├── instructions.rs
        ├── utils.rs
        ├── utils/introspection.rs
        └── instructions/
            ├── initialize.rs
            ├── initialize_pool.rs
            ├── add_liquidity.rs
            ├── burn_for_swap.rs
            └── swap_payout.rs
```

---

## Anchor.toml

```toml
[toolchain]
anchor_version = "0.31.1"
package_manager = "yarn"

[features]
resolution = true
skip-lint = false

[programs.localnet]
zeroxamm_introspection = "7xKp2mNqR8vYw3tZfHjL9sDc4eUb6aFg1nXi5oPr7QwE"

[provider]
cluster = "localnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "bash scripts/test.sh"

[hooks]
```

---

## Cargo.toml (workspace root)

```toml
[workspace]
members = [
    "programs/*"
]
resolver = "2"

[profile.release]
overflow-checks = true
lto = "fat"
codegen-units = 1
[profile.release.build-override]
opt-level = 3
incremental = false
codegen-units = 1
```

---

## programs/zeroxamm-introspection/Cargo.toml

```toml
[package]
name = "zeroxamm-introspection"
version = "0.1.0"
description = "AMM with instruction introspection for swap payout"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]
name = "zeroxamm_introspection"

[features]
default = []
cpi = ["no-entrypoint"]
no-entrypoint = []
no-idl = []
no-log-ix-name = []
idl-build = ["anchor-lang/idl-build", "anchor-spl/idl-build"]

[dependencies]
anchor-lang = "0.31.1"
anchor-spl = { version = "0.31.1", features = ["idl-build"] }

[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = ['cfg(target_os, values("solana"))'] }
```

---

## programs/zeroxamm-introspection/src/lib.rs

```rust
use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

pub use instructions::*;
pub use state::*;

declare_id!("7xKp2mNqR8vYw3tZfHjL9sDc4eUb6aFg1nXi5oPr7QwE");

#[program]
pub mod zeroxamm_introspection {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn initialize_pool(ctx: Context<InitializePool>, pool_id: u16) -> Result<()> {
        initialize_pool::handler(ctx, pool_id)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<()> {
        add_liquidity::handler(ctx, amount_a, amount_b)
    }

    pub fn burn_for_swap(
        ctx: Context<BurnForSwap>,
        amount_in: u64,
        is_a_to_b: bool,
    ) -> Result<()> {
        burn_for_swap::handler(ctx, amount_in, is_a_to_b)
    }

    pub fn swap_payout(
        ctx: Context<SwapPayout>,
        amount_in: u64,
        min_amount_out: u64,
        is_a_to_b: bool,
    ) -> Result<()> {
        swap_payout::handler(ctx, amount_in, min_amount_out, is_a_to_b)
    }
}
```

---

## programs/zeroxamm-introspection/src/error.rs

```rust
use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Amount out is zero")]
    ZeroAmountOut,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("No prior instruction in transaction")]
    MissingPriorInstruction,
    #[msg("Prior instruction is not from this program")]
    InvalidPriorProgram,
    #[msg("Prior instruction is not burn_for_swap")]
    InvalidPriorDiscriminator,
    #[msg("Prior instruction data does not match")]
    PriorInstructionDataMismatch,
    #[msg("Prior instruction accounts do not match")]
    PriorInstructionAccountsMismatch,
}
```

---

## programs/zeroxamm-introspection/src/state.rs

```rust
use anchor_lang::prelude::*;

#[account]
pub struct PoolState {
    pub bump: u8,
    pub authority_bump: u8,
    pub pool_id: u16,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub token_vault_a: Pubkey,
    pub token_vault_b: Pubkey,
    pub pool_authority: Pubkey,
    pub reserve_a: u64,
    pub reserve_b: u64,
}

impl PoolState {
    pub const LEN: usize = 8 + 1 + 1 + 2 + 32 + 32 + 32 + 32 + 32 + 8 + 8;
}
```

---

## programs/zeroxamm-introspection/src/instructions.rs

```rust
pub mod add_liquidity;
pub mod burn_for_swap;
pub mod initialize;
pub mod initialize_pool;
pub mod swap_payout;

pub use add_liquidity::*;
pub use burn_for_swap::*;
pub use initialize::*;
pub use initialize_pool::*;
pub use swap_payout::*;
```

---

## programs/zeroxamm-introspection/src/utils.rs

```rust
pub mod introspection;
```

---

## programs/zeroxamm-introspection/src/utils/introspection.rs

```rust
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions::{
    load_current_index_checked, load_instruction_at_checked, ID as INSTRUCTIONS_SYSVAR_ID,
};
use anchor_lang::Discriminator;

use crate::error::ErrorCode;
use crate::instruction;

/// Account indices in `burn_for_swap` (must match `BurnForSwap` account order).
const BURN_IX_USER: usize = 0;
const BURN_IX_POOL: usize = 1;
const BURN_IX_USER_A: usize = 2;
const BURN_IX_USER_B: usize = 3;
const BURN_IX_VAULT_A: usize = 4;
const BURN_IX_VAULT_B: usize = 5;
const BURN_IX_TOKEN_PROGRAM: usize = 6;

pub fn verify_prior_burn_for_swap(
    instruction_sysvar: &AccountInfo<'_>,
    program_id: &Pubkey,
    expected_user: &Pubkey,
    expected_pool: &Pubkey,
    expected_user_a: &Pubkey,
    expected_user_b: &Pubkey,
    expected_vault_a: &Pubkey,
    expected_vault_b: &Pubkey,
    expected_token_program: &Pubkey,
    amount_in: u64,
    is_a_to_b: bool,
) -> Result<()> {
    require_keys_eq!(
        *instruction_sysvar.key,
        INSTRUCTIONS_SYSVAR_ID,
        ErrorCode::MissingPriorInstruction
    );

    let current_index = load_current_index_checked(instruction_sysvar)
        .map_err(|_| error!(ErrorCode::MissingPriorInstruction))?;

    require_gt!(current_index, 0, ErrorCode::MissingPriorInstruction);

    let prior_ix = load_instruction_at_checked(
        (current_index - 1) as usize,
        instruction_sysvar,
    )
    .map_err(|_| error!(ErrorCode::MissingPriorInstruction))?;

    require_keys_eq!(prior_ix.program_id, *program_id, ErrorCode::InvalidPriorProgram);

    let data = prior_ix.data.as_slice();
    require!(
        data.len() >= 8 + 8 + 1,
        ErrorCode::InvalidPriorDiscriminator
    );

    let disc = &data[..8];
    require!(
        disc == instruction::BurnForSwap::DISCRIMINATOR,
        ErrorCode::InvalidPriorDiscriminator
    );

    let parsed_amount = u64::from_le_bytes(data[8..16].try_into().unwrap());
    let parsed_is_a_to_b = data[16] != 0;

    require!(
        parsed_amount == amount_in && parsed_is_a_to_b == is_a_to_b,
        ErrorCode::PriorInstructionDataMismatch
    );

    let accounts = &prior_ix.accounts;
    require!(
        accounts.len() > BURN_IX_TOKEN_PROGRAM,
        ErrorCode::PriorInstructionAccountsMismatch
    );

    require_keys_eq!(
        accounts[BURN_IX_USER].pubkey,
        *expected_user,
        ErrorCode::PriorInstructionAccountsMismatch
    );
    require_keys_eq!(
        accounts[BURN_IX_POOL].pubkey,
        *expected_pool,
        ErrorCode::PriorInstructionAccountsMismatch
    );
    require_keys_eq!(
        accounts[BURN_IX_USER_A].pubkey,
        *expected_user_a,
        ErrorCode::PriorInstructionAccountsMismatch
    );
    require_keys_eq!(
        accounts[BURN_IX_USER_B].pubkey,
        *expected_user_b,
        ErrorCode::PriorInstructionAccountsMismatch
    );
    require_keys_eq!(
        accounts[BURN_IX_VAULT_A].pubkey,
        *expected_vault_a,
        ErrorCode::PriorInstructionAccountsMismatch
    );
    require_keys_eq!(
        accounts[BURN_IX_VAULT_B].pubkey,
        *expected_vault_b,
        ErrorCode::PriorInstructionAccountsMismatch
    );
    require_keys_eq!(
        accounts[BURN_IX_TOKEN_PROGRAM].pubkey,
        *expected_token_program,
        ErrorCode::PriorInstructionAccountsMismatch
    );

    Ok(())
}
```

---

## programs/zeroxamm-introspection/src/instructions/initialize.rs

```rust
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize {}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    msg!("Greetings from: {:?}", ctx.program_id);
    Ok(())
}
```

---

## programs/zeroxamm-introspection/src/instructions/initialize_pool.rs

```rust
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::PoolState;

#[derive(Accounts)]
#[instruction(pool_id: u16)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_mint_a: Account<'info, Mint>,
    pub token_mint_b: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        space = PoolState::LEN,
        seeds = [
            b"pool",
            token_mint_a.key().as_ref(),
            token_mint_b.key().as_ref(),
            &pool_id.to_le_bytes(),
        ],
        bump
    )]
    pub pool_state: Account<'info, PoolState>,

    #[account(
        seeds = [b"authority", pool_state.key().as_ref()],
        bump
    )]
    /// CHECK: PDA pool authority; no data, used only as signer for vault CPIs.
    pub pool_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        token::mint = token_mint_a,
        token::authority = pool_authority,
        seeds = [b"vault_a", pool_state.key().as_ref()],
        bump
    )]
    pub token_vault_a: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        token::mint = token_mint_b,
        token::authority = pool_authority,
        seeds = [b"vault_b", pool_state.key().as_ref()],
        bump
    )]
    pub token_vault_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializePool>, pool_id: u16) -> Result<()> {
    let pool = &mut ctx.accounts.pool_state;

    pool.bump = ctx.bumps.pool_state;
    pool.authority_bump = ctx.bumps.pool_authority;
    pool.pool_id = pool_id;
    pool.token_mint_a = ctx.accounts.token_mint_a.key();
    pool.token_mint_b = ctx.accounts.token_mint_b.key();
    pool.token_vault_a = ctx.accounts.token_vault_a.key();
    pool.token_vault_b = ctx.accounts.token_vault_b.key();
    pool.pool_authority = ctx.accounts.pool_authority.key();
    pool.reserve_a = 0;
    pool.reserve_b = 0;

    msg!("Pool initialized: id={}, reserves=0/0", pool_id);
    Ok(())
}
```

---

## programs/zeroxamm-introspection/src/instructions/add_liquidity.rs

```rust
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::PoolState;

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub pool_state: Account<'info, PoolState>,

    #[account(
        mut,
        constraint = user_token_a.mint == pool_state.token_mint_a,
        constraint = user_token_a.owner == payer.key(),
    )]
    pub user_token_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_token_b.mint == pool_state.token_mint_b,
        constraint = user_token_b.owner == payer.key(),
    )]
    pub user_token_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = token_vault_a.key() == pool_state.token_vault_a,
    )]
    pub token_vault_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = token_vault_b.key() == pool_state.token_vault_b,
    )]
    pub token_vault_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<AddLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
    require_gt!(amount_a, 0);
    require_gt!(amount_b, 0);

    let cpi_ctx_a = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token_a.to_account_info(),
            to: ctx.accounts.token_vault_a.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
        },
    );
    token::transfer(cpi_ctx_a, amount_a)?;

    let cpi_ctx_b = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token_b.to_account_info(),
            to: ctx.accounts.token_vault_b.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
        },
    );
    token::transfer(cpi_ctx_b, amount_b)?;

    let pool = &mut ctx.accounts.pool_state;
    pool.reserve_a = pool.reserve_a.checked_add(amount_a).unwrap();
    pool.reserve_b = pool.reserve_b.checked_add(amount_b).unwrap();

    msg!("Liquidity added: {} A, {} B", amount_a, amount_b);
    Ok(())
}
```

---

## programs/zeroxamm-introspection/src/instructions/burn_for_swap.rs

```rust
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::PoolState;

#[derive(Accounts)]
pub struct BurnForSwap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool_state: Account<'info, PoolState>,

    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<BurnForSwap>, amount_in: u64, is_a_to_b: bool) -> Result<()> {
    require_gt!(amount_in, 0);

    let pool = &ctx.accounts.pool_state;

    require_keys_eq!(ctx.accounts.vault_a.key(), pool.token_vault_a);
    require_keys_eq!(ctx.accounts.vault_b.key(), pool.token_vault_b);

    if is_a_to_b {
        require_keys_eq!(ctx.accounts.user_token_a.mint, pool.token_mint_a);
        require_keys_eq!(ctx.accounts.user_token_b.mint, pool.token_mint_b);
    } else {
        require_keys_eq!(ctx.accounts.user_token_a.mint, pool.token_mint_b);
        require_keys_eq!(ctx.accounts.user_token_b.mint, pool.token_mint_a);
    }

    let (user_token_in, vault_in) = if is_a_to_b {
        (&ctx.accounts.user_token_a, &ctx.accounts.vault_a)
    } else {
        (&ctx.accounts.user_token_b, &ctx.accounts.vault_b)
    };

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: user_token_in.to_account_info(),
            to: vault_in.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::transfer(cpi_ctx, amount_in)?;

    msg!("Burn for swap: {} in (a_to_b={})", amount_in, is_a_to_b);
    Ok(())
}
```

---

## programs/zeroxamm-introspection/src/instructions/swap_payout.rs

```rust
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::error::ErrorCode;
use crate::state::PoolState;
use crate::utils::introspection::verify_prior_burn_for_swap;

pub fn get_amount_out(amount_in: u64, reserve_in: u64, reserve_out: u64) -> Option<u64> {
    if amount_in == 0 || reserve_in == 0 || reserve_out == 0 {
        return None;
    }
    let amount_in = amount_in as u128;
    let reserve_in = reserve_in as u128;
    let reserve_out = reserve_out as u128;
    let numerator = amount_in.checked_mul(reserve_out)?;
    let denominator = reserve_in.checked_add(amount_in)?;
    let out = numerator.checked_div(denominator)?;
    Some(out as u64)
}

#[derive(Accounts)]
pub struct SwapPayout<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool_state: Account<'info, PoolState>,

    /// CHECK: PDA pool authority; no data, used only as signer for vault CPIs.
    #[account(
        seeds = [b"authority", pool_state.key().as_ref()],
        bump = pool_state.authority_bump,
    )]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Instructions sysvar for introspection of the prior burn_for_swap ix.
    /// CHECK: Instructions sysvar; validated in introspection helper.
    #[account(address = sysvar::instructions::ID @ ErrorCode::MissingPriorInstruction)]
    pub instruction_sysvar: UncheckedAccount<'info>,
}

pub fn handler(
    ctx: Context<SwapPayout>,
    amount_in: u64,
    min_amount_out: u64,
    is_a_to_b: bool,
) -> Result<()> {
    require_gt!(amount_in, 0);

    verify_prior_burn_for_swap(
        &ctx.accounts.instruction_sysvar.to_account_info(),
        ctx.program_id,
        &ctx.accounts.user.key(),
        &ctx.accounts.pool_state.key(),
        &ctx.accounts.user_token_a.key(),
        &ctx.accounts.user_token_b.key(),
        &ctx.accounts.vault_a.key(),
        &ctx.accounts.vault_b.key(),
        &ctx.accounts.token_program.key(),
        amount_in,
        is_a_to_b,
    )?;

    let pool = &ctx.accounts.pool_state;

    require_keys_eq!(ctx.accounts.vault_a.key(), pool.token_vault_a);
    require_keys_eq!(ctx.accounts.vault_b.key(), pool.token_vault_b);
    require_keys_eq!(
        ctx.accounts.pool_authority.key(),
        pool.pool_authority
    );

    if is_a_to_b {
        require_keys_eq!(ctx.accounts.user_token_a.mint, pool.token_mint_a);
        require_keys_eq!(ctx.accounts.user_token_b.mint, pool.token_mint_b);
    } else {
        require_keys_eq!(ctx.accounts.user_token_a.mint, pool.token_mint_b);
        require_keys_eq!(ctx.accounts.user_token_b.mint, pool.token_mint_a);
    }

    let reserve_in = if is_a_to_b {
        pool.reserve_a
    } else {
        pool.reserve_b
    };
    let reserve_out = if is_a_to_b {
        pool.reserve_b
    } else {
        pool.reserve_a
    };

    let amount_out =
        get_amount_out(amount_in, reserve_in, reserve_out).ok_or(ErrorCode::ZeroAmountOut)?;

    require_gte!(amount_out, min_amount_out, ErrorCode::SlippageExceeded);

    let (user_token_out, vault_out) = if is_a_to_b {
        (&ctx.accounts.user_token_b, &ctx.accounts.vault_b)
    } else {
        (&ctx.accounts.user_token_a, &ctx.accounts.vault_a)
    };

    let seeds = &[
        b"authority",
        ctx.accounts.pool_state.to_account_info().key.as_ref(),
        &[pool.authority_bump],
    ];
    let signer = &[&seeds[..]];

    let cpi_ctx_out = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: vault_out.to_account_info(),
            to: user_token_out.to_account_info(),
            authority: ctx.accounts.pool_authority.to_account_info(),
        },
        signer,
    );
    token::transfer(cpi_ctx_out, amount_out)?;

    let pool = &mut ctx.accounts.pool_state;
    if is_a_to_b {
        pool.reserve_a = pool.reserve_a.checked_add(amount_in).unwrap();
        pool.reserve_b = pool.reserve_b.checked_sub(amount_out).unwrap();
    } else {
        pool.reserve_b = pool.reserve_b.checked_add(amount_in).unwrap();
        pool.reserve_a = pool.reserve_a.checked_sub(amount_out).unwrap();
    }

    msg!("Swap payout: {} in -> {} out", amount_in, amount_out);
    Ok(())
}
```

---

## Quick reference for agents

| PDA seeds | |
|-----------|--|
| Pool | `["pool", mint_a, mint_b, pool_id_le]` |
| Authority | `["authority", pool_state]` |
| Vault A | `["vault_a", pool_state]` |
| Vault B | `["vault_b", pool_state]` |

| Swap tx order | |
|---------------|--|
| 1 | `burn_for_swap(amount_in, is_a_to_b)` |
| 2 | `swap_payout(amount_in, min_amount_out, is_a_to_b)` + Instructions sysvar |

**Excluded from this file:** `tests/test_amm.rs`, `[dev-dependencies]` in program `Cargo.toml`, generated `target/idl`.
