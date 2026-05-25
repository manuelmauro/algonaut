//! Integration tests for ARC-56 features of the `contract!` macro.
//!
//! Phase 2 of the ARC-56 ADR: named structs become typed Rust structs that
//! methods accept as arguments, replacing the "tuple argument → compile error"
//! limitation.

use algonaut::contract;
use algonaut_core::AppId;
use algonaut_crypto::HashDigest;
use algonaut_model::algod::SuggestedParams;
use algonaut_transaction::account::Account;
use std::sync::Arc;

// Generate the Vault contract client, which uses named-struct arguments.
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

#[test]
fn generated_structs_exist_and_construct() {
    // Each ARC-56 struct becomes a typed Rust struct with public fields.
    let pair = Pair {
        first: 2,
        second: 3,
    };
    assert_eq!(pair.first, 2);

    let alice = Account::generate();
    let _holder = Holder {
        owner: alice.address(),
        amount: 100,
    };

    // Nested struct: a field referencing another generated struct.
    let _wrapper = Wrapper {
        inner: Pair {
            first: 1,
            second: 2,
        },
        label: "demo".to_string(),
    };
}

#[test]
fn struct_argument_builds_a_method_call() {
    let alice = Account::generate();
    let address = alice.address();
    let signer = Arc::new(alice);
    let vault = Vault::testnet(address, signer);
    assert_eq!(vault.app_id(), AppId(777));

    let params = mock_params();

    // A struct argument is passed as the typed Rust struct, not a tuple.
    let _call = vault
        .store(Pair {
            first: 10,
            second: 20,
        })
        .build(&params);
}

#[test]
fn nested_struct_and_mixed_args_build() {
    let alice = Account::generate();
    let address = alice.address();
    let signer = Arc::new(alice);
    let vault = Vault::new(AppId(777), address, signer);
    let params = mock_params();

    // Nested struct argument.
    let _wrapped = vault
        .store_wrapped(Wrapper {
            inner: Pair {
                first: 4,
                second: 5,
            },
            label: "hi".to_string(),
        })
        .build(&params);

    // A struct argument mixed with a scalar argument.
    let _scaled = vault
        .scale(
            Pair {
                first: 6,
                second: 7,
            },
            3u64,
        )
        .build(&params);

    // A method that returns a struct still builds (return decoding is a later
    // phase; the call itself is unaffected).
    let _get = vault.get_pair().build(&params);
}

#[test]
fn literal_default_argument_is_omitted() {
    let alice = Account::generate();
    let address = alice.address();
    let signer = Arc::new(alice);
    let vault = Vault::new(AppId(777), address, signer);
    let params = mock_params();

    // `incr`'s only argument has a literal default, so it takes no parameter:
    // the constant is supplied automatically.
    let _call = vault.incr().build(&params);
}
