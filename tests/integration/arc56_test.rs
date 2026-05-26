//! A real ARC-56 spec (AlgoKit's `ARC56Test`) exercises two macro behaviours a
//! real-world contract relies on:
//!
//! - its `foo` method takes `Inputs`, whose `add`/`subtract` fields are inline
//!   nested structs — generated recursively as `InputsAdd`/`InputsSubtract`;
//! - it is created through an ABI `createApplication()` method
//!   (`bareActions.create: []`), which the generated `deploy` handles by
//!   passing that method's selector as the create transaction's app argument.

use crate::contract_macro_arc56::mock_params;
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
fn inline_nested_struct_argument_builds_a_call() {
    let alice = Account::generate();
    let address = alice.address();
    let client = ARC56Test::new(AppId(123), address, Arc::new(alice));

    // `foo` takes `Inputs`, whose `add`/`subtract` are inline nested structs;
    // the macro generates them as `InputsAdd`/`InputsSubtract`, so the typed
    // call composes the way the predecessor ADR promised for nested structs.
    let _call = client
        .foo(Inputs {
            add: InputsAdd { a: 1, b: 2 },
            subtract: InputsSubtract { a: 10, b: 5 },
        })
        .build(&mock_params());

    // The OptIn lifecycle method is still generated too.
    let _opt_in = client.opt_in_to_application();
}
