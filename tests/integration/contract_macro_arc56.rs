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

pub(crate) fn mock_params() -> SuggestedParams {
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
fn literal_default_argument_is_overridable() {
    let alice = Account::generate();
    let address = alice.address();
    let signer = Arc::new(alice);
    let vault = Vault::new(AppId(777), address, signer);
    let params = mock_params();

    // `incr`'s only argument has a literal default, so it takes an `Option<u64>`
    // parameter: `None` supplies the spec default (1), `Some(v)` overrides it.
    let _default = vault.incr(None).build(&params);
    let _override = vault.incr(Some(5u64)).build(&params);
}

#[test]
fn declared_call_actions_get_lifecycle_setters() {
    let alice = Account::generate();
    let address = alice.address();
    let signer = Arc::new(alice);
    let vault = Vault::new(AppId(777), address, signer);
    let params = mock_params();

    // `enroll` declares the OptIn call action, so its builder exposes
    // `.opt_in()`, which sets the OnComplete on the built call.
    let _opt_in = vault.enroll().opt_in().build(&params);
    // NoOp is still the default when no lifecycle setter is used.
    let _noop = vault.enroll().build(&params);
}

#[test]
fn local_and_box_map_accessors_are_generated() {
    use algonaut::Algod;
    use algonaut::abi::abi_type::AbiValue;
    use algonaut_core::Address;
    use std::future::Future;

    // Vault declares (in its `state`): a global `total`/`owner`, a local `seen`
    // key, and a `boxes` box map. The accessors are `async` and hit algod, so
    // bind each as a future of its expected signature — a wrong shape (missing
    // the account for local, wrong key type for the map) fails to compile.
    type Fetch<'a> =
        std::pin::Pin<Box<dyn Future<Output = Result<Option<AbiValue>, algonaut::Error>> + 'a>>;

    // global_<key>(&self, &Algod) — no extra argument.
    fn global<'a>(v: &'a Vault, a: &'a Algod) -> Fetch<'a> {
        Box::pin(v.global_total(a))
    }
    // local_<key>(&self, &Algod, &Address) — per account.
    fn local<'a>(v: &'a Vault, a: &'a Algod, acct: &'a Address) -> Fetch<'a> {
        Box::pin(v.local_seen(a, acct))
    }
    // box_<map>(&self, &Algod, key) — AVMString key takes `&str`.
    fn box_map<'a>(v: &'a Vault, a: &'a Algod) -> Fetch<'a> {
        Box::pin(v.box_boxes(a, "counter"))
    }

    let _ = (
        global as fn(_, _) -> _,
        local as fn(_, _, _) -> _,
        box_map as fn(_, _) -> _,
    );
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

#[test]
fn struct_abi_decode_round_trips() {
    use algonaut::abi::abi_type::AbiValue;

    // Each generated struct gains an `abi_decode` that is the exact inverse of
    // `abi_encode`: encode → decode returns an equal value, with scalars coming
    // back as their Rust types (u64, Address, String).
    let pair = Pair {
        first: 10,
        second: 20,
    };
    let decoded = Pair::abi_decode(pair.clone().abi_encode()).unwrap();
    assert_eq!(decoded.first, 10);
    assert_eq!(decoded.second, 20);

    let alice = Account::generate();
    let holder = Holder {
        owner: alice.address(),
        amount: 100,
    };
    let decoded = Holder::abi_decode(holder.clone().abi_encode()).unwrap();
    assert_eq!(decoded.owner, alice.address());
    assert_eq!(decoded.amount, 100);

    // A struct with a nested-struct field and a string field round-trips too.
    let wrapper = Wrapper {
        inner: Pair {
            first: 1,
            second: 2,
        },
        label: "demo".to_string(),
    };
    let decoded = Wrapper::abi_decode(wrapper.abi_encode()).unwrap();
    assert_eq!(decoded.inner.first, 1);
    assert_eq!(decoded.inner.second, 2);
    assert_eq!(decoded.label, "demo");

    // A shape mismatch (an integer where a tuple is expected) is an error, not a
    // panic.
    assert!(Pair::abi_decode(AbiValue::from(7u64)).is_err());
    // Wrong arity (one element where two are expected) is also an error.
    assert!(Pair::abi_decode(AbiValue::Array(vec![AbiValue::from(7u64)])).is_err());
}

#[test]
fn struct_decode_from_wire_bytes() {
    use algonaut::abi::abi_type::{AbiType, AbiValue};

    // Decoding the on-wire ABI bytes (what a return/state/event read yields)
    // back through the named struct gives the typed value. `(uint64,uint64)` is
    // Pair's tuple type; this mirrors what `get_pair`'s `struct: "Pair"` return
    // overlay does after a simulate.
    let tuple: AbiType = "(uint64,uint64)".parse().unwrap();
    let wire = tuple
        .encode(AbiValue::Array(vec![
            AbiValue::from(13u64),
            AbiValue::from(13u64),
        ]))
        .unwrap();
    let value = tuple.decode(&wire).unwrap();
    let pair = Pair::abi_decode(value).unwrap();
    assert_eq!(pair.first, 13);
    assert_eq!(pair.second, 13);
}
