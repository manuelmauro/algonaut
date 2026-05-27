//! End-to-end tests for the ARC-56 `contract!` client against a live node.
//!
//! These deploy the generated `Vault` client and exercise the whole surface on
//! chain — deploy, an atomic group of typed-struct calls, a read-only simulate,
//! typed global-state reads, ARC-28 event decoding, the declared OptIn
//! lifecycle action, and a literal default argument.
//!
//! The suite is `[[test]] test = false`, so the default `cargo test` (and CI
//! without a node) skips it, and `make check-e2e` compile-checks it. Run it
//! against a local node with:
//!
//! ```sh
//! make sandbox     # a dev algod+kmd on :4001/:4002 with the dev token
//! make test-e2e    # = cargo test --test e2e
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

// A real, AlgoKit-produced spec. It is created through an ABI `createApplication`
// method (not a bare create), carries a `someNumber` TEAL template variable (a
// generated `deploy` parameter), and its `foo` method (an inline-nested struct
// arg) is omitted from the client rather than failing compilation.
algonaut::contract!("tests/fixtures/arc56_test.arc56.json");

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

/// Deploy the real `ARC56Test` contract from a freshly funded account. Unlike
/// Vault, it is created through its ABI `createApplication()` method, and its
/// `someNumber` template variable is supplied as a `deploy` argument.
async fn deploy_arc56_test(algod: &Algod, kmd: &Kmd) -> (ARC56Test, Address) {
    let account = funded_account(algod, kmd).await;
    let sender = account.address();
    let signer: Arc<dyn Signer> = Arc::new(account);
    let params = algod.suggested_params().await.expect("suggested params");
    let client = ARC56Test::deploy(algod, sender, signer, &params, 1)
        .await
        .expect("deploy ARC56Test");
    (client, sender)
}

#[tokio::test]
async fn deploy_creates_app() {
    let (vault, _) = deploy_vault(&algod(), &kmd()).await;
    assert!(vault.app_id().0 > 0, "deploy should create an application");
}

#[tokio::test]
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

#[tokio::test]
async fn arc56_test_deploys_via_abi_create() {
    // The real, AlgoKit-produced contract is created on-chain through its ABI
    // `createApplication()` method (the generated `deploy` passes that method's
    // selector as the create transaction's app argument) with its `someNumber`
    // TEAL template variable substituted at deploy time. Its other methods need
    // features beyond this client (an inline-nested struct arg, a box
    // reference), so the deploy itself is what this exercises end-to-end.
    let (client, _) = deploy_arc56_test(&algod(), &kmd()).await;
    assert!(
        client.app_id().0 > 0,
        "ABI-method create with a substituted template variable should create the app"
    );
}

#[tokio::test]
async fn non_creator_can_call_and_owner_stays_the_creator() {
    // Vault records `owner = Txn.Sender` at create and never changes it, so a
    // second account calling the app exercises the caller != creator case: the
    // call succeeds (no access control), `total` reflects it, but `owner` stays
    // the deployer.
    let algod = algod();
    let kmd = kmd();

    // Account A deploys; the contract records owner = A.
    let (vault, creator) = deploy_vault(&algod, &kmd).await;

    // A different funded account B binds a client to the same app id.
    let caller = funded_account(&algod, &kmd).await;
    let caller_addr = caller.address();
    assert_ne!(
        caller_addr, creator,
        "the caller must differ from the creator"
    );
    let caller_signer: Arc<dyn Signer> = Arc::new(caller);
    let as_caller = Vault::new(vault.app_id(), caller_addr, caller_signer);

    // B calls store(8,9): total = 8 + 9 = 17. Vault has no access control, so a
    // non-creator's call confirms.
    let params = algod.suggested_params().await.unwrap();
    let store = as_caller
        .store(Pair {
            first: 8,
            second: 9,
        })
        .build(&params);
    let executed = AtomicGroupBuilder::new()
        .add_method_call(store)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap()
        .execute(&algod)
        .await
        .unwrap();
    assert!(
        executed.confirmed_round.is_some(),
        "a non-creator's call should confirm"
    );

    // `total` reflects B's call...
    assert_eq!(
        vault.global_total(&algod).await.unwrap(),
        Some(AbiValue::from(17u64)),
        "store(8,9) sets total = 8 + 9",
    );
    // ...but `owner` is still A: set at create, untouched by `store`.
    assert_eq!(
        vault.global_owner(&algod).await.unwrap(),
        Some(AbiValue::Address(creator)),
        "owner stays the creator even though a different account called",
    );
}

#[tokio::test]
async fn account_reference_round_trips() {
    use algonaut::atomic::AbiMethodReturnValue;
    let algod = algod();
    let (vault, _) = deploy_vault(&algod, &kmd()).await;
    let params = algod.suggested_params().await.unwrap();

    // A fresh account passed as an `account` reference; the contract echoes the
    // address it resolves from the Accounts foreign array.
    let target = Account::generate();
    let executed = AtomicGroupBuilder::new()
        .add_method_call(vault.who(target.address()).build(&params))
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap()
        .execute(&algod)
        .await
        .unwrap();
    match &executed.method_results[0].return_value {
        Ok(AbiMethodReturnValue::Some(value)) => {
            assert_eq!(value, &AbiValue::Address(target.address()))
        }
        other => panic!("unexpected who return: {other:?}"),
    }
}

#[tokio::test]
async fn array_arguments_sum_on_chain() {
    use algonaut::atomic::AbiMethodReturnValue;
    let algod = algod();
    let (vault, _) = deploy_vault(&algod, &kmd()).await;
    let params = algod.suggested_params().await.unwrap();

    // Dynamic array: sum([10,20,30]) = 60.
    let dynamic = vault.sum(vec![10u64, 20, 30]).build(&params);
    // Static array: sum3([1,2,3]) = 6.
    let stat = vault.sum3([1u64, 2, 3]).build(&params);
    let executed = AtomicGroupBuilder::new()
        .add_method_call(dynamic)
        .add_method_call(stat)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap()
        .execute(&algod)
        .await
        .unwrap();
    match &executed.method_results[0].return_value {
        Ok(AbiMethodReturnValue::Some(v)) => assert_eq!(v, &AbiValue::from(60u64)),
        other => panic!("unexpected sum return: {other:?}"),
    }
    match &executed.method_results[1].return_value {
        Ok(AbiMethodReturnValue::Some(v)) => assert_eq!(v, &AbiValue::from(6u64)),
        other => panic!("unexpected sum3 return: {other:?}"),
    }
}

#[tokio::test]
async fn box_put_get_round_trips() {
    use algonaut::atomic::AbiMethodReturnValue;
    use algonaut::core::MicroAlgos;
    use algonaut::transaction::Pay;
    let algod = algod();
    let kmd = kmd();
    let account = funded_account(&algod, &kmd).await;
    let sender = account.address();
    let signer: Arc<dyn Signer> = Arc::new(account);
    let params = algod.suggested_params().await.unwrap();
    let vault = Vault::deploy(&algod, sender, Arc::clone(&signer), &params)
        .await
        .expect("deploy Vault");

    // A box raises the app account's min balance, so fund the app account.
    let fund = Pay::new(sender, vault.app_id().address(), MicroAlgos(200_000))
        .build(&params)
        .expect("build app funding");
    AtomicGroupBuilder::new()
        .add_transaction(algonaut::atomic::TransactionWithSigner::new(
            fund,
            Arc::clone(&signer),
        ))
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap()
        .execute(&algod)
        .await
        .unwrap();

    // box_put("k", 42) writes the box; the box reference must be attached.
    let put = vault
        .box_put("k".to_owned(), 42)
        .box_ref(b"k".to_vec())
        .build(&params);
    AtomicGroupBuilder::new()
        .add_method_call(put)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap()
        .execute(&algod)
        .await
        .unwrap();

    // box_get("k") reads it back as 42.
    let get = vault
        .box_get("k".to_owned())
        .box_ref(b"k".to_vec())
        .build(&params);
    let executed = AtomicGroupBuilder::new()
        .add_method_call(get)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap()
        .execute(&algod)
        .await
        .unwrap();
    match &executed.method_results[0].return_value {
        Ok(AbiMethodReturnValue::Some(v)) => assert_eq!(v, &AbiValue::from(42u64)),
        other => panic!("unexpected box_get return: {other:?}"),
    }
}

#[tokio::test]
async fn payment_transaction_argument_round_trips() {
    use algonaut::atomic::TransactionWithSigner;
    use algonaut::core::MicroAlgos;
    use algonaut::transaction::Pay;
    let algod = algod();
    let kmd = kmd();
    let account = funded_account(&algod, &kmd).await;
    let sender = account.address();
    let signer: Arc<dyn Signer> = Arc::new(account);
    let params = algod.suggested_params().await.unwrap();
    let vault = Vault::deploy(&algod, sender, Arc::clone(&signer), &params)
        .await
        .expect("deploy Vault");

    // A payment to the app account, supplied as a transaction argument; the
    // builder places it immediately before the method call. The amount clears
    // the app account's 100_000 µAlgo base min balance, so the receiving app
    // account stays valid.
    let pay = Pay::new(sender, vault.app_id().address(), MicroAlgos(200_000))
        .build(&params)
        .expect("build pay");
    let call = vault
        .deposit(
            TransactionWithSigner::new(pay, Arc::clone(&signer)),
            200_000u64,
        )
        .build(&params);
    let executed = AtomicGroupBuilder::new()
        .add_method_call(call)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap()
        .execute(&algod)
        .await
        .unwrap();
    assert!(
        executed.confirmed_round.is_some(),
        "the grouped payment + method call should confirm"
    );
}
