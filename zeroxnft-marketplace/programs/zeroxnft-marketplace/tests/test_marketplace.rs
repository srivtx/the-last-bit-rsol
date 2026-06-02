use {
    anchor_lang::{
        solana_program::{
            instruction::Instruction as AnchorInstruction, program_option::COption,
            program_pack::Pack, pubkey::Pubkey, system_program,
        },
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    anchor_spl::token::{
        spl_token::state::{Account as TokenAccountState, Mint as MintState},
        ID as TOKEN_PROGRAM_ID,
    },
    litesvm::LiteSVM,
    mpl_core::{instructions::{CreateCollectionV1Builder, CreateV1Builder}, types::DataState, ID as CORE_PROGRAM_ID},
    solana_account::Account,
    solana_address::Address,
    solana_clock::Clock,
    solana_instruction::{AccountMeta, Instruction as SvmInstruction},
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

fn marketplace_pda(program_id: &Pubkey, authority: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"marketplace", authority.as_ref()], program_id).0
}

fn listing_pda(program_id: &Pubkey, asset: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"listing", asset.as_ref()], program_id).0
}

fn offer_pda(program_id: &Pubkey, asset: &Pubkey, buyer: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"offer", asset.as_ref(), buyer.as_ref()], program_id).0
}

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

    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_700_000_000;
    svm.set_sysvar::<Clock>(&clock);

    svm.add_program(
        to_address(zeroxnft_marketplace::id()),
        include_bytes!("../../../target/deploy/zeroxnft_marketplace.so"),
    )
    .unwrap();

    svm.add_program(
        to_address(CORE_PROGRAM_ID),
        include_bytes!("../../../../zeroxnft-staking/tests/fixtures/mpl_core.so"),
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

fn inject_mint(svm: &mut LiteSVM, mint: &Pubkey, mint_authority: &Pubkey, decimals: u8) {
    let mut data = vec![0u8; MintState::LEN];
    Pack::pack(
        MintState {
            mint_authority: COption::Some(*mint_authority),
            supply: 1_000_000_000,
            decimals,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        &mut data,
    )
    .unwrap();

    svm.set_account(
        to_address(*mint),
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

fn create_collection_and_asset(
    svm: &mut LiteSVM,
    authority: &Keypair,
    owner: &Keypair,
) -> (Pubkey, Pubkey) {
    let collection = Keypair::new();
    let asset = Keypair::new();

    // Create collection.
    let ix = CreateCollectionV1Builder::new()
        .collection(pk(&collection))
        .payer(pk(authority))
        .update_authority(Some(pk(authority)))
        .name("TEST".to_string())
        .uri("https://example.com".to_string())
        .instruction();

    let res = send(svm, authority, &[authority, &collection], ix);
    assert!(res.is_ok(), "{res:?}");

    // Create asset in the collection.
    let ix = CreateV1Builder::new()
        .asset(pk(&asset))
        .collection(Some(pk(&collection)))
        .authority(Some(pk(authority)))
        .payer(pk(authority))
        .owner(Some(pk(owner)))
        .data_state(DataState::AccountState)
        .name("ASSET".to_string())
        .uri("https://example.com/a".to_string())
        .instruction();

    let res = send(svm, authority, &[authority, &asset], ix);
    assert!(res.is_ok(), "{res:?}");

    (pk(&collection), pk(&asset))
}

#[test]
fn test_list_and_delist() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let authority = Keypair::new();
    let maker = Keypair::new();
    let treasury = Keypair::new();

    svm.airdrop(&payer.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&authority.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&maker.pubkey(), 2_000_000_000).unwrap();

    let (collection, asset) = create_collection_and_asset(&mut svm, &authority, &maker);

    let program_id = zeroxnft_marketplace::id();
    let marketplace = marketplace_pda(&program_id, &pk(&authority));
    let listing = listing_pda(&program_id, &asset);

    // init marketplace
    let ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_marketplace::instruction::InitializeMarketplace { fee_bps: 250 }.data(),
        zeroxnft_marketplace::accounts::InitializeMarketplace {
            authority: pk(&authority),
            treasury: pk(&treasury),
            marketplace,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let res = send(&mut svm, &payer, &[&payer, &authority], ix);
    assert!(res.is_ok(), "{res:?}");

    // list
    let ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_marketplace::instruction::List {
            price: 1_000_000_000,
            payment_mint: Pubkey::default(),
        }
        .data(),
        zeroxnft_marketplace::accounts::List {
            maker: pk(&maker),
            payer: pk(&payer),
            marketplace,
            listing,
            asset,
            collection,
            core_program: CORE_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let res = send(&mut svm, &payer, &[&payer, &maker], ix);
    assert!(res.is_ok(), "{res:?}");

    assert!(svm.get_account(&to_address(listing)).is_some());

    // delist
    let ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_marketplace::instruction::Delist {}.data(),
        zeroxnft_marketplace::accounts::Delist {
            maker: pk(&maker),
            payer: pk(&payer),
            listing,
            asset,
            collection,
            core_program: CORE_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let res = send(&mut svm, &payer, &[&payer, &maker], ix);
    assert!(res.is_ok(), "{res:?}");
}

#[test]
fn test_buy_with_sol_splits_fee() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let authority = Keypair::new();
    let maker = Keypair::new();
    let buyer = Keypair::new();
    let treasury = Keypair::new();

    svm.airdrop(&payer.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&authority.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&maker.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&buyer.pubkey(), 3_000_000_000).unwrap();
    svm.airdrop(&treasury.pubkey(), 1_000_000_000).unwrap();

    let (collection, asset) = create_collection_and_asset(&mut svm, &authority, &maker);

    let program_id = zeroxnft_marketplace::id();
    let marketplace = marketplace_pda(&program_id, &pk(&authority));
    let listing = listing_pda(&program_id, &asset);

    let fee_bps = 250u16;
    let price = 1_000_000_000u64;
    let fee = price * fee_bps as u64 / 10_000;
    let maker_amount = price - fee;

    // init marketplace
    let ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_marketplace::instruction::InitializeMarketplace { fee_bps }.data(),
        zeroxnft_marketplace::accounts::InitializeMarketplace {
            authority: pk(&authority),
            treasury: pk(&treasury),
            marketplace,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    assert!(send(&mut svm, &payer, &[&payer, &authority], ix).is_ok());

    // list
    let ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_marketplace::instruction::List {
            price,
            payment_mint: Pubkey::default(),
        }
        .data(),
        zeroxnft_marketplace::accounts::List {
            maker: pk(&maker),
            payer: pk(&payer),
            marketplace,
            listing,
            asset,
            collection,
            core_program: CORE_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    assert!(send(&mut svm, &payer, &[&payer, &maker], ix).is_ok());

    let maker_before = svm.get_account(&to_address(pk(&maker))).unwrap().lamports;
    let treasury_before = svm.get_account(&to_address(pk(&treasury))).unwrap().lamports;

    // buy
    let ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_marketplace::instruction::Buy {}.data(),
        zeroxnft_marketplace::accounts::Buy {
            buyer: pk(&buyer),
            payer: pk(&payer),
            marketplace,
            maker: pk(&maker),
            treasury: pk(&treasury),
            listing,
            asset,
            collection,
            core_program: CORE_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    assert!(send(&mut svm, &payer, &[&payer, &buyer], ix).is_ok());

    let maker_after = svm.get_account(&to_address(pk(&maker))).unwrap().lamports;
    let treasury_after = svm.get_account(&to_address(pk(&treasury))).unwrap().lamports;

    assert!(
        maker_after - maker_before >= maker_amount,
        "maker delta too small: {}",
        maker_after - maker_before
    );
    assert_eq!(treasury_after - treasury_before, fee);

    let asset_acct = svm.get_account(&to_address(asset)).unwrap();
    let asset_state =
        mpl_core::accounts::BaseAssetV1::try_deserialize(&mut asset_acct.data.as_slice()).unwrap();
    assert_eq!(asset_state.owner, pk(&buyer));
}

#[test]
fn test_make_offer_and_cancel() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let authority = Keypair::new();
    let maker = Keypair::new();
    let buyer = Keypair::new();
    let treasury = Keypair::new();

    svm.airdrop(&payer.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&authority.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&maker.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&buyer.pubkey(), 3_000_000_000).unwrap();

    let (_collection, asset) = create_collection_and_asset(&mut svm, &authority, &maker);

    let program_id = zeroxnft_marketplace::id();
    let marketplace = marketplace_pda(&program_id, &pk(&authority));
    let offer = offer_pda(&program_id, &asset, &pk(&buyer));

    // init marketplace (needed for accept path later; harmless here)
    let ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_marketplace::instruction::InitializeMarketplace { fee_bps: 250 }.data(),
        zeroxnft_marketplace::accounts::InitializeMarketplace {
            authority: pk(&authority),
            treasury: pk(&treasury),
            marketplace,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    assert!(send(&mut svm, &payer, &[&payer, &authority], ix).is_ok());

    // make offer
    let amount = 500_000_000u64;
    let ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_marketplace::instruction::MakeOffer { amount }.data(),
        zeroxnft_marketplace::accounts::MakeOffer {
            buyer: pk(&buyer),
            asset: asset,
            offer,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let res = send(&mut svm, &payer, &[&payer, &buyer], ix);
    assert!(res.is_ok(), "{res:?}");
    assert!(svm.get_account(&to_address(offer)).is_some());

    // cancel offer
    let ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_marketplace::instruction::CancelOffer {}.data(),
        zeroxnft_marketplace::accounts::CancelOffer {
            buyer: pk(&buyer),
            asset: asset,
            offer,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let res = send(&mut svm, &payer, &[&payer, &buyer], ix);
    assert!(res.is_ok(), "{res:?}");
    assert!(svm.get_account(&to_address(offer)).is_none());
}

#[test]
fn test_make_offer_and_accept() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let authority = Keypair::new();
    let maker = Keypair::new();
    let buyer = Keypair::new();
    let treasury = Keypair::new();

    svm.airdrop(&payer.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&authority.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&maker.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&buyer.pubkey(), 3_000_000_000).unwrap();
    svm.airdrop(&treasury.pubkey(), 1_000_000_000).unwrap();

    let (collection, asset) = create_collection_and_asset(&mut svm, &authority, &maker);

    let program_id = zeroxnft_marketplace::id();
    let marketplace = marketplace_pda(&program_id, &pk(&authority));
    let offer = offer_pda(&program_id, &asset, &pk(&buyer));

    let fee_bps = 250u16;
    let ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_marketplace::instruction::InitializeMarketplace { fee_bps }.data(),
        zeroxnft_marketplace::accounts::InitializeMarketplace {
            authority: pk(&authority),
            treasury: pk(&treasury),
            marketplace,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    assert!(send(&mut svm, &payer, &[&payer, &authority], ix).is_ok());

    let amount = 800_000_000u64;
    let ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_marketplace::instruction::MakeOffer { amount }.data(),
        zeroxnft_marketplace::accounts::MakeOffer {
            buyer: pk(&buyer),
            asset: asset,
            offer,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let res = send(&mut svm, &payer, &[&payer, &buyer], ix);
    assert!(res.is_ok(), "{res:?}");

    let maker_before = svm.get_account(&to_address(pk(&maker))).unwrap().lamports;
    let treasury_before = svm.get_account(&to_address(pk(&treasury))).unwrap().lamports;

    let ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_marketplace::instruction::AcceptOffer {}.data(),
        zeroxnft_marketplace::accounts::AcceptOffer {
            maker: pk(&maker),
            payer: pk(&payer),
            buyer: pk(&buyer),
            marketplace,
            treasury: pk(&treasury),
            asset_key: asset,
            offer,
            asset,
            collection,
            core_program: CORE_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let res = send(&mut svm, &payer, &[&payer, &maker], ix);
    assert!(res.is_ok(), "{res:?}");

    let fee = amount * fee_bps as u64 / 10_000;
    let maker_amount = amount - fee;

    let maker_after = svm.get_account(&to_address(pk(&maker))).unwrap().lamports;
    let treasury_after = svm.get_account(&to_address(pk(&treasury))).unwrap().lamports;
    assert_eq!(maker_after - maker_before, maker_amount);
    assert_eq!(treasury_after - treasury_before, fee);

    let asset_acct = svm.get_account(&to_address(asset)).unwrap();
    let asset_state =
        mpl_core::accounts::BaseAssetV1::try_deserialize(&mut asset_acct.data.as_slice()).unwrap();
    assert_eq!(asset_state.owner, pk(&buyer));
}

#[test]
fn test_buy_with_token_splits_fee() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    let authority = Keypair::new();
    let maker = Keypair::new();
    let buyer = Keypair::new();
    let treasury = Keypair::new();

    svm.airdrop(&payer.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&authority.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&maker.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&buyer.pubkey(), 2_000_000_000).unwrap();

    let (collection, asset) = create_collection_and_asset(&mut svm, &authority, &maker);

    let program_id = zeroxnft_marketplace::id();
    let marketplace = marketplace_pda(&program_id, &pk(&authority));
    let listing = listing_pda(&program_id, &asset);

    let fee_bps = 250u16;
    let ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_marketplace::instruction::InitializeMarketplace { fee_bps }.data(),
        zeroxnft_marketplace::accounts::InitializeMarketplace {
            authority: pk(&authority),
            treasury: pk(&treasury),
            marketplace,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    assert!(send(&mut svm, &payer, &[&payer, &authority], ix).is_ok());

    // Create a fake USDC mint + ATAs in SVM.
    let payment_mint = Pubkey::new_unique();
    inject_mint(&mut svm, &payment_mint, &pk(&authority), 6);

    let buyer_ata = Pubkey::new_unique();
    let maker_ata = Pubkey::new_unique();
    let treasury_ata = Pubkey::new_unique();

    let price = 1_000_000_000u64;
    inject_token_account(&mut svm, &buyer_ata, &payment_mint, &pk(&buyer), price);
    inject_token_account(&mut svm, &maker_ata, &payment_mint, &pk(&maker), 0);
    inject_token_account(&mut svm, &treasury_ata, &payment_mint, &pk(&treasury), 0);

    // list with payment mint
    let ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_marketplace::instruction::List {
            price,
            payment_mint,
        }
        .data(),
        zeroxnft_marketplace::accounts::List {
            maker: pk(&maker),
            payer: pk(&payer),
            marketplace,
            listing,
            asset,
            collection,
            core_program: CORE_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    assert!(send(&mut svm, &payer, &[&payer, &maker], ix).is_ok());

    // buy_with_token
    let ix = AnchorInstruction::new_with_bytes(
        program_id,
        &zeroxnft_marketplace::instruction::BuyWithToken {}.data(),
        zeroxnft_marketplace::accounts::BuyWithToken {
            buyer: pk(&buyer),
            payer: pk(&payer),
            marketplace,
            treasury: pk(&treasury),
            listing,
            maker: pk(&maker),
            payment_mint,
            buyer_ata,
            maker_ata,
            treasury_ata,
            asset,
            collection,
            token_program: TOKEN_PROGRAM_ID,
            core_program: CORE_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let res = send(&mut svm, &payer, &[&payer, &buyer], ix);
    assert!(res.is_ok(), "{res:?}");

    let buyer_after = svm.get_account(&to_address(buyer_ata)).unwrap().data;
    let maker_after = svm.get_account(&to_address(maker_ata)).unwrap().data;
    let treasury_after = svm.get_account(&to_address(treasury_ata)).unwrap().data;

    let buyer_state = TokenAccountState::unpack(&buyer_after).unwrap();
    let maker_state = TokenAccountState::unpack(&maker_after).unwrap();
    let treasury_state = TokenAccountState::unpack(&treasury_after).unwrap();

    let fee = price * fee_bps as u64 / 10_000;
    let maker_amount = price - fee;

    assert_eq!(buyer_state.amount, 0);
    assert_eq!(maker_state.amount, maker_amount);
    assert_eq!(treasury_state.amount, fee);

    let asset_acct = svm.get_account(&to_address(asset)).unwrap();
    let asset_state =
        mpl_core::accounts::BaseAssetV1::try_deserialize(&mut asset_acct.data.as_slice()).unwrap();
    assert_eq!(asset_state.owner, pk(&buyer));
}

