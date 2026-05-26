//! A real ARC-56 spec (AlgoKit's `ARC56Test`) drives two macro behaviours that
//! a real-world contract relies on:
//!
//! - its `foo` method takes an inline-nested struct the macro can't model, so
//!   it is **omitted** from the client (and listed in the struct doc) rather
//!   than emitting a `compile_error!` that would sink the whole spec;
//! - it is created through an ABI `createApplication()` method
//!   (`bareActions.create: []`), which the generated `deploy` handles by
//!   passing that method's selector as the create transaction's app argument.

use algonaut::contract;
use algonaut_core::AppId;
use algonaut_transaction::account::Account;
use std::sync::Arc;

contract!("tests/fixtures/arc56_test.arc56.json");

// Compile-only: the ABI-method create path in the generated `deploy`
// type-checks against the real APIs, including the `some_number` parameter
// generated for the spec's `someNumber` template variable. Never called.
#[allow(dead_code)]
async fn deploy_typechecks(
    algod: &algonaut::Algod,
    sender: algonaut_core::Address,
    signer: Arc<dyn algonaut_transaction::Signer>,
    params: &algonaut_model::algod::SuggestedParams,
) -> Result<ARC56Test, algonaut::Error> {
    ARC56Test::deploy(algod, sender, signer, params, 42).await
}

#[test]
fn unsupported_method_is_omitted_not_a_compile_error() {
    // Compiling this module at all is the assertion: the macro omitted `foo`
    // (inline-nested struct argument) instead of failing the build. The
    // supported OptIn lifecycle method is still generated and callable.
    let alice = Account::generate();
    let address = alice.address();
    let client = ARC56Test::new(AppId(123), address, Arc::new(alice));
    let _opt_in = client.opt_in_to_application();
}
