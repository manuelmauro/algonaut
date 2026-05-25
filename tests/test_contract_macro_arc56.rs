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

// Compile-only check: type-checks the generated read-only `simulate` path
// against the real simulate API. It is never called (no node to talk to);
// merely compiling it verifies the generated async method's signature and the
// `AtomicGroupBuilder`/`simulate` chain line up.
#[allow(dead_code)]
async fn readonly_simulate_typechecks(
    vault: &Vault,
    algod: &algonaut::Algod,
    params: &SuggestedParams,
) -> Result<algonaut::atomic::SimulateOutcome, algonaut::Error> {
    vault.get_pair().simulate(algod, params).await
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

#[test]
fn arc28_events_decode_from_logs() {
    use algonaut::abi::abi_type::{AbiType, AbiValue};
    use sha2::{Digest, Sha512_256};

    // Build a synthetic ARC-28 log for `Counted(uint64)` with total = 42:
    // the 4-byte selector followed by the ABI-encoded argument tuple.
    let selector = &Sha512_256::digest(b"Counted(uint64)")[..4];
    let tuple: AbiType = "(uint64)".parse().unwrap();
    let body = tuple
        .encode(AbiValue::Array(vec![AbiValue::from(42u64)]))
        .unwrap();
    let mut log = selector.to_vec();
    log.extend_from_slice(&body);

    // A log that matches no event selector is ignored.
    let noise = vec![0u8, 1, 2, 3, 4];

    let events = Vault::decode_events(&[noise, log]);
    assert_eq!(events.len(), 1);
    match &events[0] {
        VaultEvent::Counted(AbiValue::Array(fields)) => {
            assert_eq!(fields, &vec![AbiValue::from(42u64)]);
        }
        other => panic!("unexpected event: {other:?}"),
    }
}
