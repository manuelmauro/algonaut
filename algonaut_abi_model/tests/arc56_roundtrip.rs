//! ARC-56 parsing and round-trip coverage for the shared app-spec model.
//!
//! Phase 1 of the `contract-macro-arc56-app-spec` ADR: the model must parse the
//! full ARC-56 schema and preserve it across a serialize/parse round-trip. The
//! ARC-4 byte-identity guarantee is covered by the unit tests in the crate and
//! by `algonaut_abi`'s `abi_json_tests`.

use algonaut_abi_model::{AbiContract, StructFieldType};

const FULL_ARC56: &str = include_str!("fixtures/full.arc56.json");

#[test]
fn parses_full_arc56_spec() {
    let c: AbiContract = serde_json::from_str(FULL_ARC56).expect("fixture should parse");

    assert_eq!(c.name, "Vault");
    assert_eq!(c.arcs, vec![22, 28, 56]);
    assert_eq!(c.networks.len(), 2);

    // Named structs, including an inline nested one.
    assert!(c.structs.contains_key("Pair"));
    let outer = &c.structs["Outer"][0];
    assert!(matches!(outer.type_, StructFieldType::Nested(_)));

    // Method with a struct arg, a struct return, and a literal default value.
    let store = c.methods.iter().find(|m| m.name == "store").unwrap();
    assert_eq!(store.args[0].struct_.as_deref(), Some("Pair"));
    assert_eq!(store.returns.struct_.as_deref(), Some("Pair"));
    assert_eq!(store.readonly, Some(false));
    let default = store.args[1].default_value.as_ref().unwrap();
    assert_eq!(default.source, "literal");
    let rec = store.recommendations.as_ref().unwrap();
    assert_eq!(rec.inner_transaction_count, Some(1));
    assert_eq!(rec.boxes.as_ref().unwrap().read_bytes, 100);

    // Read-only method.
    let get_total = c.methods.iter().find(|m| m.name == "get_total").unwrap();
    assert_eq!(get_total.readonly, Some(true));
    // The struct overlay does not change the canonical signature.
    assert_eq!(get_total.get_signature(), "get_total()uint64");

    // Declared state.
    let state = c.state.as_ref().unwrap();
    assert_eq!(state.schema.global.ints, 1);
    assert!(state.keys.global.contains_key("total"));
    assert_eq!(state.maps.box_["balances"].key_type, "address");

    // Bare actions, events, and deploy metadata.
    assert_eq!(c.bare_actions.as_ref().unwrap().create, vec!["NoOp"]);
    assert_eq!(c.events[0].name, "Stored");
    assert_eq!(c.compiler_info.as_ref().unwrap().compiler, "puya");
    assert!(c.source.is_some());
    assert!(c.byte_code.is_some());
    assert!(c.source_info.is_some());
    assert!(c.template_variables.contains_key("TMPL_OWNER"));
    assert_eq!(c.scratch_variables["counter"].slot, 0);
}

#[test]
fn full_arc56_round_trips_by_value() {
    let parsed: AbiContract = serde_json::from_str(FULL_ARC56).unwrap();
    let serialized = serde_json::to_string(&parsed).unwrap();
    let reparsed: AbiContract = serde_json::from_str(&serialized).unwrap();
    assert_eq!(parsed, reparsed);
}
