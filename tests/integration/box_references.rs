//! Box references attached to `contract!` method calls (#348).
//!
//! Methods that read or write box storage need the box reference present on the
//! transaction. The generated builders expose `.box_ref(key)` (a box of this
//! app) and `.box_ref_of(app, key)`; the reference flows to the application-call
//! transaction. `boxMap` dynamic-key derivation is out of scope — the caller
//! supplies the raw key bytes.

use algonaut::atomic::{AtomicGroupBuilder, MethodCall};
use algonaut::contract;
use algonaut::transaction::transaction::{ApplicationCallTransaction, TransactionType};
use algonaut_core::AppId;
use algonaut_crypto::HashDigest;
use algonaut_model::algod::SuggestedParams;
use algonaut_transaction::account::Account;
use std::sync::Arc;

contract!("tests/fixtures/vault.arc56.json");

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

fn app_call(call: MethodCall) -> ApplicationCallTransaction {
    let group = AtomicGroupBuilder::new()
        .add_method_call(call)
        .build()
        .expect("group builds");
    let txs = group.transactions();
    match &txs[txs.len() - 1].transaction.txn_type {
        TransactionType::ApplicationCallTransaction(ac) => ac.clone(),
        other => panic!("expected an application call, got {other:?}"),
    }
}

#[test]
fn box_ref_is_attached_to_the_call() {
    let sender = Account::generate();
    let vault = Vault::new(AppId(1), sender.address(), Arc::new(sender));
    let call = vault
        .box_put("k".to_owned(), 42u64)
        .box_ref(b"k".to_vec())
        .build(&mock_params());
    let ac = app_call(call);
    let boxes = ac.boxes.expect("a box reference is attached");
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].name, b"k".to_vec());
    assert!(boxes[0].app_id.is_none());
}

#[test]
fn without_box_ref_no_boxes_are_attached() {
    let sender = Account::generate();
    let vault = Vault::new(AppId(1), sender.address(), Arc::new(sender));
    let call = vault.box_get("k".to_owned()).build(&mock_params());
    assert_eq!(app_call(call).boxes, None);
}

#[test]
fn box_ref_of_targets_another_app() {
    let sender = Account::generate();
    let vault = Vault::new(AppId(1), sender.address(), Arc::new(sender));
    let call = vault
        .box_get("k".to_owned())
        .box_ref_of(AppId(55), b"shared".to_vec())
        .build(&mock_params());
    let boxes = app_call(call).boxes.expect("a box reference is attached");
    assert_eq!(boxes[0].app_id, Some(AppId(55)));
    assert_eq!(boxes[0].name, b"shared".to_vec());
}
