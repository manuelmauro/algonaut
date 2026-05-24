//! Minimal JSON parsing for ARC-4 ABI contract files.
//!
//! We define our own minimal structs here rather than depending on algonaut_abi
//! to avoid circular dependencies (algonaut_abi re-exports macros from this crate).

use serde::Deserialize;
use std::collections::HashMap;

/// Top-level ARC-4 contract JSON structure.
#[derive(Debug, Deserialize)]
pub struct ContractJson {
    /// Contract name (used to derive the generated struct name).
    pub name: String,
    /// Optional contract description (parsed but not currently used).
    #[serde(default)]
    #[allow(dead_code)]
    pub desc: Option<String>,
    /// Method definitions.
    #[serde(default)]
    pub methods: Vec<MethodJson>,
    /// Network deployments mapping genesis hash to app info.
    #[serde(default)]
    pub networks: HashMap<String, NetworkInfo>,
}

/// A method definition in the ARC-4 ABI.
#[derive(Debug, Deserialize)]
pub struct MethodJson {
    /// Method name.
    pub name: String,
    /// Optional method description.
    #[serde(default)]
    pub desc: Option<String>,
    /// Method arguments.
    #[serde(default)]
    pub args: Vec<ArgJson>,
    /// Return type (ABI type string).
    #[serde(default)]
    pub returns: ReturnJson,
}

/// An argument definition in a method.
#[derive(Debug, Deserialize)]
pub struct ArgJson {
    /// Argument name (optional per ARC-4).
    #[serde(default)]
    pub name: Option<String>,
    /// ABI type string (e.g., "uint64", "address", "byte[]").
    #[serde(rename = "type")]
    pub type_str: String,
    /// Optional argument description (parsed but not currently used).
    #[serde(default)]
    #[allow(dead_code)]
    pub desc: Option<String>,
}

/// Return type specification.
#[derive(Debug, Deserialize, Default)]
pub struct ReturnJson {
    /// ABI type string for the return value.
    #[serde(rename = "type", default = "default_void")]
    pub type_str: String,
    /// Optional description (parsed but not currently used).
    #[serde(default)]
    #[allow(dead_code)]
    pub desc: Option<String>,
}

fn default_void() -> String {
    "void".to_owned()
}

/// Network-specific deployment info.
#[derive(Debug, Deserialize)]
pub struct NetworkInfo {
    /// Application ID on this network.
    #[serde(rename = "appID")]
    pub app_id: u64,
}

impl MethodJson {
    /// Reconstruct the ARC-4 method signature string (e.g., "add(uint64,uint64)uint64").
    pub fn signature(&self) -> String {
        let arg_types: Vec<&str> = self.args.iter().map(|a| a.type_str.as_str()).collect();
        format!(
            "{}({}){}",
            self.name,
            arg_types.join(","),
            self.returns.type_str
        )
    }
}

/// Known genesis hashes mapped to network names.
pub fn genesis_to_network(genesis_hash: &str) -> Option<&'static str> {
    match genesis_hash {
        "wGHE2Pwdvd7S12BL5FaOP20EGYesN73ktiC1qzkkit8=" => Some("testnet"),
        "SGO1GKSzyE7IEPItTxCByw9x8FmnrCDexi9/cOUJOiI=" => Some("mainnet"),
        "mFgazF+2uRS1tMiL9dsj01hJGySEmPN28B/TjjvpVW0=" => Some("betanet"),
        _ => None,
    }
}

/// Parse an ARC-4 contract JSON string.
pub fn parse_contract_json(json_str: &str) -> Result<ContractJson, String> {
    serde_json::from_str(json_str).map_err(|e| format!("failed to parse ABI JSON: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_reconstruction() {
        let method = MethodJson {
            name: "add".to_owned(),
            desc: None,
            args: vec![
                ArgJson {
                    name: Some("a".to_owned()),
                    type_str: "uint64".to_owned(),
                    desc: None,
                },
                ArgJson {
                    name: Some("b".to_owned()),
                    type_str: "uint64".to_owned(),
                    desc: None,
                },
            ],
            returns: ReturnJson {
                type_str: "uint64".to_owned(),
                desc: None,
            },
        };
        assert_eq!(method.signature(), "add(uint64,uint64)uint64");
    }

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
        assert_eq!(contract.methods[0].signature(), "add(uint64,uint64)uint64");
    }

    #[test]
    fn test_genesis_to_network() {
        assert_eq!(
            genesis_to_network("wGHE2Pwdvd7S12BL5FaOP20EGYesN73ktiC1qzkkit8="),
            Some("testnet")
        );
        assert_eq!(
            genesis_to_network("SGO1GKSzyE7IEPItTxCByw9x8FmnrCDexi9/cOUJOiI="),
            Some("mainnet")
        );
        assert_eq!(genesis_to_network("unknown-hash"), None);
    }
}
