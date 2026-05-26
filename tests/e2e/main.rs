//! End-to-end tests for the ARC-56 `contract!` client against a live node.
//!
//! These deploy the generated `Vault` client and exercise the whole surface on
//! chain — deploy, an atomic group of typed-struct calls, a read-only simulate,
//! typed global-state reads, ARC-28 event decoding, the declared OptIn
//! lifecycle action, and a literal default argument.
//!
//! They are `#[ignore]`d, so the default `cargo test` (and CI without a node)
//! only *compiles* them. Run them against a local node with:
//!
//! ```sh
//! make sandbox     # a dev algod+kmd on :4001/:4002 with the dev token
//! make test-e2e    # cargo test --test e2e_contract_macro_arc56 -- --ignored
//! ```
//!
//! They self-fund: each test mints a fresh account and tops it up from the
//! sandbox's KMD default wallet, so no mnemonics or manual funding are needed.
//! `ALGOD_URL` / `ALGOD_TOKEN` / `KMD_URL` / `KMD_TOKEN` override the defaults.

use algonaut::abi::abi_type::AbiValue;
use algonaut::atomic::{AbiMethodReturnValue, AtomicGroupBuilder};
use algonaut::core::{Address, MicroAlgos};
use algonaut::transaction::account::Account;
use algonaut::transaction::{Pay, Signer};
use algonaut::{Algod, Kmd};
use std::env;
use std::sync::Arc;

algonaut::contract!("tests/fixtures/vault.arc56.json");

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn algod() -> Algod {
    Algod::new(
        &env_or("ALGOD_URL", "http://localhost:4001"),
        &env_or("ALGOD_TOKEN", &"a".repeat(64)),
    )
    .expect("algod client")
}

fn kmd() -> Kmd {
    Kmd::new(
        &env_or("KMD_URL", "http://localhost:4002"),
        &env_or("KMD_TOKEN", &"a".repeat(64)),
    )
    .expect("kmd client")
}

/// Mint a fresh account and fund it from the sandbox KMD default wallet, so the
/// tests need no pre-funded mnemonic. KMD signs the dispensing payment; the
/// fresh account (whose key we hold) then signs the contract interactions.
async fn funded_account(algod: &Algod, kmd: &Kmd) -> Account {
    let account = Account::generate();

    let wallet_id = kmd
        .list_wallets()
        .await
        .expect("list wallets")
        .wallets
        .into_iter()
        .find(|w| w.name == "unencrypted-default-wallet")
        .expect("default wallet present (run `make sandbox`)")
        .id;
    let handle = kmd
        .init_wallet_handle(&wallet_id, "")
        .await
        .expect("init wallet handle")
        .wallet_handle_token;
    let dispenser: Address = kmd
        .list_keys(&handle)
        .await
        .expect("list keys")
        .addresses
        .first()
        .expect("a funded genesis account")
        .parse()
        .expect("genesis address parses");

    let params = algod.suggested_params().await.expect("suggested params");
    let pay = Pay::new(dispenser, account.address(), MicroAlgos(10_000_000))
        .build(&params)
        .expect("build funding payment");
    let signed = kmd
        .sign(&handle, "", &pay)
        .await
        .expect("kmd signs funding payment")
        .signed_transaction;
    algod
        .submit_raw(&signed)
        .await
        .expect("submit funding payment")
        .confirm()
        .await
        .expect("funding payment confirms");

    account
}

/// Deploy a fresh `Vault` from a freshly funded account.
async fn deploy_vault(algod: &Algod, kmd: &Kmd) -> (Vault, Address) {
    let account = funded_account(algod, kmd).await;
    let sender = account.address();
    let signer: Arc<dyn Signer> = Arc::new(account);
    let params = algod.suggested_params().await.expect("suggested params");
    let vault = Vault::deploy(algod, sender, signer, &params)
        .await
        .expect("deploy Vault");
    (vault, sender)
}

#[tokio::test]
#[ignore = "requires a live algod+kmd; run via `make test-e2e` (see `make sandbox`)"]
async fn deploy_creates_app() {
    let (vault, _) = deploy_vault(&algod(), &kmd()).await;
    assert!(vault.app_id().0 > 0, "deploy should create an application");
}

#[tokio::test]
#[ignore = "requires a live algod+kmd; run via `make test-e2e` (see `make sandbox`)"]
async fn full_walkthrough() {
    let algod = algod();
    let (vault, sender) = deploy_vault(&algod, &kmd()).await;
    let params = algod.suggested_params().await.unwrap();

    // An atomic group of typed-struct calls: store(2,3), scale((4,5),10),
    // store_wrapped((6,7),"demo"). store sets total=5 and emits Counted(5);
    // store_wrapped sets total=6+7=13.
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
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap()
        .execute(&algod)
        .await
        .unwrap();
    assert!(executed.confirmed_round.is_some(), "group should confirm");

    // scale (2nd call) returns (4 + 5) * 10 = 90.
    match &executed.method_results[1].return_value {
        Ok(AbiMethodReturnValue::Some(value)) => assert_eq!(value, &AbiValue::from(90u64)),
        other => panic!("unexpected scale return: {other:?}"),
    }

    // ARC-28 events: store (the first call) logged Counted(5).
    let logs = executed.method_results[0]
        .transaction_info
        .logs
        .as_ref()
        .expect("store should produce a log");
    let raw: Vec<Vec<u8>> = logs.iter().map(|log| log.0.clone()).collect();
    assert_eq!(
        Vault::decode_events(&raw),
        vec![VaultEvent::Counted(AbiValue::Array(vec![AbiValue::from(
            5u64
        )]))],
    );

    // Read-only simulate: get_pair returns (total, total) = (13, 13).
    let simulated = vault.get_pair().simulate(&algod, &params).await.unwrap();
    match &simulated.method_results[0].return_value {
        Ok(AbiMethodReturnValue::Some(value)) => assert_eq!(
            value,
            &AbiValue::Array(vec![AbiValue::from(13u64), AbiValue::from(13u64)])
        ),
        other => panic!("unexpected get_pair return: {other:?}"),
    }

    // Typed global-state reads, decoded per the declared ARC-56 types.
    assert_eq!(
        vault.global_total(&algod).await.unwrap(),
        Some(AbiValue::from(13u64)),
    );
    assert_eq!(
        vault.global_owner(&algod).await.unwrap(),
        Some(AbiValue::Address(sender)),
    );
}

#[tokio::test]
#[ignore = "requires a live algod+kmd; run via `make test-e2e` (see `make sandbox`)"]
async fn opt_in_via_enroll_succeeds() {
    let algod = algod();
    let (vault, _) = deploy_vault(&algod, &kmd()).await;
    let params = algod.suggested_params().await.unwrap();

    // `enroll` declares the OptIn call action; `.opt_in()` sets the on-complete,
    // and the contract approves the opt-in.
    let opt_in = vault.enroll().opt_in().build(&params);
    let executed = AtomicGroupBuilder::new()
        .add_method_call(opt_in)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap()
        .execute(&algod)
        .await
        .unwrap();
    assert!(executed.confirmed_round.is_some(), "opt-in should confirm");
}

#[tokio::test]
#[ignore = "requires a live algod+kmd; run via `make test-e2e` (see `make sandbox`)"]
async fn incr_uses_literal_default_on_chain() {
    let algod = algod();
    let (vault, _) = deploy_vault(&algod, &kmd()).await;
    let params = algod.suggested_params().await.unwrap();

    // `incr`'s `step` has a literal default of 1, supplied automatically. On a
    // fresh deploy total=0, so incr returns 0 + 1 = 1.
    let incr = vault.incr().build(&params);
    let executed = AtomicGroupBuilder::new()
        .add_method_call(incr)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap()
        .execute(&algod)
        .await
        .unwrap();
    match &executed.method_results[0].return_value {
        Ok(AbiMethodReturnValue::Some(value)) => assert_eq!(value, &AbiValue::from(1u64)),
        other => panic!("unexpected incr return: {other:?}"),
    }
}
