//! Offline integration tests for the `contract!` macro's `deploy` enhancements
//! (issue #345): the `byteCode`-only deploy path and create-method constructor
//! arguments.
//!
//! `deploy` is `async` and submits to a node, so these tests cannot run it.
//! Instead they pin its *generated shape* — that it exists, and that its
//! parameter list matches what the spec implies — by type-checking calls inside
//! functions that are never executed. A signature change breaks compilation,
//! which is what we want to catch offline. The live behaviour is covered by the
//! `e2e` suite.

use algonaut::contract;
use algonaut_core::{Address, AppId};
use algonaut_model::algod::SuggestedParams;
use algonaut_transaction::account::Account;
use std::sync::Arc;

// A spec that ships only precompiled `byteCode` (no TEAL `source`) and is
// created through an ABI `createApplication(address,uint64)` method.
contract!("tests/fixtures/bytecode_deploy.arc56.json");

// A `byteCode`-only spec with no ABI create method: a bare app-create.
contract!("tests/fixtures/bytecode_bare.arc56.json");

/// A `byteCode`-only spec still generates a `deploy`: the macro builds the
/// approval/clear programs directly from the base64 byteCode rather than
/// compiling TEAL `source` through algod.
///
/// The create method `createApplication(address, uint64)` contributes two typed
/// `deploy` parameters (`owner: Address`, `seed: u64`) after the fixed
/// `algod`/`sender`/`signer`/`params` arguments — proving the constructor args
/// flow through as typed `deploy` parameters. The function is never called; it
/// exists so a signature change fails compilation.
#[allow(dead_code)]
async fn bytecode_deploy_has_create_method_params(
    algod: &algonaut::Algod,
    sender: Address,
    signer: Arc<dyn algonaut_transaction::Signer>,
    params: &SuggestedParams,
    owner: Address,
    seed: u64,
) -> ByteCodeDeploy {
    ByteCodeDeploy::deploy(algod, sender, signer, params, owner, seed)
        .await
        .unwrap()
}

/// A `byteCode`-only spec with a bare create (no ABI create method) generates a
/// `deploy` with no extra constructor parameters — just the fixed arguments.
#[allow(dead_code)]
async fn bytecode_bare_deploy_has_no_extra_params(
    algod: &algonaut::Algod,
    sender: Address,
    signer: Arc<dyn algonaut_transaction::Signer>,
    params: &SuggestedParams,
) -> ByteCodeBare {
    ByteCodeBare::deploy(algod, sender, signer, params)
        .await
        .unwrap()
}

/// The generated clients are otherwise ordinary: their non-create methods build
/// the way every other ARC-56 client's do.
#[test]
fn bytecode_clients_build_calls() {
    let alice = Account::generate();
    let address = alice.address();
    let client = ByteCodeDeploy::new(AppId(1), address, Arc::new(alice));
    let params = crate::contract_macro_arc56::mock_params();
    let _call = client.ping().build(&params);

    let bob = Account::generate();
    let baddr = bob.address();
    let bare = ByteCodeBare::new(AppId(2), baddr, Arc::new(bob));
    let _call = bare.ping().build(&params);
}
