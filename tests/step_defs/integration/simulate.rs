use crate::step_defs::integration::world::World;
use algonaut::simulate::SimulateRequestBuilder;
use algonaut_algod::models::{
    SimulateRequest, SimulateRequestTransactionGroup, SimulateTransactionGroupResult,
};
use cucumber::{given, then, when};
use data_encoding::BASE64;

#[when(expr = "I simulate the transaction")]
async fn i_simulate_the_transaction(w: &mut World) {
    let algod = w.algod.as_ref().expect("algod not set");

    let st = if w.simulate_unsigned {
        // Wire form for an unsigned txn: a `SignedTransaction` carrying
        // the all-zero placeholder signature; algod's simulator reports
        // this as "signedtxn has no sig".
        let tx = w.tx.clone().expect("tx not set");
        algonaut_transaction::SignedTransaction {
            transaction_id: tx.id().unwrap(),
            transaction: tx,
            sig: algonaut_transaction::transaction::TransactionSignature::Single(
                algonaut_crypto::Signature([0; 64]),
            ),
            auth_address: None,
        }
    } else {
        w.signed_tx.clone().expect("signed tx not set")
    };

    let req = SimulateRequest::new(vec![SimulateRequestTransactionGroup::new(vec![st])]);
    let resp = algod.simulate_txns(req).await.expect("simulate failed");
    w.simulate_response = Some(resp);
    w.simulate_unsigned = false;
}

#[when(expr = "I prepare the transaction without signatures for simulation")]
async fn i_prepare_the_transaction_without_signatures_for_simulation(w: &mut World) {
    w.simulate_unsigned = true;
}

#[then(expr = "the simulation should succeed without any failure message")]
#[when(expr = "the simulation should succeed without any failure message")]
async fn the_simulation_should_succeed_without_any_failure_message(w: &mut World) {
    let resp = w.simulate_response.as_ref().expect("no simulate response");
    for (i, g) in resp.txn_groups.iter().enumerate() {
        if let Some(msg) = &g.failure_message {
            if !msg.is_empty() {
                panic!("group {i} failed: {msg}");
            }
        }
    }
}

#[then(
    regex = r#"^the simulation should report a failure at group "(\d+)", path "([^"]+)" with message "([^"]+)"$"#
)]
async fn the_simulation_should_report_a_failure_at_group(
    w: &mut World,
    group_idx: u64,
    path_str: String,
    expected_message: String,
) {
    let resp = w.simulate_response.as_ref().expect("no simulate response");
    let g: &SimulateTransactionGroupResult = resp
        .txn_groups
        .get(group_idx as usize)
        .expect("group index out of range");

    let msg = g
        .failure_message
        .as_deref()
        .expect("expected a failure message but the group did not fail");
    assert!(
        msg.contains(&expected_message),
        "expected failure containing {expected_message:?}, got {msg:?}"
    );

    let expected_path: Vec<u64> = path_str
        .split(',')
        .map(|p| p.trim().parse().expect("path component must be a number"))
        .collect();
    let actual_path = g
        .failed_at
        .as_ref()
        .expect("expected failed-at path but it was missing");
    assert_eq!(
        actual_path, &expected_path,
        "failed-at path mismatch: expected {expected_path:?}, got {actual_path:?}"
    );
}

#[then(expr = "I simulate the current transaction group with the composer")]
#[when(expr = "I simulate the current transaction group with the composer")]
async fn i_simulate_the_current_transaction_group_with_the_composer(w: &mut World) {
    let algod = w.algod.as_ref().expect("algod not set").clone();
    let composer = w.tx_composer.as_mut().expect("composer not set");
    let result = composer
        .simulate(&algod)
        .await
        .expect("composer simulate failed");
    w.simulate_response = Some(result.simulate_response);
}

#[when(expr = "I make a new simulate request.")]
#[given(expr = "I make a new simulate request.")]
async fn i_make_a_new_simulate_request(w: &mut World) {
    w.simulate_request = Some(SimulateRequest::new(vec![]));
}

#[then(expr = "I allow more logs on that simulate request.")]
#[when(expr = "I allow more logs on that simulate request.")]
async fn i_allow_more_logs_on_that_simulate_request(w: &mut World) {
    let req = w
        .simulate_request
        .as_mut()
        .expect("no simulate request — call `I make a new simulate request.` first");
    req.allow_more_logging = Some(true);
}

#[then(regex = r#"^I allow (\d+) more budget on that simulate request\.$"#)]
#[when(regex = r#"^I allow (\d+) more budget on that simulate request\.$"#)]
async fn i_allow_more_budget_on_that_simulate_request(w: &mut World, extra: u64) {
    let req = w
        .simulate_request
        .as_mut()
        .expect("no simulate request — call `I make a new simulate request.` first");
    req.extra_opcode_budget = Some(extra);
}

#[then(regex = r#"^I allow exec trace options "([^"]+)" on that simulate request\.$"#)]
#[when(regex = r#"^I allow exec trace options "([^"]+)" on that simulate request\.$"#)]
async fn i_allow_exec_trace_options_on_that_simulate_request(w: &mut World, opts: String) {
    let req = w
        .simulate_request
        .as_mut()
        .expect("no simulate request — call `I make a new simulate request.` first");

    let mut builder = SimulateRequestBuilder::new(vec![]);
    if req.allow_more_logging.unwrap_or(false) {
        builder = builder.allow_more_logging(true);
    }
    if let Some(b) = req.extra_opcode_budget {
        builder = builder.extra_opcode_budget(b);
    }
    builder = builder.with_exec_trace(|mut t| {
        for opt in opts.split(',').map(|s| s.trim()) {
            t = match opt {
                "stack" => t.stack(),
                "scratch" => t.scratch(),
                "state" => t.state(),
                other => panic!("unknown exec trace option: {other}"),
            };
        }
        t
    });
    *req = builder.build();
}

#[then(expr = "I simulate the transaction group with the simulate request.")]
#[when(expr = "I simulate the transaction group with the simulate request.")]
async fn i_simulate_the_transaction_group_with_the_simulate_request(w: &mut World) {
    let algod = w.algod.as_ref().expect("algod not set").clone();
    let req = w
        .simulate_request
        .clone()
        .expect("no simulate request — call `I make a new simulate request.` first");
    let composer = w.tx_composer.as_mut().expect("composer not set");
    let result = composer
        .simulate_with(&algod, req)
        .await
        .expect("composer simulate_with failed");
    w.simulate_response = Some(result.simulate_response);
}

#[then(expr = "I check the simulation result has power packs allow-more-logging.")]
async fn i_check_the_simulation_result_has_power_packs_allow_more_logging(w: &mut World) {
    let resp = w.simulate_response.as_ref().expect("no simulate response");
    let overrides = resp
        .eval_overrides
        .as_deref()
        .expect("expected eval-overrides but got none");
    // The API doesn't echo `allow-more-logging` directly — it echoes
    // the bumped log-call and log-size ceilings instead.
    assert!(
        overrides.max_log_calls.is_some() || overrides.max_log_size.is_some(),
        "expected lifted log limits in eval-overrides, got {overrides:?}"
    );
}

#[then(
    regex = r#"^I check the simulation result has power packs extra-opcode-budget with extra budget (\d+)\.$"#
)]
async fn i_check_the_simulation_result_has_power_packs_extra_opcode_budget(
    w: &mut World,
    expected: u64,
) {
    let resp = w.simulate_response.as_ref().expect("no simulate response");
    let overrides = resp
        .eval_overrides
        .as_deref()
        .expect("expected eval-overrides but got none");
    assert_eq!(overrides.extra_opcode_budget, Some(expected));
}

#[then(
    regex = r#"^(\d+)th unit in the "([^"]+)" trace at txn-groups path "([^"]+)" should add value "([^"]*)" to stack, pop (\d+) values from stack, write value "([^"]*)" to scratch slot "([^"]*)"\.$"#
)]
async fn nth_unit_in_trace_stack_scratch(
    w: &mut World,
    n: usize,
    trace_kind: String,
    path_str: String,
    add_stack: String,
    pop_count: u64,
    scratch_value: String,
    scratch_slot: String,
) {
    let unit = trace_unit_at(w, n, &trace_kind, &path_str);

    let actual_pop = unit.stack_pop_count.unwrap_or(0);
    assert_eq!(
        actual_pop, pop_count,
        "stack-pop-count mismatch at unit {n}: expected {pop_count}, got {actual_pop}"
    );

    let expected_additions = parse_avm_values_csv(&add_stack);
    let actual_additions: Vec<(u64, String)> = unit
        .stack_additions
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|v| {
            (
                v.r#type,
                if v.r#type == 2 {
                    v.uint.map(|u| u.to_string()).unwrap_or_default()
                } else {
                    v.bytes.clone().unwrap_or_default()
                },
            )
        })
        .collect();
    assert_eq!(
        actual_additions, expected_additions,
        "stack-additions mismatch at unit {n}"
    );

    let expected_scratch = if scratch_value.is_empty() {
        None
    } else {
        Some((
            scratch_slot
                .parse::<u64>()
                .expect("scratch slot must parse"),
            parse_avm_value(&scratch_value),
        ))
    };
    let actual_scratch = unit
        .scratch_changes
        .as_deref()
        .unwrap_or(&[])
        .first()
        .map(|c| {
            (
                c.slot,
                (
                    c.new_value.r#type,
                    if c.new_value.r#type == 2 {
                        c.new_value.uint.map(|u| u.to_string()).unwrap_or_default()
                    } else {
                        c.new_value.bytes.clone().unwrap_or_default()
                    },
                ),
            )
        });
    assert_eq!(
        actual_scratch, expected_scratch,
        "scratch-changes mismatch at unit {n}"
    );
}

#[then(regex = r#"^"([^"]+)" hash at txn-groups path "([^"]+)" should be "([^"]+)"\.$"#)]
async fn hash_at_txn_groups_path_should_be(
    w: &mut World,
    program_kind: String,
    path_str: String,
    expected_hash: String,
) {
    let resp = w.simulate_response.as_ref().expect("no simulate response");
    let group = resp.txn_groups.first().expect("no group");
    let path: Vec<u64> = path_str
        .split(',')
        .map(|p| p.trim().parse().unwrap())
        .collect();
    let txn_result = group
        .txn_results
        .get(path[0] as usize)
        .expect("path index out of range");
    let trace = txn_result
        .exec_trace
        .as_deref()
        .expect("no exec-trace on txn-result");

    let actual = match program_kind.as_str() {
        "approval" => trace.approval_program_hash.clone(),
        "clear-state" => trace.clear_state_program_hash.clone(),
        "logic-sig" => trace.logic_sig_hash.clone(),
        other => panic!("unknown program kind: {other}"),
    };
    let actual = actual.expect("expected program hash on exec-trace but got none");
    assert_eq!(actual, expected_hash);
}

#[then(
    regex = r#"^(\d+)th unit in the "([^"]+)" trace at txn-groups path "([^"]+)" should write to "([^"]+)" state "([^"]+)" with new value "([^"]+)"\.$"#
)]
async fn nth_unit_in_trace_state_write(
    w: &mut World,
    n: usize,
    trace_kind: String,
    path_str: String,
    state_kind: String,
    expected_key: String,
    expected_new_value: String,
) {
    let unit = trace_unit_at(w, n, &trace_kind, &path_str);
    let changes = unit
        .state_changes
        .as_deref()
        .expect("expected state-changes on this trace unit");

    let want_type = match state_kind.as_str() {
        "global" => "g",
        "local" => "l",
        "box" => "b",
        other => panic!("unknown state kind: {other}"),
    };
    let expected_key_b64 = BASE64.encode(expected_key.as_bytes());
    let matching = changes
        .iter()
        .find(|c| c.app_state_type == want_type && c.operation == "w" && c.key == expected_key_b64);
    let op = matching.expect("no matching write in this unit");
    let new_val = op
        .new_value
        .as_deref()
        .expect("write operation must have a new-value");
    let actual = (
        new_val.r#type,
        if new_val.r#type == 2 {
            new_val.uint.map(|u| u.to_string()).unwrap_or_default()
        } else {
            new_val.bytes.clone().unwrap_or_default()
        },
    );
    assert_eq!(actual, parse_avm_value(&expected_new_value));
}

#[then(regex = r#"^the current application initial "([^"]+)" state should be empty\.$"#)]
async fn the_current_application_initial_state_should_be_empty(w: &mut World, state_kind: String) {
    let resp = w.simulate_response.as_ref().expect("no simulate response");
    let initial = resp
        .initial_states
        .as_ref()
        .expect("expected initial-states on response");

    let app_id = w.app_id.expect("no current app id");
    let bucket = locate_app_initial_state(initial, app_id, &state_kind);
    let empty = match &bucket {
        None => true,
        Some(serde_json::Value::Array(a)) => a.is_empty(),
        Some(serde_json::Value::Object(o)) => o.is_empty(),
        Some(serde_json::Value::Null) => true,
        _ => false,
    };
    assert!(
        empty,
        "initial {state_kind} state was not empty: {bucket:?}"
    );
}

#[then(
    regex = r#"^the current application initial "([^"]+)" state should contain "([^"]+)" with value "([^"]+)"\.$"#
)]
async fn the_current_application_initial_state_should_contain(
    w: &mut World,
    state_kind: String,
    expected_key: String,
    expected_value: String,
) {
    let resp = w.simulate_response.as_ref().expect("no simulate response");
    let initial = resp
        .initial_states
        .as_ref()
        .expect("expected initial-states on response");

    let app_id = w.app_id.expect("no current app id");
    let bucket = locate_app_initial_state(initial, app_id, &state_kind)
        .expect("no initial state bucket found");
    let entries = bucket
        .as_array()
        .expect("initial state bucket should be an array");

    let expected_key_b64 = BASE64.encode(expected_key.as_bytes());
    let found = entries.iter().any(|e| {
        let key = e.get("key").and_then(|k| k.as_str()).unwrap_or_default();
        if key != expected_key_b64 {
            return false;
        }
        let val = e.get("value").unwrap_or(&serde_json::Value::Null);
        avm_value_matches(val, &expected_value)
    });
    assert!(
        found,
        "initial {state_kind} state did not contain {expected_key}={expected_value}"
    );
}

fn trace_unit_at<'a>(
    w: &'a World,
    n: usize,
    trace_kind: &str,
    path_str: &str,
) -> &'a algonaut_algod::models::SimulationOpcodeTraceUnit {
    let resp = w.simulate_response.as_ref().expect("no simulate response");
    let group = resp.txn_groups.first().expect("no group");
    let path: Vec<u64> = path_str
        .split(',')
        .map(|p| p.trim().parse().unwrap())
        .collect();
    let txn_result = group
        .txn_results
        .get(path[0] as usize)
        .expect("path index out of range");
    let trace = txn_result
        .exec_trace
        .as_deref()
        .expect("no exec-trace on txn-result");
    let units: &[algonaut_algod::models::SimulationOpcodeTraceUnit] = match trace_kind {
        "approval" => trace.approval_program_trace.as_deref().unwrap_or(&[]),
        "clear-state" => trace.clear_state_program_trace.as_deref().unwrap_or(&[]),
        "logic-sig" => trace.logic_sig_trace.as_deref().unwrap_or(&[]),
        other => panic!("unknown trace kind: {other}"),
    };
    units.get(n).expect("trace unit index out of range")
}

fn parse_avm_values_csv(s: &str) -> Vec<(u64, String)> {
    if s.is_empty() {
        return Vec::new();
    }
    s.split(',').map(|v| parse_avm_value(v.trim())).collect()
}

/// Parses cucumber's `type:value` shorthand into a `(type, value)` tuple
/// that mirrors `AvmValue` (`type` is 1 for bytes, 2 for uint64).
fn parse_avm_value(s: &str) -> (u64, String) {
    if let Some(rest) = s.strip_prefix("uint64:") {
        (2, rest.to_string())
    } else if let Some(rest) = s.strip_prefix("bytes:") {
        (1, rest.to_string())
    } else {
        panic!("unrecognised AVM value literal: {s}");
    }
}

fn avm_value_matches(json: &serde_json::Value, literal: &str) -> bool {
    let (want_type, want_value) = parse_avm_value(literal);
    let Some(ty) = json.get("type").and_then(|t| t.as_u64()) else {
        return false;
    };
    if ty != want_type {
        return false;
    }
    if want_type == 2 {
        json.get("uint")
            .and_then(|u| u.as_u64())
            .map(|u| u.to_string() == want_value)
            .unwrap_or(false)
    } else {
        json.get("bytes")
            .and_then(|b| b.as_str())
            .map(|b| b == want_value)
            .unwrap_or(false)
    }
}

/// Walk the free-form `initial-states` JSON looking for the requested
/// state bucket on the current app. The shape is:
///
/// ```json
/// {
///   "app-initial-states": [
///     { "id": <appId>, "app-globals": [...], "app-locals": [...], "app-boxes": [...] }
///   ]
/// }
/// ```
fn locate_app_initial_state<'a>(
    initial: &'a serde_json::Value,
    app_id: u64,
    state_kind: &str,
) -> Option<&'a serde_json::Value> {
    let key = match state_kind {
        "global" => "app-globals",
        "local" => "app-locals",
        "box" => "app-boxes",
        other => panic!("unknown state kind: {other}"),
    };
    let apps = initial.get("app-initial-states")?.as_array()?;
    let app = apps
        .iter()
        .find(|a| a.get("id").and_then(|i| i.as_u64()) == Some(app_id))?;
    app.get(key)
}
