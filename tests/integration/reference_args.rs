//! Reference arguments (account/asset/application) on `contract!` methods (#346).
//!
//! The macro maps an `account`/`asset`/`application` argument to an
//! `Address`/`AssetId`/`AppId` parameter and hands the value to the method-call
//! builder, which appends it to the transaction's foreign-accounts/-assets/-apps
//! array and encodes the `uint8` index as the ABI argument. These tests build
//! the call offline and assert the resulting application-call transaction's
//! foreign arrays.

use algonaut::atomic::{AtomicGroupBuilder, MethodCall};
use algonaut::contract;
use algonaut::transaction::transaction::{ApplicationCallTransaction, TransactionType};
use algonaut_core::{AppId, AssetId};
use algonaut_crypto::HashDigest;
use algonaut_model::algod::SuggestedParams;
use algonaut_transaction::account::Account;
use std::sync::Arc;

contract!("tests/fixtures/references.json");

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

/// Build the single method call into a group and return its application-call
/// transaction, so tests can inspect the foreign arrays the builder populated.
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
fn account_reference_populates_foreign_accounts() {
    let sender = Account::generate();
    let target = Account::generate();
    let client = References::new(AppId(1), sender.address(), Arc::new(sender));
    let ac = app_call(client.use_account(target.address()).build(&mock_params()));
    assert_eq!(ac.accounts, Some(vec![target.address()]));
    // selector + one uint8 index argument
    assert_eq!(ac.app_arguments.as_ref().unwrap().len(), 2);
}

#[test]
fn asset_reference_populates_foreign_assets() {
    let sender = Account::generate();
    let client = References::new(AppId(1), sender.address(), Arc::new(sender));
    let ac = app_call(client.use_asset(AssetId(123)).build(&mock_params()));
    assert_eq!(ac.foreign_assets, Some(vec![AssetId(123)]));
    assert_eq!(ac.app_arguments.as_ref().unwrap().len(), 2);
}

#[test]
fn application_reference_populates_foreign_apps() {
    let sender = Account::generate();
    let client = References::new(AppId(1), sender.address(), Arc::new(sender));
    let ac = app_call(client.use_app(AppId(999)).build(&mock_params()));
    assert_eq!(ac.foreign_apps, Some(vec![AppId(999)]));
    assert_eq!(ac.app_arguments.as_ref().unwrap().len(), 2);
}

#[test]
fn reference_mixed_with_value_argument() {
    let sender = Account::generate();
    let target = Account::generate();
    let client = References::new(AppId(1), sender.address(), Arc::new(sender));
    let ac = app_call(client.mixed(target.address(), 7u64).build(&mock_params()));
    assert_eq!(ac.accounts, Some(vec![target.address()]));
    // selector + uint8 index + uint64 value
    assert_eq!(ac.app_arguments.as_ref().unwrap().len(), 3);
}
