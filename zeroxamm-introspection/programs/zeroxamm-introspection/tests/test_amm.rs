use {
    anchor_lang::{
        solana_program::{
            instruction::Instruction as AnchorInstruction,
            program_option::COption,
            program_pack::Pack,
            pubkey::Pubkey,
            system_program,
            sysvar,
        },
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    anchor_spl::token::{
        spl_token::state::{Account as TokenAccountState, Mint},
        ID as TOKEN_PROGRAM_ID,
    },
    litesvm::LiteSVM,
    solana_account::Account,
    solana_address::Address,
    solana_instruction::{AccountMeta, Instruction as SvmInstruction},
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    zeroxamm_introspection::state::PoolState,
};

fn to_address(p: Pubkey) -> Address {
    Address::from(p.to_bytes())
}

fn pk(k: &Keypair) -> Pubkey {
    Pubkey::new_from_array(k.pubkey().to_bytes())
}

fn to_svm_ix(ix: AnchorInstruction) -> SvmInstruction {
    SvmInstruction {
        program_id: to_address(ix.program_id),
        accounts: ix
            .accounts
            .into_iter()
            .map(|a| AccountMeta {
                pubkey: to_address(a.pubkey),
                is_signer: a.is_signer,
                is_writable: a.is_writable,
            })
            .collect(),
        data: ix.data,
    }
}

fn setup_svm() -> LiteSVM {
    let mut svm = LiteSVM::new();
    svm.add_program(
        to_address(zeroxamm_introspection::id()),
        include_bytes!("../../../target/deploy/zeroxamm_introspection.so"),
    )
    .unwrap();
    svm.add_program(
        to_address(TOKEN_PROGRAM_ID),
        include_bytes!("../../../../zeroxnft-staking/tests/fixtures/spl_token.so"),
    )
    .unwrap();
    svm
}

fn send(
    svm: &mut LiteSVM,
    payer: &Keypair,
    signers: &[&Keypair],
    instruction: AnchorInstruction,
) -> litesvm::types::TransactionResult {
    send_multi(svm, payer, signers, &[instruction])
}

fn send_multi(
    svm: &mut LiteSVM,
    payer: &Keypair,
    signers: &[&Keypair],
    instructions: &[AnchorInstruction],
) -> litesvm::types::TransactionResult {
    let blockhash = svm.latest_blockhash();
    let svm_ixs: Vec<SvmInstruction> = instructions.iter().map(|ix| to_svm_ix(ix.clone())).collect();
    let msg = Message::new_with_blockhash(&svm_ixs, Some(&payer.pubkey()), &blockhash);
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
        to_address(pk(mint)),
        Account {
            lamports: 10_000_000,
            data,
            owner: to_address(TOKEN_PROGRAM_ID),
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
}

fn inject_token_account(
    svm: &mut LiteSVM,
    address: &Pubkey,
    mint: &Pubkey,
    owner: &Pubkey,
    amount: u64,
) {
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
        to_address(*address),
        Account {
            lamports: 2_039_280,
            data,
            owner: to_address(TOKEN_PROGRAM_ID),
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

fn setup_funded_pool(
    svm: &mut LiteSVM,
    program_id: Pubkey,
    payer: &Keypair,
    mint_a: &Keypair,
    mint_b: &Keypair,
    user_token_a: &Keypair,
    user_token_b: &Keypair,
    liquidity_a: u64,
    liquidity_b: u64,
) -> (Pubkey, Pubkey, Pubkey, Pubkey) {
    let pool_id = 0u16;
    let (pool_state, pool_authority, vault_a, vault_b) =
        pool_addresses(&program_id, &pk(mint_a), &pk(mint_b), pool_id);

    let ix_init_pool = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxamm_introspection::instruction::InitializePool { pool_id }.data(),
        zeroxamm_introspection::accounts::InitializePool {
            payer: pk(payer),
            token_mint_a: pk(mint_a),
            token_mint_b: pk(mint_b),
            pool_state,
            pool_authority,
            token_vault_a: vault_a,
            token_vault_b: vault_b,
            token_program: TOKEN_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    send(svm, payer, &[payer], ix_init_pool).unwrap();

    let ix_add = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxamm_introspection::instruction::AddLiquidity {
            amount_a: liquidity_a,
            amount_b: liquidity_b,
        }
        .data(),
        zeroxamm_introspection::accounts::AddLiquidity {
            payer: pk(payer),
            pool_state,
            user_token_a: pk(user_token_a),
            user_token_b: pk(user_token_b),
            token_vault_a: vault_a,
            token_vault_b: vault_b,
            token_program: TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
    );
    send(svm, payer, &[payer], ix_add).unwrap();

    (pool_state, pool_authority, vault_a, vault_b)
}

fn burn_for_swap_ix(
    program_id: Pubkey,
    user: Pubkey,
    pool_state: Pubkey,
    user_token_a: Pubkey,
    user_token_b: Pubkey,
    vault_a: Pubkey,
    vault_b: Pubkey,
    amount_in: u64,
    is_a_to_b: bool,
) -> AnchorInstruction {
    AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxamm_introspection::instruction::BurnForSwap {
            amount_in,
            is_a_to_b,
        }
        .data(),
        zeroxamm_introspection::accounts::BurnForSwap {
            user,
            pool_state,
            user_token_a,
            user_token_b,
            vault_a,
            vault_b,
            token_program: TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}

fn swap_payout_ix(
    program_id: Pubkey,
    user: Pubkey,
    pool_state: Pubkey,
    pool_authority: Pubkey,
    user_token_a: Pubkey,
    user_token_b: Pubkey,
    vault_a: Pubkey,
    vault_b: Pubkey,
    amount_in: u64,
    min_amount_out: u64,
    is_a_to_b: bool,
) -> AnchorInstruction {
    AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxamm_introspection::instruction::SwapPayout {
            amount_in,
            min_amount_out,
            is_a_to_b,
        }
        .data(),
        zeroxamm_introspection::accounts::SwapPayout {
            user,
            pool_state,
            pool_authority,
            user_token_a,
            user_token_b,
            vault_a,
            vault_b,
            token_program: TOKEN_PROGRAM_ID,
            instruction_sysvar: sysvar::instructions::ID,
        }
        .to_account_metas(None),
    )
}

#[test]
fn test_initialize() {
    let program_id = zeroxamm_introspection::id();
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxamm_introspection::instruction::Initialize {}.data(),
        zeroxamm_introspection::accounts::Initialize {}.to_account_metas(None),
    );

    let res = send(&mut svm, &payer, &[&payer], ix);
    assert!(res.is_ok(), "{res:?}");
}

#[test]
fn test_initialize_pool() {
    let program_id = zeroxamm_introspection::id();
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let mint_a = Keypair::new();
    let mint_b = Keypair::new();
    let pool_id = 0u16;

    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    inject_mint(&mut svm, &mint_a, &pk(&payer));
    inject_mint(&mut svm, &mint_b, &pk(&payer));

    let (pool_state, pool_authority, vault_a, vault_b) =
        pool_addresses(&program_id, &pk(&mint_a), &pk(&mint_b), pool_id);

    let ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxamm_introspection::instruction::InitializePool { pool_id }.data(),
        zeroxamm_introspection::accounts::InitializePool {
            payer: pk(&payer),
            token_mint_a: pk(&mint_a),
            token_mint_b: pk(&mint_b),
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

    let pool_account = svm.get_account(&to_address(pool_state)).unwrap();
    assert_eq!(pool_account.owner, to_address(program_id));

    let pool = PoolState::try_deserialize(&mut pool_account.data.as_slice()).unwrap();
    assert_eq!(pool.pool_id, pool_id);
    assert_eq!(pool.reserve_a, 0);
    assert_eq!(pool.reserve_b, 0);
}

#[test]
fn test_add_liquidity() {
    let program_id = zeroxamm_introspection::id();
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let mint_a = Keypair::new();
    let mint_b = Keypair::new();

    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    inject_mint(&mut svm, &mint_a, &pk(&payer));
    inject_mint(&mut svm, &mint_b, &pk(&payer));

    let user_token_a = Keypair::new();
    let user_token_b = Keypair::new();
    inject_token_account(&mut svm, &pk(&user_token_a), &pk(&mint_a), &pk(&payer), 10_000);
    inject_token_account(&mut svm, &pk(&user_token_b), &pk(&mint_b), &pk(&payer), 10_000);

    let (pool_state, _, vault_a, vault_b) = setup_funded_pool(
        &mut svm,
        program_id,
        &payer,
        &mint_a,
        &mint_b,
        &user_token_a,
        &user_token_b,
        1_000,
        1_000,
    );

    let pool = PoolState::try_deserialize(
        &mut svm.get_account(&to_address(pool_state)).unwrap().data.as_slice(),
    )
    .unwrap();
    assert_eq!(pool.reserve_a, 1_000);
    assert_eq!(pool.reserve_b, 1_000);

    let vault_a_state =
        TokenAccountState::unpack(&svm.get_account(&to_address(vault_a)).unwrap().data).unwrap();
    assert_eq!(vault_a_state.amount, 1_000);
    let vault_b_state =
        TokenAccountState::unpack(&svm.get_account(&to_address(vault_b)).unwrap().data).unwrap();
    assert_eq!(vault_b_state.amount, 1_000);
}

#[test]
fn test_burn_for_swap_only() {
    let program_id = zeroxamm_introspection::id();
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let mint_a = Keypair::new();
    let mint_b = Keypair::new();

    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    inject_mint(&mut svm, &mint_a, &pk(&payer));
    inject_mint(&mut svm, &mint_b, &pk(&payer));

    let user_token_a = Keypair::new();
    let user_token_b = Keypair::new();
    inject_token_account(&mut svm, &pk(&user_token_a), &pk(&mint_a), &pk(&payer), 10_000);
    inject_token_account(&mut svm, &pk(&user_token_b), &pk(&mint_b), &pk(&payer), 10_000);

    let (pool_state, _, vault_a, vault_b) = setup_funded_pool(
        &mut svm,
        program_id,
        &payer,
        &mint_a,
        &mint_b,
        &user_token_a,
        &user_token_b,
        1_000,
        1_000,
    );

    inject_token_account(&mut svm, &pk(&user_token_a), &pk(&mint_a), &pk(&payer), 10_000);

    let ix = burn_for_swap_ix(
        program_id,
        pk(&payer),
        pool_state,
        pk(&user_token_a),
        pk(&user_token_b),
        vault_a,
        vault_b,
        100,
        true,
    );
    let res = send(&mut svm, &payer, &[&payer], ix);
    assert!(res.is_ok(), "{res:?}");

    let user_a = TokenAccountState::unpack(
        &svm.get_account(&to_address(pk(&user_token_a)))
            .unwrap()
            .data,
    )
    .unwrap();
    assert_eq!(user_a.amount, 9_900);
    let vault_a_state =
        TokenAccountState::unpack(&svm.get_account(&to_address(vault_a)).unwrap().data).unwrap();
    assert_eq!(vault_a_state.amount, 1_100);
}

#[test]
fn test_swap_payout_happy_path() {
    let program_id = zeroxamm_introspection::id();
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let mint_a = Keypair::new();
    let mint_b = Keypair::new();

    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    inject_mint(&mut svm, &mint_a, &pk(&payer));
    inject_mint(&mut svm, &mint_b, &pk(&payer));

    let user_token_a = Keypair::new();
    let user_token_b = Keypair::new();
    inject_token_account(&mut svm, &pk(&user_token_a), &pk(&mint_a), &pk(&payer), 10_000);
    inject_token_account(&mut svm, &pk(&user_token_b), &pk(&mint_b), &pk(&payer), 10_000);

    let (pool_state, pool_authority, vault_a, vault_b) = setup_funded_pool(
        &mut svm,
        program_id,
        &payer,
        &mint_a,
        &mint_b,
        &user_token_a,
        &user_token_b,
        1_000,
        1_000,
    );

    inject_token_account(&mut svm, &pk(&user_token_a), &pk(&mint_a), &pk(&payer), 10_000);

    let ix_burn = burn_for_swap_ix(
        program_id,
        pk(&payer),
        pool_state,
        pk(&user_token_a),
        pk(&user_token_b),
        vault_a,
        vault_b,
        100,
        true,
    );
    let ix_payout = swap_payout_ix(
        program_id,
        pk(&payer),
        pool_state,
        pool_authority,
        pk(&user_token_a),
        pk(&user_token_b),
        vault_a,
        vault_b,
        100,
        90,
        true,
    );

    let res = send_multi(&mut svm, &payer, &[&payer], &[ix_burn, ix_payout]);
    assert!(res.is_ok(), "{res:?}");

    let user_b = TokenAccountState::unpack(
        &svm.get_account(&to_address(pk(&user_token_b)))
            .unwrap()
            .data,
    )
    .unwrap();
    assert_eq!(user_b.amount, 9_090);

    let pool = PoolState::try_deserialize(
        &mut svm.get_account(&to_address(pool_state)).unwrap().data.as_slice(),
    )
    .unwrap();
    assert_eq!(pool.reserve_a, 1_100);
    assert_eq!(pool.reserve_b, 910);
}

#[test]
fn test_swap_payout_without_prior_burn_fails() {
    let program_id = zeroxamm_introspection::id();
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let mint_a = Keypair::new();
    let mint_b = Keypair::new();

    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    inject_mint(&mut svm, &mint_a, &pk(&payer));
    inject_mint(&mut svm, &mint_b, &pk(&payer));

    let user_token_a = Keypair::new();
    let user_token_b = Keypair::new();
    inject_token_account(&mut svm, &pk(&user_token_a), &pk(&mint_a), &pk(&payer), 10_000);
    inject_token_account(&mut svm, &pk(&user_token_b), &pk(&mint_b), &pk(&payer), 10_000);

    let (pool_state, pool_authority, vault_a, vault_b) = setup_funded_pool(
        &mut svm,
        program_id,
        &payer,
        &mint_a,
        &mint_b,
        &user_token_a,
        &user_token_b,
        1_000,
        1_000,
    );

    let ix_payout = swap_payout_ix(
        program_id,
        pk(&payer),
        pool_state,
        pool_authority,
        pk(&user_token_a),
        pk(&user_token_b),
        vault_a,
        vault_b,
        100,
        90,
        true,
    );

    let res = send(&mut svm, &payer, &[&payer], ix_payout);
    assert!(res.is_err(), "Expected failure without prior burn_for_swap");
}

#[test]
fn test_swap_payout_wrong_prior_amount_fails() {
    let program_id = zeroxamm_introspection::id();
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let mint_a = Keypair::new();
    let mint_b = Keypair::new();

    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    inject_mint(&mut svm, &mint_a, &pk(&payer));
    inject_mint(&mut svm, &mint_b, &pk(&payer));

    let user_token_a = Keypair::new();
    let user_token_b = Keypair::new();
    inject_token_account(&mut svm, &pk(&user_token_a), &pk(&mint_a), &pk(&payer), 10_000);
    inject_token_account(&mut svm, &pk(&user_token_b), &pk(&mint_b), &pk(&payer), 10_000);

    let (pool_state, pool_authority, vault_a, vault_b) = setup_funded_pool(
        &mut svm,
        program_id,
        &payer,
        &mint_a,
        &mint_b,
        &user_token_a,
        &user_token_b,
        1_000,
        1_000,
    );

    inject_token_account(&mut svm, &pk(&user_token_a), &pk(&mint_a), &pk(&payer), 10_000);

    let ix_burn = burn_for_swap_ix(
        program_id,
        pk(&payer),
        pool_state,
        pk(&user_token_a),
        pk(&user_token_b),
        vault_a,
        vault_b,
        100,
        true,
    );
    let ix_payout = swap_payout_ix(
        program_id,
        pk(&payer),
        pool_state,
        pool_authority,
        pk(&user_token_a),
        pk(&user_token_b),
        vault_a,
        vault_b,
        50,
        90,
        true,
    );

    let res = send_multi(&mut svm, &payer, &[&payer], &[ix_burn, ix_payout]);
    assert!(res.is_err(), "Expected failure when payout amount_in != burn amount");
}

#[test]
fn test_swap_slippage_fails() {
    let program_id = zeroxamm_introspection::id();
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let mint_a = Keypair::new();
    let mint_b = Keypair::new();

    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    inject_mint(&mut svm, &mint_a, &pk(&payer));
    inject_mint(&mut svm, &mint_b, &pk(&payer));

    let user_token_a = Keypair::new();
    let user_token_b = Keypair::new();
    inject_token_account(&mut svm, &pk(&user_token_a), &pk(&mint_a), &pk(&payer), 10_000);
    inject_token_account(&mut svm, &pk(&user_token_b), &pk(&mint_b), &pk(&payer), 10_000);

    let (pool_state, pool_authority, vault_a, vault_b) = setup_funded_pool(
        &mut svm,
        program_id,
        &payer,
        &mint_a,
        &mint_b,
        &user_token_a,
        &user_token_b,
        1_000,
        1_000,
    );

    inject_token_account(&mut svm, &pk(&user_token_a), &pk(&mint_a), &pk(&payer), 10_000);

    let ix_burn = burn_for_swap_ix(
        program_id,
        pk(&payer),
        pool_state,
        pk(&user_token_a),
        pk(&user_token_b),
        vault_a,
        vault_b,
        100,
        true,
    );
    let ix_payout = swap_payout_ix(
        program_id,
        pk(&payer),
        pool_state,
        pool_authority,
        pk(&user_token_a),
        pk(&user_token_b),
        vault_a,
        vault_b,
        100,
        95,
        true,
    );

    let res = send_multi(&mut svm, &payer, &[&payer], &[ix_burn, ix_payout]);
    assert!(res.is_err(), "Expected slippage failure");
}
