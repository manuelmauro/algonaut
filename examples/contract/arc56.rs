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
//! Running it needs a funded account and a developer-API algod node: set
//! `ALGOD_URL`, `ALGOD_TOKEN`, and `ALICE_MNEMONIC` (a local `make sandbox`
//! satisfies the first two out of the box). Progress is logged through the
//! `log` crate; it defaults to `info`, and `RUST_LOG=debug` turns up the volume.

use algonaut::Algod;
use algonaut::abi::abi_type::AbiValue;
use algonaut::atomic::{AbiMethodReturnValue, AtomicGroupBuilder};
use algonaut::transaction::Signer;
use algonaut::transaction::account::Account;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::sync::Arc;

#[macro_use]
extern crate log;

// Generate the typed `Vault` client from its ARC-56 app spec. This expands to:
//   - the `Vault` client struct (+ `testnet()` / `mainnet()` constructors);
//   - the `Pair`, `Holder`, `Wrapper` argument structs;
//   - the `VaultEvent` enum and `Vault::decode_events`;
//   - per-method builders, a `deploy` constructor, and `global_*` state getters.
algonaut::contract!("tests/fixtures/vault.arc56.json");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    // Default to `info` so the walkthrough is visible out of the box; override
    // with `RUST_LOG` (e.g. `RUST_LOG=debug`) the usual way.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let algod_url = env::var("ALGOD_URL")?;
    let algod = Algod::new(&algod_url, &env::var("ALGOD_TOKEN")?)?;
    let alice = Account::from_mnemonic(&env::var("ALICE_MNEMONIC")?)?;
    let sender = alice.address();
    let signer: Arc<dyn Signer> = Arc::new(alice);
    let params = algod.suggested_params().await?;
    info!("connected to algod at {algod_url} as {sender}");

    // 1. Deploy: compile the contract's TEAL programs, submit an app-create with
    //    the declared state schema, and get a client bound to the new app id.
    info!("deploy: compiling the contract's TEAL and submitting app-create");
    let vault = Vault::deploy(&algod, sender, Arc::clone(&signer), &params).await?;
    info!("deploy: Vault is live as app id {}", vault.app_id().0);

    // If the app already exists, construct the client directly instead:
    //   let vault = Vault::new(AppId(123), sender, Arc::clone(&signer));
    // or use a named-network constructor from the spec's `networks`:
    //   let vault = Vault::testnet(sender, Arc::clone(&signer));

    // 2. Typed struct arguments, composed into one atomic group. `Pair` is the
    //    generated Rust struct for the ARC-56 `Pair`; `scale` mixes a struct
    //    with a plain scalar; `store_wrapped` takes a nested struct.
    info!(r#"call: grouping store(2,3) + scale((4,5),10) + store_wrapped((6,7),"demo")"#);
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
    info!(
        "call: group confirmed in round {}",
        executed.confirmed_round.unwrap_or_default()
    );
    // `scale` is the 2nd call in the group; surface its uint64 return value.
    if let Some(scale_result) = executed.method_results.get(1) {
        match &scale_result.return_value {
            Ok(AbiMethodReturnValue::Some(value)) => info!("call: scale returned {value:?}"),
            Ok(AbiMethodReturnValue::Void) => {}
            Err(e) => warn!("call: scale return decode failed: {e:?}"),
        }
    }

    // 3. A default argument: `incr`'s `step` has a literal default of 1, so the
    //    generated method takes an `Option<u64>` — pass `None` for the literal
    //    default, or `Some(v)` to override it.
    let _incr = vault.incr(None).build(&params);
    let _incr_override = vault.incr(Some(5)).build(&params);
    info!("default: built incr(None) (uses literal 1) and incr(Some(5)) (overrides it)");

    // 4. Lifecycle: `enroll` declares the `OptIn` call action, so its builder
    //    exposes `.opt_in()`, which sets the transaction's on-complete.
    let _opt_in = vault.enroll().opt_in().build(&params);
    info!("lifecycle: built enroll().opt_in() — OnComplete set to OptIn from the declared action");

    // 5. Read-only call through simulate: no fee, nothing submitted. `get_pair`
    //    returns `(uint64,uint64)`, decoded into the outcome.
    info!("simulate: dry-running get_pair (read-only, nothing submitted)");
    let simulated = vault.get_pair().simulate(&algod, &params).await?;
    if let Some(result) = simulated.method_results.first() {
        match &result.return_value {
            Ok(AbiMethodReturnValue::Some(value)) => info!("simulate: get_pair returned {value:?}"),
            Ok(AbiMethodReturnValue::Void) => {}
            Err(e) => warn!("simulate: get_pair decode failed: {e:?}"),
        }
    }

    // 6. Typed global-state reads, decoded per the key's declared ARC-56 type.
    if let Some(total) = vault.global_total(&algod).await? {
        info!("state: global `total` = {total:?}");
    }
    match vault.global_owner(&algod).await? {
        Some(owner) => info!("state: global `owner` = {owner:?}"),
        None => info!("state: global `owner` is unset"),
    }

    // 7. Decode ARC-28 events from a confirmed transaction's logs.
    if let Some(first) = executed.method_results.first()
        && let Some(logs) = &first.transaction_info.logs
    {
        let raw: Vec<Vec<u8>> = logs.iter().map(|log| log.0.clone()).collect();
        let events = Vault::decode_events(&raw);
        info!(
            "events: decoded {} from the first call's logs",
            events.len()
        );
        for event in events {
            match event {
                VaultEvent::Counted(AbiValue::Array(fields)) => {
                    info!("events: Counted -> {fields:?}");
                }
                other => info!("events: {other:?}"),
            }
        }
    }

    info!("done");
    Ok(())
}
