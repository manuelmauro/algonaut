//! Array arguments (`T[]` and `T[N]`) on `contract!` methods (#347).
//!
//! `T[]` maps to `Vec<R>` and `T[N]` to `[R; N]` (with `R` the Rust mapping of
//! the element type); each element is encoded recursively and wrapped in
//! `AbiValue::Array`, the form `algonaut_abi` encodes both array kinds from.
//! These tests build the call offline and assert the encoded application
//! argument bytes.

use algonaut::atomic::{AtomicGroupBuilder, MethodCall};
use algonaut::contract;
use algonaut::transaction::transaction::TransactionType;
use algonaut_core::AppId;
use algonaut_crypto::HashDigest;
use algonaut_model::algod::SuggestedParams;
use algonaut_transaction::account::Account;
use std::sync::Arc;

contract!("tests/fixtures/arrays.json");

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
fn dynamic_uint64_array_encodes_with_length_prefix() {
    let sender = Account::generate();
    let client = Arrays::new(AppId(1), sender.address(), Arc::new(sender));
    let args = app_args(client.sum(vec![1u64, 2, 3]).build(&mock_params()));
    // arg[0] = 4-byte selector; arg[1] = uint16 length (0x0003) + three big-endian uint64s.
    let mut expected = vec![0x00u8, 0x03];
    for v in [1u64, 2, 3] {
        expected.extend_from_slice(&v.to_be_bytes());
    }
    assert_eq!(args[1], expected);
}

#[test]
fn static_uint64_array_encodes_without_length_prefix() {
    let sender = Account::generate();
    let client = Arrays::new(AppId(1), sender.address(), Arc::new(sender));
    let args = app_args(client.sum3([4u64, 5, 6]).build(&mock_params()));
    let mut expected = Vec::new();
    for v in [4u64, 5, 6] {
        expected.extend_from_slice(&v.to_be_bytes());
    }
    assert_eq!(args[1], expected);
}

#[test]
fn byte_array_still_maps_to_vec_u8() {
    let sender = Account::generate();
    let client = Arrays::new(AppId(1), sender.address(), Arc::new(sender));
    // byte[] keeps its canonical Vec<u8> mapping.
    let args = app_args(client.first_byte(vec![9u8, 8, 7]).build(&mock_params()));
    assert_eq!(args[1], vec![0x00u8, 0x03, 9, 8, 7]);
}

#[test]
fn nested_array_builds() {
    let sender = Account::generate();
    let client = Arrays::new(AppId(1), sender.address(), Arc::new(sender));
    // uint64[][] -> Vec<Vec<u64>>; just assert it builds and produces an arg.
    let args = app_args(
        client
            .matrix(vec![vec![1u64, 2], vec![3]])
            .build(&mock_params()),
    );
    assert!(args.len() >= 2);
}
