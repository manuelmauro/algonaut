//! A controlled spec with a mix of supported and unsupported method arguments.
//! The macro yields a usable partial client: the supported methods are
//! generated — `add`, `transfer` (a `pay` transaction arg), `opt_in_asset` (an
//! `asset` reference mapped to an `AssetId`), `batch_add` (a `uint64[]` mapped
//! to a `Vec`), and `add_fixed` (`ufixed64x2` arguments mapped to the `Ufixed`
//! newtype) — while `odd_width`, whose `uint24` argument has no canonical Rust
//! type, is omitted and recorded in the client's doc comment — not turned into
//! a `compile_error!`.

use algonaut::contract;
use algonaut_abi::macro_support::Ufixed;
use algonaut_core::{AppId, AssetId};
use algonaut_transaction::account::Account;
use std::sync::Arc;

contract!("tests/fixtures/calculator_unsupported.json");

#[test]
fn supported_method_present_unsupported_omitted() {
    let alice = Account::generate();
    let address = alice.address();
    let client = CalculatorWithUnsupported::new(AppId(1), address, Arc::new(alice));

    // `add(uint64,uint64)` is supported, so it is generated and callable.
    let _add = client.add(2, 3);

    // The reference-, array-, and ufixed-argument methods are generated now
    // too, and build a call without a node here. (`transfer`'s `pay`
    // transaction arg is likewise supported — exercised on-chain in
    // `transaction_args.rs`.)
    let _opt_in = client.opt_in_asset(AssetId(1));
    let _batch = client.batch_add(vec![1u64, 2, 3]);
    let _fixed = client.add_fixed(Ufixed::new(150u64), Ufixed::new(250u64));

    // `odd_width(uint24)` is omitted: `uint24` has no canonical Rust type, so
    // referencing `client.odd_width(..)` here would not compile. That omission
    // — rather than a `compile_error!` — is the behaviour under test: an
    // unsupported argument yields a usable partial client.
}
