use {
    anchor_lang::{
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    zeroxescrow_o1::state::Counter,
};

fn setup_svm() -> LiteSVM {
    let program_id = zeroxescrow_o1::id();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/zeroxescrow_o1.so");
    svm.add_program(program_id, bytes).unwrap();
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

#[test]
fn test_initialize() {
    let mut svm = setup_svm();
    let user = Keypair::new();
    let counter = Keypair::new();

    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();

    let ix = Instruction::new_with_bytes(
        zeroxescrow_o1::id(),
        &zeroxescrow_o1::instruction::Initialize {}.data(),
        zeroxescrow_o1::accounts::Initialize {
            counter: counter.pubkey(),
            user: user.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    let res = send(&mut svm, &user, &[&user, &counter], ix);
    assert!(res.is_ok(), "{res:?}");

    let account = svm.get_account(&counter.pubkey()).unwrap();
    assert_eq!(account.owner, zeroxescrow_o1::id());

    let counter_state =
        Counter::try_deserialize(&mut account.data.as_slice()).unwrap();
    assert_eq!(counter_state.count, 0);
}