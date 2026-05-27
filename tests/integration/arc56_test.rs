//! A real ARC-56 spec (AlgoKit's `ARC56Test`). This offline test covers the
//! inline-nested-struct generation a real-world contract relies on: its `foo`
//! method takes `Inputs`, whose `add`/`subtract` fields are inline nested
//! structs — generated recursively as `InputsAdd`/`InputsSubtract`.
//!
//! (The spec is also created through an ABI `createApplication()` method rather
//! than a bare create; the generated `deploy` handles that by passing the
//! method's selector as the create transaction's app argument, and the `e2e`
//! suite exercises that path end-to-end against a live node.)

use crate::contract_macro_arc56::mock_params;
use algonaut::contract;
use algonaut_core::AppId;
use algonaut_transaction::account::Account;
use std::sync::Arc;

contract!("tests/fixtures/arc56_test.arc56.json");

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

#[test]
fn state_accessors_are_generated_with_expected_signatures() {
    use algonaut::Algod;
    use algonaut::abi::abi_type::AbiValue;
    use algonaut_core::Address;
    use std::future::Future;

    // The accessors are `async` and hit algod, so they can't run offline. Bind
    // each one as a function pointer of its expected signature instead: a wrong
    // shape (missing the account arg, wrong key type, …) fails to compile, so
    // this asserts the generated API surface without a node.
    //
    // `arc56_test` declares (per its `state`):
    //   global key `globalKey` : AVMBytes -> uint64
    //   local  key `localKey`  : AVMBytes -> uint64   (needs an account)
    //   box    key `boxKey`    : AVMBytes -> string
    //   local map `localMap`   : AVMBytes -> string   (account + &[u8] key)
    // The `globalMap` (inline-struct value) and `boxMap` (struct *key*) are
    // skipped: their key/value types aren't decodable scalars.

    type Fetch<'a> =
        std::pin::Pin<Box<dyn Future<Output = Result<Option<AbiValue>, algonaut::Error>> + 'a>>;

    // global_<key>(&self, &Algod)
    fn assert_global<'a>(c: &'a ARC56Test, a: &'a Algod) -> Fetch<'a> {
        Box::pin(c.global_global_key(a))
    }
    // local_<key>(&self, &Algod, &Address)
    fn assert_local<'a>(c: &'a ARC56Test, a: &'a Algod, acct: &'a Address) -> Fetch<'a> {
        Box::pin(c.local_local_key(a, acct))
    }
    // box_<key>(&self, &Algod)
    fn assert_box<'a>(c: &'a ARC56Test, a: &'a Algod) -> Fetch<'a> {
        Box::pin(c.box_box_key(a))
    }
    // <class>_<map>(&self, &Algod, [&Address,] key)
    fn assert_local_map<'a>(c: &'a ARC56Test, a: &'a Algod, acct: &'a Address) -> Fetch<'a> {
        Box::pin(c.local_local_map(a, acct, b"k".as_slice()))
    }

    // Reference the items so the asserts above are exercised by the type-checker.
    let _ = (
        assert_global as fn(_, _) -> _,
        assert_local as fn(_, _, _) -> _,
        assert_box as fn(_, _) -> _,
        assert_local_map as fn(_, _, _) -> _,
    );
}
