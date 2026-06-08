//! Anonymous tuple arguments (`(T1,T2,…)`) on `contract!` methods (#342).
//!
//! A tuple type `(T1,T2,…)` maps to a Rust tuple `(R1,R2,…)` (with `Ri` the
//! Rust mapping of `Ti`); each element is encoded recursively and wrapped in
//! `AbiValue::Array`, the same representation named ARC-56 structs encode to
//! (a struct is a tuple under the hood). These tests build the call offline
//! and assert the encoded application argument bytes.

use algonaut::atomic::{AtomicGroupBuilder, MethodCall};
use algonaut::contract;
use algonaut::transaction::transaction::TransactionType;
use algonaut_core::{Address, AppId};
use algonaut_crypto::HashDigest;
use algonaut_model::algod::SuggestedParams;
use algonaut_transaction::account::Account;
use std::sync::Arc;

contract!("tests/fixtures/tuples.json");

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

fn app_args(call: MethodCall) -> Vec<Vec<u8>> {
    let group = AtomicGroupBuilder::new()
        .add_method_call(call)
        .build()
        .expect("group builds");
    let txs = group.transactions();
    match &txs[txs.len() - 1].transaction.txn_type {
        TransactionType::ApplicationCallTransaction(ac) => ac.app_arguments.clone().unwrap(),
        other => panic!("expected an application call, got {other:?}"),
    }
}

#[test]
fn static_pair_tuple_encodes_as_concatenated_head() {
    let sender = Account::generate();
    let client = Tuples::new(AppId(1), sender.address(), Arc::new(sender));
    // (uint64,uint64) is fully static: two big-endian uint64s, no length prefix.
    let args = app_args(client.store_pair((7u64, 9u64)).build(&mock_params()));
    let mut expected = Vec::new();
    expected.extend_from_slice(&7u64.to_be_bytes());
    expected.extend_from_slice(&9u64.to_be_bytes());
    assert_eq!(args[1], expected);
}

#[test]
fn mixed_static_tuple_encodes_in_field_order() {
    let sender = Account::generate();
    let client = Tuples::new(AppId(1), sender.address(), Arc::new(sender));
    let addr = Address([0xABu8; 32]);
    // (uint64,bool,address): 8 + 1 + 32 = 41 static bytes, in declared order.
    let args = app_args(client.mixed((42u64, true, addr)).build(&mock_params()));
    let mut expected = Vec::new();
    expected.extend_from_slice(&42u64.to_be_bytes());
    expected.push(0x80); // a single packed bool: the high bit set for `true`.
    expected.extend_from_slice(&[0xABu8; 32]);
    assert_eq!(args[1], expected);
}

#[test]
fn nested_tuple_with_dynamic_tail_builds() {
    let sender = Account::generate();
    let client = Tuples::new(AppId(1), sender.address(), Arc::new(sender));
    // ((uint64,uint64),string): the inner pair is the 16-byte static head, the
    // string is the dynamic tail. Just assert it builds and the head is correct.
    let args = app_args(
        client
            .nested(((1u64, 2u64), "hi".to_string()))
            .build(&mock_params()),
    );
    // head: inner pair (16 bytes) + 2-byte offset to the string tail (0x0012).
    let mut expected_head = Vec::new();
    expected_head.extend_from_slice(&1u64.to_be_bytes());
    expected_head.extend_from_slice(&2u64.to_be_bytes());
    expected_head.extend_from_slice(&[0x00, 0x12]);
    assert_eq!(&args[1][..18], &expected_head[..]);
    // tail: uint16 length (0x0002) + "hi".
    assert_eq!(&args[1][18..], &[0x00, 0x02, b'h', b'i']);
}

#[test]
fn tuple_containing_array_builds() {
    let sender = Account::generate();
    let client = Tuples::new(AppId(1), sender.address(), Arc::new(sender));
    // (uint64[],uint64) -> (Vec<u64>, u64); just assert it builds and encodes.
    let args = app_args(
        client
            .tuple_of_array((vec![1u64, 2u64], 3u64))
            .build(&mock_params()),
    );
    assert!(args.len() >= 2);
}
