//! Transaction arguments (`pay`/`axfer`/…) on `contract!` methods (#349).
//!
//! A transaction argument is a caller-supplied `TransactionWithSigner` that the
//! builder places immediately before the method call in the atomic group. These
//! tests build the group offline and assert the slot ordering.

use algonaut::atomic::{AtomicGroupBuilder, TransactionWithSigner};
use algonaut::contract;
use algonaut::transaction::Pay;
use algonaut::transaction::transaction::TransactionType;
use algonaut_core::{AppId, MicroAlgos};
use algonaut_crypto::HashDigest;
use algonaut_model::algod::SuggestedParams;
use algonaut_transaction::account::Account;
use std::sync::Arc;

contract!("tests/fixtures/transactions.json");

fn mock_params() -> SuggestedParams {
    SuggestedParams {
        consensus_version: "test".to_string(),
        fee: algonaut_core::MicroAlgos(0),
        genesis_hash: HashDigest([0u8; 32]),
        genesis_id: "test".to_string(),
        last_round: algonaut_core::Round(1000),
        min_fee: algonaut_core::MicroAlgos(1000),
    }
}

#[test]
fn payment_argument_is_placed_before_the_method_call() {
    let sender = Account::generate();
    let sender_addr = sender.address();
    let client = Deposits::new(AppId(1), sender_addr, Arc::new(sender));
    let params = mock_params();

    let pay = Pay::new(sender_addr, AppId(1).address(), MicroAlgos(50_000))
        .build(&params)
        .expect("build pay");
    let call = client
        .deposit(TransactionWithSigner::unsigned(pay), 50_000u64)
        .build(&params);

    let group = AtomicGroupBuilder::new()
        .add_method_call(call)
        .build()
        .expect("group builds");
    let txs = group.transactions();
    assert_eq!(txs.len(), 2, "the pay argument adds its own slot");
    assert!(matches!(
        txs[0].transaction.txn_type,
        TransactionType::Payment(_)
    ));
    assert!(matches!(
        txs[1].transaction.txn_type,
        TransactionType::ApplicationCallTransaction(_)
    ));
}
