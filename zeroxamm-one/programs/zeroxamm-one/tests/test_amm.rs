use {
    anchor_lang::{
        solana_program::{
            instruction::Instruction, program_option::COption, program_pack::Pack,
            pubkey::Pubkey, system_program,
        },
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    anchor_spl::token::{
        spl_token::state::{Account as TokenAccountState, Mint},
        ID as TOKEN_PROGRAM_ID,
    },
    litesvm::LiteSVM,
    solana_account::Account,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    zeroxamm_one::state::PoolState,
};

fn setup_svm() -> LiteSVM {
    let program_id = zeroxamm_one::id();
    let mut svm = LiteSVM::new();
    svm.add_program(
        program_id,
        include_bytes!("../../../target/deploy/zeroxamm_one.so"),
    )
    .unwrap();
    svm.add_program(
        TOKEN_PROGRAM_ID,
        include_bytes!("../../../target/deploy/spl_token.so"),
    )
    .unwrap();
    svm
}

fn send(
    svm: &mut LiteSVM,
    payer: &Keypair,
    signers: &[&Keypair],
    instruction: Instruction,
) -> litesvm::types::TransactionResult {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

fn inject_mint(svm: &mut LiteSVM, mint: &Keypair, mint_authority: &Pubkey) {
    let mut data = vec![0u8; Mint::LEN];
    let mint_state = Mint {
        mint_authority: COption::Some(*mint_authority),
        supply: 1_000_000_000,
        decimals: 6,
        is_initialized: true,
        freeze_authority: COption::None,
    };
    Pack::pack(mint_state, &mut data).unwrap();

    svm.set_account(
        mint.pubkey(),
        Account {
            lamports: 10_000_000,
            data,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
}

fn inject_token_account(svm: &mut LiteSVM, address: &Pubkey, mint: &Pubkey, owner: &Pubkey, amount: u64) {
    let mut data = vec![0u8; TokenAccountState::LEN];
    let account_state = TokenAccountState {
        mint: *mint,
        owner: *owner,
        amount,
        delegate: COption::None,
        state: anchor_spl::token::spl_token::state::AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };
    Pack::pack(account_state, &mut data).unwrap();

    svm.set_account(
        *address,
        Account {
            lamports: 2_039_280,
            data,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
}

fn pool_addresses(
    program_id: &Pubkey,
    mint_a: &Pubkey,
    mint_b: &Pubkey,
    pool_id: u16,
) -> (Pubkey, Pubkey, Pubkey, Pubkey) {
    let (pool_state, _) = Pubkey::find_program_address(
        &[
            b"pool",
            mint_a.as_ref(),
            mint_b.as_ref(),
            &pool_id.to_le_bytes(),
        ],
        program_id,
    );
    let (pool_authority, _) =
        Pubkey::find_program_address(&[b"authority", pool_state.as_ref()], program_id);
    let (vault_a, _) =
        Pubkey::find_program_address(&[b"vault_a", pool_state.as_ref()], program_id);
    let (vault_b, _) =
        Pubkey::find_program_address(&[b"vault_b", pool_state.as_ref()], program_id);
    (pool_state, pool_authority, vault_a, vault_b)
}

#[test]
fn test_initialize_pool() {
    let program_id = zeroxamm_one::id();
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let mint_a = Keypair::new();
    let mint_b = Keypair::new();
    let pool_id = 0u16;

    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    inject_mint(&mut svm, &mint_a, &payer.pubkey());
    inject_mint(&mut svm, &mint_b, &payer.pubkey());

    let (pool_state, pool_authority, vault_a, vault_b) =
        pool_addresses(&program_id, &mint_a.pubkey(), &mint_b.pubkey(), pool_id);

    let ix = Instruction::new_with_bytes(
        program_id,
        &zeroxamm_one::instruction::InitializePool { pool_id }.data(),
        zeroxamm_one::accounts::InitializePool {
            payer: payer.pubkey(),
            token_mint_a: mint_a.pubkey(),
            token_mint_b: mint_b.pubkey(),
            pool_state,
            pool_authority,
            token_vault_a: vault_a,
            token_vault_b: vault_b,
            token_program: TOKEN_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    let res = send(&mut svm, &payer, &[&payer], ix);
    assert!(res.is_ok(), "{res:?}");

    let pool_account = svm.get_account(&pool_state).unwrap();
    assert_eq!(pool_account.owner, program_id);

    let pool = PoolState::try_deserialize(&mut pool_account.data.as_slice()).unwrap();
    assert_eq!(pool.pool_id, pool_id);
    assert_eq!(pool.reserve_a, 0);
    assert_eq!(pool.reserve_b, 0);
    assert_eq!(pool.token_mint_a, mint_a.pubkey());
    assert_eq!(pool.token_mint_b, mint_b.pubkey());
    assert_eq!(pool.token_vault_a, vault_a);
    assert_eq!(pool.token_vault_b, vault_b);
    assert_eq!(pool.pool_authority, pool_authority);

    let vault_a_account = svm.get_account(&vault_a).unwrap();
    let vault_b_account = svm.get_account(&vault_b).unwrap();
    assert_eq!(vault_a_account.owner, TOKEN_PROGRAM_ID);
    assert_eq!(vault_b_account.owner, TOKEN_PROGRAM_ID);
}

#[test]
fn test_add_liquidity() {
    let program_id = zeroxamm_one::id();
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let mint_a = Keypair::new();
    let mint_b = Keypair::new();
    let pool_id = 0u16;

    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    inject_mint(&mut svm, &mint_a, &payer.pubkey());
    inject_mint(&mut svm, &mint_b, &payer.pubkey());

    let (pool_state, pool_authority, vault_a, vault_b) =
        pool_addresses(&program_id, &mint_a.pubkey(), &mint_b.pubkey(), pool_id);

    // Create user token accounts
    let user_token_a = Keypair::new();
    let user_token_b = Keypair::new();
    inject_token_account(&mut svm, &user_token_a.pubkey(), &mint_a.pubkey(), &payer.pubkey(), 10_000);
    inject_token_account(&mut svm, &user_token_b.pubkey(), &mint_b.pubkey(), &payer.pubkey(), 10_000);

    // Initialize pool
    let ix_init = Instruction::new_with_bytes(
        program_id,
        &zeroxamm_one::instruction::InitializePool { pool_id }.data(),
        zeroxamm_one::accounts::InitializePool {
            payer: payer.pubkey(),
            token_mint_a: mint_a.pubkey(),
            token_mint_b: mint_b.pubkey(),
            pool_state,
            pool_authority,
            token_vault_a: vault_a,
            token_vault_b: vault_b,
            token_program: TOKEN_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    send(&mut svm, &payer, &[&payer], ix_init).unwrap();

    // Add liquidity
    let ix_add = Instruction::new_with_bytes(
        program_id,
        &zeroxamm_one::instruction::AddLiquidity { amount_a: 1_000, amount_b: 1_000 }.data(),
        zeroxamm_one::accounts::AddLiquidity {
            payer: payer.pubkey(),
            pool_state,
            user_token_a: user_token_a.pubkey(),
            user_token_b: user_token_b.pubkey(),
            token_vault_a: vault_a,
            token_vault_b: vault_b,
            token_program: TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
    );
    let res = send(&mut svm, &payer, &[&payer], ix_add);
    assert!(res.is_ok(), "{res:?}");

    // Check reserves updated
    let pool_account = svm.get_account(&pool_state).unwrap();
    let pool = PoolState::try_deserialize(&mut pool_account.data.as_slice()).unwrap();
    assert_eq!(pool.reserve_a, 1_000);
    assert_eq!(pool.reserve_b, 1_000);

    // Check vault balances
    let vault_a_data = svm.get_account(&vault_a).unwrap().data;
    let vault_a_state = TokenAccountState::unpack(&vault_a_data).unwrap();
    assert_eq!(vault_a_state.amount, 1_000);

    let vault_b_data = svm.get_account(&vault_b).unwrap().data;
    let vault_b_state = TokenAccountState::unpack(&vault_b_data).unwrap();
    assert_eq!(vault_b_state.amount, 1_000);
}

#[test]
fn test_swap_a_to_b() {
    let program_id = zeroxamm_one::id();
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let mint_a = Keypair::new();
    let mint_b = Keypair::new();
    let pool_id = 0u16;

    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    inject_mint(&mut svm, &mint_a, &payer.pubkey());
    inject_mint(&mut svm, &mint_b, &payer.pubkey());

    let (pool_state, pool_authority, vault_a, vault_b) =
        pool_addresses(&program_id, &mint_a.pubkey(), &mint_b.pubkey(), pool_id);

    let user_token_a = Keypair::new();
    let user_token_b = Keypair::new();
    inject_token_account(&mut svm, &user_token_a.pubkey(), &mint_a.pubkey(), &payer.pubkey(), 10_000);
    inject_token_account(&mut svm, &user_token_b.pubkey(), &mint_b.pubkey(), &payer.pubkey(), 10_000);

    // Initialize pool
    let ix_init = Instruction::new_with_bytes(
        program_id,
        &zeroxamm_one::instruction::InitializePool { pool_id }.data(),
        zeroxamm_one::accounts::InitializePool {
            payer: payer.pubkey(),
            token_mint_a: mint_a.pubkey(),
            token_mint_b: mint_b.pubkey(),
            pool_state,
            pool_authority,
            token_vault_a: vault_a,
            token_vault_b: vault_b,
            token_program: TOKEN_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    send(&mut svm, &payer, &[&payer], ix_init).unwrap();

    // Add liquidity: 1000 A, 1000 B
    let ix_add = Instruction::new_with_bytes(
        program_id,
        &zeroxamm_one::instruction::AddLiquidity { amount_a: 1_000, amount_b: 1_000 }.data(),
        zeroxamm_one::accounts::AddLiquidity {
            payer: payer.pubkey(),
            pool_state,
            user_token_a: user_token_a.pubkey(),
            user_token_b: user_token_b.pubkey(),
            token_vault_a: vault_a,
            token_vault_b: vault_b,
            token_program: TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
    );
    send(&mut svm, &payer, &[&payer], ix_add).unwrap();

    // Refill user token A for swap
    inject_token_account(&mut svm, &user_token_a.pubkey(), &mint_a.pubkey(), &payer.pubkey(), 10_000);

    // Swap 100 A for B (expected ~90 B)
    let ix_swap = Instruction::new_with_bytes(
        program_id,
        &zeroxamm_one::instruction::Swap { amount_in: 100, min_amount_out: 90, is_a_to_b: true }.data(),
        zeroxamm_one::accounts::Swap {
            user: payer.pubkey(),
            pool_state,
            pool_authority,
            user_token_a: user_token_a.pubkey(),
            user_token_b: user_token_b.pubkey(),
            vault_a,
            vault_b,
            token_program: TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
    );
    let res = send(&mut svm, &payer, &[&payer], ix_swap);
    assert!(res.is_ok(), "{res:?}");

    // Check user received B
    let user_b_data = svm.get_account(&user_token_b.pubkey()).unwrap().data;
    let user_b_state = TokenAccountState::unpack(&user_b_data).unwrap();
    assert_eq!(user_b_state.amount, 9_090);

    // Check pool reserves updated
    let pool_account = svm.get_account(&pool_state).unwrap();
    let pool = PoolState::try_deserialize(&mut pool_account.data.as_slice()).unwrap();
    assert_eq!(pool.reserve_a, 1_100);
    assert_eq!(pool.reserve_b, 910);
}

#[test]
fn test_swap_slippage_fails() {
    let program_id = zeroxamm_one::id();
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let mint_a = Keypair::new();
    let mint_b = Keypair::new();
    let pool_id = 0u16;

    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    inject_mint(&mut svm, &mint_a, &payer.pubkey());
    inject_mint(&mut svm, &mint_b, &payer.pubkey());

    let (pool_state, pool_authority, vault_a, vault_b) =
        pool_addresses(&program_id, &mint_a.pubkey(), &mint_b.pubkey(), pool_id);

    let user_token_a = Keypair::new();
    let user_token_b = Keypair::new();
    inject_token_account(&mut svm, &user_token_a.pubkey(), &mint_a.pubkey(), &payer.pubkey(), 10_000);
    inject_token_account(&mut svm, &user_token_b.pubkey(), &mint_b.pubkey(), &payer.pubkey(), 10_000);

    // Initialize pool
    let ix_init = Instruction::new_with_bytes(
        program_id,
        &zeroxamm_one::instruction::InitializePool { pool_id }.data(),
        zeroxamm_one::accounts::InitializePool {
            payer: payer.pubkey(),
            token_mint_a: mint_a.pubkey(),
            token_mint_b: mint_b.pubkey(),
            pool_state,
            pool_authority,
            token_vault_a: vault_a,
            token_vault_b: vault_b,
            token_program: TOKEN_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    send(&mut svm, &payer, &[&payer], ix_init).unwrap();

    // Add liquidity
    let ix_add = Instruction::new_with_bytes(
        program_id,
        &zeroxamm_one::instruction::AddLiquidity { amount_a: 1_000, amount_b: 1_000 }.data(),
        zeroxamm_one::accounts::AddLiquidity {
            payer: payer.pubkey(),
            pool_state,
            user_token_a: user_token_a.pubkey(),
            user_token_b: user_token_b.pubkey(),
            token_vault_a: vault_a,
            token_vault_b: vault_b,
            token_program: TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
    );
    send(&mut svm, &payer, &[&payer], ix_add).unwrap();

    // Refill user token A for swap attempt
    inject_token_account(&mut svm, &user_token_a.pubkey(), &mint_a.pubkey(), &payer.pubkey(), 10_000);

    // Swap with min_amount_out higher than possible (should fail)
    let ix_swap = Instruction::new_with_bytes(
        program_id,
        &zeroxamm_one::instruction::Swap { amount_in: 100, min_amount_out: 95, is_a_to_b: true }.data(),
        zeroxamm_one::accounts::Swap {
            user: payer.pubkey(),
            pool_state,
            pool_authority,
            user_token_a: user_token_a.pubkey(),
            user_token_b: user_token_b.pubkey(),
            vault_a,
            vault_b,
            token_program: TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
    );
    let res = send(&mut svm, &payer, &[&payer], ix_swap);
    assert!(res.is_err(), "Expected slippage failure");
}
