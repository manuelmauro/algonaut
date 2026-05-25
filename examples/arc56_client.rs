//! Full ARC-56 `contract!` example.
//!
//! A single `contract!("…/vault.arc56.json")` invocation generates a typed
//! client for an ARC-56 "Extended App Description", and this example exercises
//! the whole surface it produces:
//!
//! - **deploy** — compile the contract's TEAL source and create the app;
//! - **named structs** — `Pair`/`Holder`/`Wrapper` as typed arguments;
//! - **default arguments** — `incr`'s `step` defaults to a literal;
//! - **read-only simulate** — dry-run `get_pair` and read its return value;
//! - **lifecycle** — opt in via the declared `OptIn` action;
//! - **global state reads** — decoded per the declared ARC-56 type;
//! - **ARC-28 events** — decode a confirmed transaction's logs.
//!
//! It is written to compile as living documentation. Running it needs a funded
//! account and a developer-API algod node: set `ALGOD_URL`, `ALGOD_TOKEN`, and
//! `ALICE_MNEMONIC`.

use algonaut::Algod;
use algonaut::abi::abi_type::AbiValue;
use algonaut::atomic::{AbiMethodReturnValue, AtomicGroupBuilder};
use algonaut::transaction::Signer;
use algonaut::transaction::account::Account;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::sync::Arc;

// Generate the typed `Vault` client from its ARC-56 app spec. This expands to:
//   - the `Vault` client struct (+ `testnet()` / `mainnet()` constructors);
//   - the `Pair`, `Holder`, `Wrapper` argument structs;
//   - the `VaultEvent` enum and `Vault::decode_events`;
//   - per-method builders, a `deploy` constructor, and `global_*` state getters.
algonaut::contract!("tests/fixtures/vault.arc56.json");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let algod = Algod::new(&env::var("ALGOD_URL")?, &env::var("ALGOD_TOKEN")?)?;
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;
    let sender = alice.address();
    let signer: Arc<dyn Signer> = Arc::new(alice);
    let params = algod.suggested_params().await?;

    // 1. Deploy: compile the contract's TEAL programs, submit an app-create with
    //    the declared state schema, and get a client bound to the new app id.
    let vault = Vault::deploy(&algod, sender, Arc::clone(&signer), &params).await?;
    println!("deployed Vault as app {:?}", vault.app_id());

    // If the app already exists, construct the client directly instead:
    //   let vault = Vault::new(AppId(123), sender, Arc::clone(&signer));
    // or use a named-network constructor from the spec's `networks`:
    //   let vault = Vault::testnet(sender, Arc::clone(&signer));

    // 2. Typed struct arguments, composed into one atomic group. `Pair` is the
    //    generated Rust struct for the ARC-56 `Pair`; `scale` mixes a struct
    //    with a plain scalar; `store_wrapped` takes a nested struct.
    let store = vault
        .store(Pair {
            first: 2,
            second: 3,
        })
        .build(&params);
    let scale = vault
        .scale(
            Pair {
                first: 4,
                second: 5,
            },
            10,
        )
        .build(&params);
    let wrapped = vault
        .store_wrapped(Wrapper {
            inner: Pair {
                first: 6,
                second: 7,
            },
            label: "demo".to_owned(),
        })
        .build(&params);

    let executed = AtomicGroupBuilder::new()
        .add_method_call(store)
        .add_method_call(scale)
        .add_method_call(wrapped)
        .build()?
        .sign()
        .await?
        .execute(&algod)
        .await?;
    println!("group confirmed in round {:?}", executed.confirmed_round);

    // 3. A default argument: `incr`'s `step` has a literal default of 1, so the
    //    generated method takes no parameter — the constant is supplied for you.
    let _incr = vault.incr().build(&params);

    // 4. Lifecycle: `enroll` declares the `OptIn` call action, so its builder
    //    exposes `.opt_in()`, which sets the transaction's on-complete.
    let _opt_in = vault.enroll().opt_in().build(&params);

    // 5. Read-only call through simulate: no fee, nothing submitted. `get_pair`
    //    returns `(uint64,uint64)`, decoded into the outcome.
    let simulated = vault.get_pair().simulate(&algod, &params).await?;
    if let Some(result) = simulated.method_results.first() {
        match &result.return_value {
            Ok(AbiMethodReturnValue::Some(value)) => println!("get_pair -> {value:?}"),
            Ok(AbiMethodReturnValue::Void) => {}
            Err(e) => println!("get_pair decode error: {e:?}"),
        }
    }

    // 6. Typed global-state reads, decoded per the key's declared ARC-56 type.
    if let Some(total) = vault.global_total(&algod).await? {
        println!("global `total` = {total:?}");
    }
    let _owner = vault.global_owner(&algod).await?;

    // 7. Decode ARC-28 events from a confirmed transaction's logs.
    if let Some(first) = executed.method_results.first()
        && let Some(logs) = &first.transaction_info.logs
    {
        let raw: Vec<Vec<u8>> = logs.iter().map(|log| log.0.clone()).collect();
        for event in Vault::decode_events(&raw) {
            match event {
                VaultEvent::Counted(AbiValue::Array(fields)) => {
                    println!("Counted event fields: {fields:?}");
                }
                other => println!("event: {other:?}"),
            }
        }
    }

    Ok(())
}
