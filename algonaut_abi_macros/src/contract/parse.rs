//! Reads ARC-4 ABI contract JSON into the shared [`algonaut_abi_model`] types.
//!
//! The data model itself lives in [`algonaut_abi_model`] — a dependency-light
//! leaf crate shared with the runtime `algonaut_abi` — so the macro and the
//! runtime agree on the JSON shape by construction, rather than by keeping
//! parallel structs in sync. This module only adds the compile-time
//! file-parsing entry point.

pub use algonaut_abi_model::{AbiContract, AbiMethod, genesis_to_network};

/// Parse an ARC-4 contract JSON string into the shared model.
pub fn parse_contract_json(json_str: &str) -> Result<AbiContract, String> {
    serde_json::from_str(json_str).map_err(|e| format!("failed to parse ABI JSON: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_contract() {
        let json = r#"{
            "name": "Calculator",
            "methods": [
                {
                    "name": "add",
                    "args": [
                        {"name": "a", "type": "uint64"},
                        {"name": "b", "type": "uint64"}
                    ],
                    "returns": {"type": "uint64"}
                }
            ]
        }"#;
        let contract = parse_contract_json(json).unwrap();
        assert_eq!(contract.name, "Calculator");
        assert_eq!(contract.methods.len(), 1);
        assert_eq!(
            contract.methods[0].get_signature(),
            "add(uint64,uint64)uint64"
        );
    }

    #[test]
    fn test_parse_reports_error() {
        let err = parse_contract_json("{ not json").unwrap_err();
        assert!(err.contains("failed to parse ABI JSON"));
    }
}
