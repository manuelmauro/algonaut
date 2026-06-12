use crate::step_defs::integration::world::World;
use algonaut::dryrun::{DryrunRequestBuilder, field_name, result};
use algonaut_core::{Address, AppId, CompiledTeal, MicroAlgos};
use algonaut_encoding::Bytes;
use algonaut_model::algod::{
    Account, Application, ApplicationLocalState, ApplicationParams, ApplicationStateSchema,
    DryrunSource,
};
use algonaut_transaction::{Pay, builder::TransactionParams, contract_account::ContractAccount};
use cucumber::{given, then, when};
use std::fs;

const DRYRUN_APP_ID: u64 = 1;
const NONEXISTENT_SENDER: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAY5HFKQ";
// Secondary account address used in localwrite.teal tests for accounts[1].
// This address is hardcoded in the shared algorand-sdk-testing feature files.
const SECONDARY_ACCOUNT: &str = "6Z3C3LDVWGMX23BMSYMANACQOSINPFIRF77H7N3AWJZYV6OH6GWTJKVMXY";

fn read_program(name: &str) -> Vec<u8> {
    fs::read(format!("tests/cucumber/features/resources/{name}"))
        .unwrap_or_else(|e| panic!("failed to read tests/cucumber/features/resources/{name}: {e}"))
}

#[derive(Debug, Clone)]
struct DryrunParams;
impl TransactionParams for DryrunParams {
    fn last_round(&self) -> u64 {
        0
    }
    fn min_fee(&self) -> u64 {
        1000
    }
    fn genesis_hash(&self) -> algonaut_crypto::HashDigest {
        algonaut_crypto::HashDigest([0u8; 32])
    }
    fn genesis_id(&self) -> &String {
        static EMPTY: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        EMPTY.get_or_init(String::new)
    }
}

fn dummy_payment_txn(sender: Address) -> algonaut_transaction::Transaction {
    let params = DryrunParams;
    Pay::new(sender, sender, MicroAlgos(0))
        .build(&params)
        .expect("building dummy payment")
}

// -------------------------------------------------------------------
// dryrun.feature
// -------------------------------------------------------------------

#[when(regex = r#"^I dryrun a "([^"]+)" program "([^"]+)"$"#)]
async fn i_dryrun_a_program(w: &mut World, kind: String, program: String) {
    let algod = w.algod.as_ref().expect("algod not set");
    let bytes = read_program(&program);

    // Wrap the program as a logic-sig over a noop payment.
    let (signed, sources) = match kind.as_str() {
        "compiled" => {
            let compiled = CompiledTeal(bytes);
            let ca = ContractAccount::new(compiled);
            let txn = dummy_payment_txn(*ca.address());
            let signed = ca.sign(txn, vec![]).expect("signing contract account");
            (signed, Vec::<DryrunSource>::new())
        }
        "source" => {
            // Empty logic-sig in the txn; algod will compile + inject
            // via DryrunSource.
            let placeholder =
                ContractAccount::new(CompiledTeal(vec![0x02, 0x20, 0x01, 0x01, 0x22]));
            let txn = dummy_payment_txn(*placeholder.address());
            let signed = placeholder.sign(txn, vec![]).expect("signing placeholder");
            let src = DryrunSource {
                app_index: 0,
                field_name: field_name::LSIG.to_string(),
                source: String::from_utf8(bytes).expect("teal source must be utf8"),
                txn_index: 0,
            };
            (signed, vec![src])
        }
        other => panic!("unknown dryrun program kind: {other}"),
    };

    let mut builder = DryrunRequestBuilder::from_signed_txns(&[signed]).expect("from_signed_txns");
    for src in sources {
        builder = builder.add_source(src);
    }
    let request = builder.build();

    let resp = algod.teal_dryrun(Some(request)).await.expect("teal_dryrun");
    w.dryrun_response = Some(resp);
}

#[then(regex = r#"^I get execution result "([^"]+)"$"#)]
async fn i_get_execution_result(w: &mut World, expected: String) {
    let resp = w.dryrun_response.as_ref().expect("dryrun response not set");
    let status = result::first_status(resp).unwrap_or("");
    assert_eq!(status, expected, "dryrun status mismatch");
}

// -------------------------------------------------------------------
// dryrun_testing.feature
// -------------------------------------------------------------------

fn build_dryrun_test_case(program_path: &str, kind: &str) -> algonaut_model::algod::DryrunRequest {
    let raw = read_program(program_path);
    let is_compiled = program_path.ends_with(".tok");
    let creator: Address = NONEXISTENT_SENDER.parse().expect("dummy address");

    match kind {
        "lsig" => {
            let (signed, sources) = if is_compiled {
                let ca = ContractAccount::new(CompiledTeal(raw));
                let txn = dummy_payment_txn(*ca.address());
                let signed = ca.sign(txn, vec![]).unwrap();
                (signed, Vec::<DryrunSource>::new())
            } else {
                let placeholder =
                    ContractAccount::new(CompiledTeal(vec![0x02, 0x20, 0x01, 0x01, 0x22]));
                let txn = dummy_payment_txn(*placeholder.address());
                let signed = placeholder.sign(txn, vec![]).unwrap();
                (
                    signed,
                    vec![DryrunSource {
                        app_index: 0,
                        field_name: field_name::LSIG.to_string(),
                        source: String::from_utf8(raw.clone()).expect("teal source must be utf8"),
                        txn_index: 0,
                    }],
                )
            };

            let mut builder = DryrunRequestBuilder::from_signed_txns(&[signed]).unwrap();
            for src in sources {
                builder = builder.add_source(src);
            }
            builder.build()
        }
        "approv" | "clearp" => {
            // App call to a synthetic application id 1. The dryrun
            // request supplies the app's programs.
            let (approval, clear, sources) = if is_compiled {
                if kind == "approv" {
                    (
                        raw.clone(),
                        vec![0x02, 0x20, 0x01, 0x01, 0x22],
                        Vec::<DryrunSource>::new(),
                    )
                } else {
                    (vec![0x02, 0x20, 0x01, 0x01, 0x22], raw.clone(), vec![])
                }
            } else {
                let src = DryrunSource {
                    app_index: DRYRUN_APP_ID,
                    field_name: kind.to_string(),
                    source: String::from_utf8(raw).expect("teal source must be utf8"),
                    txn_index: 0,
                };
                (
                    vec![0x02, 0x20, 0x01, 0x01, 0x22],
                    vec![0x02, 0x20, 0x01, 0x01, 0x22],
                    vec![src],
                )
            };

            // The dryrun program writes global/local state, so the app must
            // declare a schema with room for those writes — otherwise algod
            // rejects with "store … count exceeds schema … count 0" and the
            // expected deltas never appear.
            let schema = || {
                Box::new(ApplicationStateSchema {
                    num_byte_slice: 64,
                    num_uint: 64,
                })
            };
            let app = Application {
                id: DRYRUN_APP_ID,
                params: Box::new(ApplicationParams {
                    creator: creator.to_string(),
                    approval_program: Bytes(approval),
                    clear_state_program: Bytes(clear),
                    extra_program_pages: None,
                    global_state: None,
                    global_state_schema: Some(schema()),
                    local_state_schema: Some(schema()),
                }),
            };

            // Build an app-call txn referencing app id 1.
            // Include the secondary account so that localwrite.teal can write
            // to accounts[1].
            // For clearp, use ClearApplication (on_complete=ClearState);
            // for approv, use CallApplication (on_complete=NoOp).
            let params = DryrunParams;
            let secondary: Address = SECONDARY_ACCOUNT.parse().expect("secondary address");
            let txn = if kind == "clearp" {
                algonaut_transaction::builder::ClearApplication::new(creator, AppId(DRYRUN_APP_ID))
                    .accounts(vec![secondary])
                    .build(&params)
                    .expect("app call txn")
            } else {
                algonaut_transaction::builder::CallApplication::new(creator, AppId(DRYRUN_APP_ID))
                    .accounts(vec![secondary])
                    .build(&params)
                    .expect("app call txn")
            };

            // Sign as a no-op contract account (the dryrun endpoint does
            // not validate signatures).
            let placeholder =
                ContractAccount::new(CompiledTeal(vec![0x02, 0x20, 0x01, 0x01, 0x22]));
            let signed = placeholder.sign(txn, vec![]).unwrap();

            // For local-state writes to work, accounts must be "opted in" to
            // the app. The cucumber scenarios assert local deltas at specific
            // account indices:
            //   - approv tests check index 0 (the sender/creator)
            //   - clearp tests check index 1 (a secondary account)
            // Following the Python SDK pattern, we supply two accounts both
            // with apps_local_state for our synthetic app id.
            let local_state_schema = ApplicationStateSchema {
                num_byte_slice: 64,
                num_uint: 64,
            };
            let make_opted_in_account = |addr: &str| Account {
                address: addr.to_string(),
                amount: 1_000_000,
                amount_without_pending_rewards: 1_000_000,
                min_balance: 100_000,
                pending_rewards: 0,
                rewards: 0,
                round: 0,
                status: "Offline".to_string(),
                total_apps_opted_in: 1,
                total_assets_opted_in: 0,
                total_created_apps: 0,
                total_created_assets: 0,
                apps_local_state: Some(vec![ApplicationLocalState::new(
                    DRYRUN_APP_ID,
                    local_state_schema.clone(),
                )]),
                ..Default::default()
            };

            // Account 0: the creator/sender (NONEXISTENT_SENDER / zero address)
            // Account 1: the secondary account (from the shared SDK test fixtures)
            let accounts = vec![
                make_opted_in_account(&creator.to_string()),
                make_opted_in_account(SECONDARY_ACCOUNT),
            ];

            let mut builder = DryrunRequestBuilder::from_signed_txns(&[signed]).unwrap();
            builder = builder.apps(vec![app]).accounts(accounts);
            for src in sources {
                builder = builder.add_source(src);
            }
            builder.build()
        }
        other => panic!("unknown dryrun test kind: {other}"),
    }
}

#[given(regex = r#"^dryrun test case with "([^"]+)" of type "([^"]+)"$"#)]
async fn dryrun_test_case(w: &mut World, program: String, kind: String) {
    let algod = w.algod.as_ref().expect("algod not set");
    let request = build_dryrun_test_case(&program, &kind);
    let resp = algod.teal_dryrun(Some(request)).await.expect("teal_dryrun");
    w.dryrun_response = Some(resp);
    w.dryrun_kind = Some(kind);
}

#[then(regex = r#"^status assert of "([^"]+)" is succeed$"#)]
async fn status_assert(w: &mut World, expected: String) {
    let resp = w.dryrun_response.as_ref().expect("dryrun response not set");
    let kind = w.dryrun_kind.as_deref().unwrap_or("");
    let txn = resp.txns.first().expect("no dryrun txn results");

    // For app call tests (approv/clearp), check app_call_status;
    // for logic sig tests (lsig), check logic_sig_status.
    let status = match kind {
        "approv" | "clearp" => result::app_call_status(txn).unwrap_or(""),
        _ => result::logic_sig_status(txn).unwrap_or(""),
    };
    assert_eq!(status, expected, "dryrun status mismatch");
}

#[then(regex = r#"^global delta assert with "([^"]*)", "([^"]*)" and (\d+) is succeed$"#)]
async fn global_delta_succeed(w: &mut World, key: String, value: String, action: u64) {
    let resp = w.dryrun_response.as_ref().expect("dryrun response not set");
    let txn = resp.txns.first().expect("no dryrun txn results");
    let deltas = txn.global_delta.as_deref().unwrap_or(&[]);
    let hit = deltas
        .iter()
        .find(|kv| kv.key == key)
        .expect("key not found in global delta");
    assert_eq!(hit.value.action, action, "action mismatch");
    match action {
        1 => assert_eq!(hit.value.bytes.as_deref(), Some(value.as_str())),
        2 => assert_eq!(hit.value.uint.unwrap_or(0).to_string(), value),
        _ => panic!("unknown delta action {action}"),
    }
}

#[then(regex = r#"^global delta assert with "([^"]*)", "([^"]*)" and (\d+) is failed$"#)]
async fn global_delta_failed(w: &mut World, key: String, value: String, action: u64) {
    let resp = w.dryrun_response.as_ref().expect("dryrun response not set");
    let txn = resp.txns.first().expect("no dryrun txn results");
    let deltas = txn.global_delta.as_deref().unwrap_or(&[]);
    let matches = deltas.iter().any(|kv| {
        kv.key == key
            && kv.value.action == action
            && match action {
                1 => kv.value.bytes.as_deref() == Some(value.as_str()),
                2 => kv.value.uint.unwrap_or(0).to_string() == value,
                _ => false,
            }
    });
    assert!(!matches, "expected delta NOT to match, but it did");
}

#[then(
    regex = r#"^local delta assert for "([^"]*)" of accounts (\d+) with "([^"]*)", "([^"]*)" and (\d+) is succeed$"#
)]
async fn local_delta_succeed(
    w: &mut World,
    addr: String,
    index: usize,
    key: String,
    value: String,
    action: u64,
) {
    let resp = w.dryrun_response.as_ref().expect("dryrun response not set");
    let txn = resp.txns.first().expect("no dryrun txn results");
    let deltas = txn.local_deltas.as_deref().unwrap_or(&[]);
    let account_delta = deltas
        .get(index)
        .or_else(|| deltas.iter().find(|d| d.address == addr))
        .expect("local delta entry not found");
    let kvs = &account_delta.delta;
    let hit = kvs
        .iter()
        .find(|kv| kv.key == key)
        .expect("key not in local delta");
    assert_eq!(hit.value.action, action);
    match action {
        1 => assert_eq!(hit.value.bytes.as_deref(), Some(value.as_str())),
        2 => assert_eq!(hit.value.uint.unwrap_or(0).to_string(), value),
        _ => panic!("unknown delta action {action}"),
    }
    // Address is unused except for fallback lookup; mention it to
    // silence dead-code warnings when the index path succeeds.
    let _ = addr;
}
