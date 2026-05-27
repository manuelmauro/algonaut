//! Offline tests for sourced (non-literal) `defaultValue` arguments.
//!
//! `global`/`local`/`box`/`method` sources need a runtime read, so the macro
//! generates those methods as `async fn (&Algod, …)` returning a builder: the
//! defaulted argument is dropped from the parameter list (the read supplies it
//! at call time). A `literal` default is still supplied synchronously, and a
//! plain required argument is untouched. These tests can't reach a node, so
//! they assert the generated *shapes* via the type-checker.

use algonaut::Algod;
use algonaut::contract;
use algonaut_core::AppId;
use algonaut_transaction::account::Account;
use std::sync::Arc;

contract!("tests/fixtures/sourced_defaults.json");

fn client() -> Defaults {
    let alice = Account::generate();
    let address = alice.address();
    Defaults::new(AppId(123), address, Arc::new(alice))
}

// Each sourced-default method takes `&Algod` and no value argument (the default
// is read at call time), and is `async`. This `async fn` compiles only if every
// generated method has that shape — a wrong signature is a compile error. It is
// never awaited (no node), so it only exercises the type-checker.
#[allow(dead_code)]
async fn exercise_sourced_default_shapes(
    c: &Defaults,
    algod: &Algod,
) -> Result<(), algonaut::Error> {
    let params = crate::contract_macro_arc56::mock_params();
    // global source: `from_global(&Algod)` — `amount` dropped.
    let _ = c.from_global(algod).await?.build(&params);
    // local source: `from_local(&Algod)` — reads the client's sender.
    let _ = c.from_local(algod).await?.build(&params);
    // box source: `from_box(&Algod)` — `label` dropped.
    let _ = c.from_box(algod).await?.build(&params);
    // method source: `from_method(&Algod)` — `seed` dropped.
    let _ = c.from_method(algod).await?.build(&params);
    Ok(())
}

#[test]
fn sourced_default_methods_are_async_take_algod_and_drop_the_arg() {
    // Reference the type-checked async fn so it is compiled.
    let _ = exercise_sourced_default_shapes;
}

#[test]
fn plain_argument_method_stays_sync_and_keeps_its_argument() {
    // The control method has an ordinary required argument and no default, so
    // it stays a synchronous builder method taking the argument.
    let c = client();
    let params = crate::contract_macro_arc56::mock_params();
    let _call = c.plain(7u64).build(&params);
}

#[test]
fn readonly_source_method_is_callable_directly_too() {
    // The read-only method that backs the `method` default is itself a normal
    // (sync, no-arg) builder method on the client.
    let c = client();
    let params = crate::contract_macro_arc56::mock_params();
    let _sim_builder = c.current_seed().build(&params);
}
