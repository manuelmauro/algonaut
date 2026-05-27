//! `ufixedNxM` arguments on `contract!` methods (#342).
//!
//! `ufixedNxM` maps to the `Ufixed<N, M>` value newtype, which carries its
//! `M`-decimal-place scale in the type and wraps the raw, *unscaled* `N`-bit
//! integer (`round(real * 10^M)`). On the wire a `ufixedNxM` encodes exactly as
//! a `uintN` (`AbiValue::Int`), so these tests assert the same big-endian
//! integer bytes a `uintN` argument would produce. See
//! `docs/adr/contract-macro-ufixed-args.md`.

use algonaut::atomic::{AtomicGroupBuilder, MethodCall};
use algonaut::contract;
use algonaut::transaction::transaction::TransactionType;
use algonaut_abi::macro_support::Ufixed;
use algonaut_core::AppId;
use algonaut_crypto::HashDigest;
use algonaut_model::algod::SuggestedParams;
use algonaut_transaction::account::Account;
use num_bigint::BigUint;
use std::sync::Arc;

contract!("tests/fixtures/ufixed.json");

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
fn ufixed64x2_encodes_as_unscaled_uint64() {
    let sender = Account::generate();
    let client = Prices::new(AppId(1), sender.address(), Arc::new(sender));
    // ufixed64x2 value 1.50 -> raw 150; on the wire it is a big-endian uint64.
    let args = app_args(client.set_price(Ufixed::new(150u64)).build(&mock_params()));
    assert_eq!(args[1], 150u64.to_be_bytes().to_vec());
}

#[test]
fn ufixed256x10_encodes_as_32_byte_big_endian() {
    let sender = Account::generate();
    let client = Prices::new(AppId(1), sender.address(), Arc::new(sender));
    // A 256-bit ufixed encodes as a 32-byte big-endian integer.
    let raw = BigUint::from(123456789u64);
    let args = app_args(client.wide(Ufixed::new(raw.clone())).build(&mock_params()));
    let mut expected = vec![0u8; 32];
    let be = raw.to_bytes_be();
    expected[32 - be.len()..].copy_from_slice(&be);
    assert_eq!(args[1], expected);
}

#[test]
fn ufixed8x2_encodes_as_single_byte() {
    let sender = Account::generate();
    let client = Prices::new(AppId(1), sender.address(), Arc::new(sender));
    // ufixed8x2 value 0.99 -> raw 99, a single byte.
    let args = app_args(client.narrow(Ufixed::new(99u8)).build(&mock_params()));
    assert_eq!(args[1], vec![99u8]);
}
