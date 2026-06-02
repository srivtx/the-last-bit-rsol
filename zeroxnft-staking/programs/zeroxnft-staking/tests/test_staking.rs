use {
    anchor_lang::{
        solana_program::{
            instruction::Instruction as AnchorInstruction,
            program_option::COption,
            program_pack::Pack,
            pubkey::Pubkey,
            system_program,
        },
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    anchor_spl::token::{
        spl_token::{self, instruction::mint_to, state::Mint},
        ID as TOKEN_PROGRAM_ID,
    },
    litesvm::LiteSVM,
    mpl_core::{
        instructions::{CreateCollectionV1Builder, CreateV1Builder},
        types::{
            Attribute, Attributes, DataState, Plugin, PluginAuthority, PluginAuthorityPair,
        },
        ID as CORE_PROGRAM_ID,
    },
    solana_account::Account,
    solana_address::Address,
    solana_clock::Clock,
    solana_instruction::{AccountMeta, Instruction as SvmInstruction},
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    zeroxnft_staking::state::StakeConfig,
};

const REWARD_PER_SECOND: u64 = 1_000_000;
const STAKED_COUNT_KEY: &str = "staked_count";

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
    // Default LiteSVM clock is 0; program treats staked == "0" as unstaked.
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_700_000_000;
    svm.set_sysvar::<Clock>(&clock);

    svm.add_program(
        to_address(zeroxnft_staking::id()),
        include_bytes!("../../../target/deploy/zeroxnft_staking.so"),
    )
    .unwrap();
    svm.add_program(
        to_address(CORE_PROGRAM_ID),
        include_bytes!("../../../tests/fixtures/mpl_core.so"),
    )
    .unwrap();
    svm.add_program(
        to_address(TOKEN_PROGRAM_ID),
        include_bytes!("../../../tests/fixtures/spl_token.so"),
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
    let blockhash = svm.latest_blockhash();
    let ix = to_svm_ix(instruction);
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    let result = svm.send_transaction(tx);
    if result.is_ok() {
        svm.expire_blockhash();
        let clock = svm.get_sysvar::<Clock>();
        svm.warp_to_slot(clock.slot + 1);
    }
    result
}

fn inject_mint(svm: &mut LiteSVM, mint: Pubkey, authority: Pubkey) {
    let mut data = vec![0u8; Mint::LEN];
    Pack::pack(
        Mint {
            mint_authority: COption::Some(authority),
            supply: 0,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        &mut data,
    )
    .unwrap();
    svm.set_account(
        to_address(mint),
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

fn inject_token_account(svm: &mut LiteSVM, address: Pubkey, mint: Pubkey, owner: Pubkey, amount: u64) {
    let mut data = vec![0u8; spl_token::state::Account::LEN];
    Pack::pack(
        spl_token::state::Account {
            mint,
            owner,
            amount,
            delegate: COption::None,
            state: spl_token::state::AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        &mut data,
    )
    .unwrap();
    svm.set_account(
        to_address(address),
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

fn create_collection_with_staked_count(
    svm: &mut LiteSVM,
    authority: &Keypair,
    collection: &Keypair,
) {
    let plugins = vec![PluginAuthorityPair {
        plugin: Plugin::Attributes(Attributes {
            attribute_list: vec![Attribute {
                key: STAKED_COUNT_KEY.to_string(),
                value: "0".to_string(),
            }],
        }),
        authority: Some(PluginAuthority::UpdateAuthority),
    }];

    let ix = CreateCollectionV1Builder::new()
        .collection(pk(collection))
        .update_authority(Some(pk(authority)))
        .payer(pk(authority))
        .name("ZeroX Staking Collection".into())
        .uri("https://example.com/collection.json".into())
        .plugins(plugins)
        .instruction();

    let res = send(svm, authority, &[authority, collection], ix);
    assert!(res.is_ok(), "create collection failed: {res:?}");
}

fn create_asset_in_collection(
    svm: &mut LiteSVM,
    authority: &Keypair,
    owner: &Keypair,
    collection: &Keypair,
    asset: &Keypair,
) {
    let ix = CreateV1Builder::new()
        .asset(pk(asset))
        .collection(Some(pk(collection)))
        .authority(Some(pk(authority)))
        .payer(pk(authority))
        .owner(Some(pk(owner)))
        .data_state(DataState::AccountState)
        .name("ZeroX NFT".into())
        .uri("https://example.com/nft.json".into())
        .instruction();

    let res = send(svm, authority, &[authority, asset], ix);
    assert!(res.is_ok(), "create asset failed: {res:?}");
}

fn config_pdas(program_id: &Pubkey, collection: &Pubkey) -> (Pubkey, Pubkey) {
    let (stake_config, _) =
        Pubkey::find_program_address(&[b"config", collection.as_ref()], program_id);
    let (reward_vault, _) =
        Pubkey::find_program_address(&[b"reward_vault", stake_config.as_ref()], program_id);
    (stake_config, reward_vault)
}

fn advance_clock(svm: &mut LiteSVM, seconds: i64) {
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp += seconds;
    svm.set_sysvar::<Clock>(&clock);
}

#[test]
fn test_initialize_stake_claim_unstake() {
    let program_id = zeroxnft_staking::id();
    let mut svm = setup_svm();

    let authority = Keypair::new();
    let owner = Keypair::new();
    let collection = Keypair::new();
    let asset = Keypair::new();
    let reward_mint = Keypair::new();

    svm.airdrop(&authority.pubkey(), 50_000_000_000).unwrap();
    svm.airdrop(&owner.pubkey(), 10_000_000_000).unwrap();

    inject_mint(&mut svm, pk(&reward_mint), pk(&authority));

    create_collection_with_staked_count(&mut svm, &authority, &collection);
    create_asset_in_collection(&mut svm, &authority, &owner, &collection, &asset);

    let (stake_config, reward_vault) = config_pdas(&program_id, &pk(&collection));

    let init_ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_staking::instruction::Initialize {
            reward_per_second: REWARD_PER_SECOND,
        }
        .data(),
        zeroxnft_staking::accounts::Initialize {
            authority: pk(&authority),
            collection: pk(&collection),
            reward_mint: pk(&reward_mint),
            stake_config,
            reward_vault,
            token_program: TOKEN_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    assert!(send(&mut svm, &authority, &[&authority], init_ix).is_ok());

    let mint_ix = mint_to(
        &TOKEN_PROGRAM_ID,
        &pk(&reward_mint),
        &reward_vault,
        &pk(&authority),
        &[],
        50_000_000_000,
    )
    .unwrap();
    assert!(
        send(&mut svm, &authority, &[&authority], mint_ix).is_ok(),
        "mint to vault failed"
    );

    let user_reward_ata = anchor_spl::associated_token::get_associated_token_address(
        &pk(&owner),
        &pk(&reward_mint),
    );
    inject_token_account(&mut svm, user_reward_ata, pk(&reward_mint), pk(&owner), 0);

    let stake_ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_staking::instruction::Stake {}.data(),
        zeroxnft_staking::accounts::Stake {
            owner: pk(&owner),
            update_authority: pk(&authority),
            payer: pk(&authority),
            stake_config,
            asset: pk(&asset),
            collection: pk(&collection),
            core_program: CORE_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let stake_res = send(&mut svm, &authority, &[&authority, &owner], stake_ix);
    assert!(stake_res.is_ok(), "stake failed: {stake_res:?}");

    advance_clock(&mut svm, 5);

    let claim_ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_staking::instruction::ClaimRewards {}.data(),
        zeroxnft_staking::accounts::ClaimRewards {
            authority: pk(&authority),
            owner: pk(&owner),
            update_authority: pk(&authority),
            payer: pk(&authority),
            stake_config,
            reward_mint: pk(&reward_mint),
            reward_vault,
            user_reward_ata,
            asset: pk(&asset),
            collection: pk(&collection),
            core_program: CORE_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let claim_res = send(&mut svm, &authority, &[&authority, &owner], claim_ix);
    assert!(claim_res.is_ok(), "claim failed: {claim_res:?}");

    let user_after_claim = svm.get_account(&to_address(user_reward_ata)).unwrap();
    let user_amount = {
        let mut data = user_after_claim.data.as_slice();
        spl_token::state::Account::unpack(&mut data).unwrap().amount
    };
    assert!(user_amount > 0, "expected reward tokens after claim");

    advance_clock(&mut svm, 2);

    let unstake_ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_staking::instruction::Unstake {}.data(),
        zeroxnft_staking::accounts::Unstake {
            owner: pk(&owner),
            update_authority: pk(&authority),
            payer: pk(&authority),
            stake_config,
            asset: pk(&asset),
            collection: pk(&collection),
            core_program: CORE_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    assert!(send(&mut svm, &authority, &[&authority, &owner], unstake_ix).is_ok());

    let cfg = StakeConfig::try_deserialize(
        &mut svm
            .get_account(&to_address(stake_config))
            .unwrap()
            .data
            .as_slice(),
    )
    .unwrap();
    assert_eq!(cfg.collection, pk(&collection));
}

#[test]
fn test_stake_claim_unstake_again() {
    let program_id = zeroxnft_staking::id();
    let mut svm = setup_svm();

    let authority = Keypair::new();
    let owner = Keypair::new();
    let collection = Keypair::new();
    let asset = Keypair::new();
    let reward_mint = Keypair::new();

    svm.airdrop(&authority.pubkey(), 50_000_000_000).unwrap();
    svm.airdrop(&owner.pubkey(), 10_000_000_000).unwrap();
    inject_mint(&mut svm, pk(&reward_mint), pk(&authority));

    create_collection_with_staked_count(&mut svm, &authority, &collection);
    create_asset_in_collection(&mut svm, &authority, &owner, &collection, &asset);

    let (stake_config, reward_vault) = config_pdas(&program_id, &pk(&collection));

    let init_ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_staking::instruction::Initialize {
            reward_per_second: REWARD_PER_SECOND,
        }
        .data(),
        zeroxnft_staking::accounts::Initialize {
            authority: pk(&authority),
            collection: pk(&collection),
            reward_mint: pk(&reward_mint),
            stake_config,
            reward_vault,
            token_program: TOKEN_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    assert!(send(&mut svm, &authority, &[&authority], init_ix).is_ok());

    let mint_ix = mint_to(
        &TOKEN_PROGRAM_ID,
        &pk(&reward_mint),
        &reward_vault,
        &pk(&authority),
        &[],
        50_000_000_000,
    )
    .unwrap();
    assert!(send(&mut svm, &authority, &[&authority], mint_ix).is_ok());

    let user_reward_ata = anchor_spl::associated_token::get_associated_token_address(
        &pk(&owner),
        &pk(&reward_mint),
    );
    inject_token_account(&mut svm, user_reward_ata, pk(&reward_mint), pk(&owner), 0);

    let stake_accounts = zeroxnft_staking::accounts::Stake {
        owner: pk(&owner),
        update_authority: pk(&authority),
        payer: pk(&authority),
        stake_config,
        asset: pk(&asset),
        collection: pk(&collection),
        core_program: CORE_PROGRAM_ID,
        system_program: system_program::ID,
    };

    let stake_ix = |svm: &mut LiteSVM| {
        send(
            svm,
            &authority,
            &[&authority, &owner],
            AnchorInstruction::new_with_bytes(
                program_id,
                &zeroxnft_staking::instruction::Stake {}.data(),
                stake_accounts.to_account_metas(None),
            ),
        )
    };

    let unstake_ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_staking::instruction::Unstake {}.data(),
        zeroxnft_staking::accounts::Unstake {
            owner: pk(&owner),
            update_authority: pk(&authority),
            payer: pk(&authority),
            stake_config,
            asset: pk(&asset),
            collection: pk(&collection),
            core_program: CORE_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    assert!(stake_ix(&mut svm).is_ok(), "first stake failed");
    advance_clock(&mut svm, 3);
    let unstake_res = send(&mut svm, &authority, &[&authority, &owner], unstake_ix.clone());
    assert!(unstake_res.is_ok(), "unstake failed: {unstake_res:?}");
    let restake_res = stake_ix(&mut svm);
    assert!(restake_res.is_ok(), "restake failed: {restake_res:?}");
    advance_clock(&mut svm, 3);
    assert!(send(&mut svm, &authority, &[&authority, &owner], unstake_ix).is_ok());
}
