//! Validate the ARC-56 fixtures against the official ARC-56 JSON Schema.
//!
//! `fixtures/arc56.schema.json` (draft-07) is vendored from
//! algorandfoundation/algokit-client-generator-ts. It is self-contained (no
//! remote `$ref`s), so validation runs fully offline. This guards that every
//! spec the model is tested against is a genuinely valid ARC-56 document — and
//! that any future fixture stays conformant.

use serde_json::Value;

const SCHEMA: &str = include_str!("fixtures/arc56.schema.json");

#[track_caller]
fn assert_valid_arc56(label: &str, fixture: &str) {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema is valid JSON");
    let instance: Value = serde_json::from_str(fixture).expect("fixture is valid JSON");
    let validator = jsonschema::validator_for(&schema).expect("ARC-56 schema compiles");

    let errors: Vec<String> = validator
        .iter_errors(&instance)
        .map(|error| format!("  - {error} (at `{}`)", error.instance_path()))
        .collect();
    assert!(
        errors.is_empty(),
        "{label} is not a valid ARC-56 document:\n{}",
        errors.join("\n")
    );
}

#[test]
fn full_fixture_is_valid_arc56() {
    assert_valid_arc56("full.arc56.json", include_str!("fixtures/full.arc56.json"));
}

#[test]
fn structs_fixture_is_valid_arc56() {
    assert_valid_arc56(
        "structs.arc56.json",
        include_str!("fixtures/structs.arc56.json"),
    );
}

#[test]
fn reti_fixture_is_valid_arc56() {
    assert_valid_arc56("reti.arc56.json", include_str!("fixtures/reti.arc56.json"));
}

#[test]
fn nfd_fixture_is_valid_arc56() {
    assert_valid_arc56("nfd.arc56.json", include_str!("fixtures/nfd.arc56.json"));
}

#[test]
fn zero_coupon_bond_fixture_is_valid_arc56() {
    assert_valid_arc56(
        "zero_coupon_bond.arc56.json",
        include_str!("fixtures/zero_coupon_bond.arc56.json"),
    );
}

#[test]
fn rejects_non_arc56_document() {
    // An empty object is missing every required top-level key, so it must be
    // rejected — proof the validator has teeth and isn't passing everything.
    let schema: Value = serde_json::from_str(SCHEMA).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(!validator.is_valid(&serde_json::json!({})));
}
