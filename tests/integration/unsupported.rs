//! A controlled spec with a mix of supported and unsupported method arguments.
//! The macro yields a usable partial client: the supported methods are
//! generated (`add`, plus `opt_in_asset`, whose `asset` reference argument is
//! now mapped to an `AssetId`), while the rest (a transaction arg, a dynamic
//! array) are omitted and recorded in the client's doc comment — not turned
//! into a `compile_error!`.

use algonaut::contract;
use algonaut_core::AppId;
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

    // `transfer` (pay txn arg) and `batch_add` (uint64[]) are omitted:
    // referencing them here would not compile, which is exactly the
    // partial-client behaviour under test. (`opt_in_asset`'s `asset` reference
    // argument is now supported, so that method is generated.)
}
