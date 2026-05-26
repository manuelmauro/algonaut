//! The top-level contract and interface types, plus per-network deployment
//! info and the genesis-hash-to-network mapping.

use crate::{
    AbiMethod, Actions, CompilerInfo, ContractState, Event, ProgramPair, ProgramSourceInfoPair,
    ScratchVariable, StructField, TemplateVariable,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An ARC-4 / ARC-56 contract: a concrete set of methods implemented by a
/// single app.
///
/// The first four fields are the ARC-4 core; the rest are ARC-56 extensions,
/// each optional so an ARC-4 document round-trips unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbiContract {
    /// Contract name (the generated client struct is named after it).
    pub name: String,

    /// Optional human-readable description.
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,

    /// Per-network deployment info, keyed by genesis hash.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub networks: HashMap<String, AbiContractNetworkInfo>,

    /// The contract's methods.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<AbiMethod>,

    // --- ARC-56 extensions ------------------------------------------------
    /// ARC numbers this contract conforms to (ARC-56 always implies 4 and 56).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arcs: Vec<u64>,

    /// Named struct (named-tuple) definitions, keyed by struct name.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub structs: HashMap<String, Vec<StructField>>,

    /// Declared global/local/box state: schema, fixed keys, and dynamic maps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<ContractState>,

    /// Create/call actions supported by bare (non-ABI) calls.
    #[serde(
        rename = "bareActions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub bare_actions: Option<Actions>,

    /// Source-map / PC-offset info for the approval and clear programs.
    #[serde(
        rename = "sourceInfo",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub source_info: Option<ProgramSourceInfoPair>,

    /// Pre-compiled TEAL for the approval and clear programs (base64).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<ProgramPair>,

    /// Compiled bytecode for the approval and clear programs (base64).
    #[serde(rename = "byteCode", default, skip_serializing_if = "Option::is_none")]
    pub byte_code: Option<ProgramPair>,

    /// Which compiler produced the contract, and its version.
    #[serde(
        rename = "compilerInfo",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub compiler_info: Option<CompilerInfo>,

    /// ARC-28 events emitted by the contract.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Event>,

    /// TEAL template variables and their (optional) values.
    #[serde(
        rename = "templateVariables",
        default,
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub template_variables: HashMap<String, TemplateVariable>,

    /// Scratch-slot assignments used by the contract.
    #[serde(
        rename = "scratchVariables",
        default,
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub scratch_variables: HashMap<String, ScratchVariable>,
}

/// An ARC-4 interface: a logical grouping of methods, with no deployment info.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbiInterface {
    /// Interface name.
    pub name: String,

    /// Optional human-readable description.
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,

    /// The interface's methods.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<AbiMethod>,
}

/// Per-network deployment info: which app ID hosts the contract on a network.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbiContractNetworkInfo {
    /// The application ID on this network.
    #[serde(rename = "appID")]
    pub app_id: u64,
}

/// Map a known mainnet/testnet/betanet genesis hash to its network name.
///
/// Returns `None` for any other (or custom) network, leaving the caller to
/// decide how to name it.
pub fn genesis_to_network(genesis_hash: &str) -> Option<&'static str> {
    match genesis_hash {
        "wGHE2Pwdvd7S12BL5FaOP20EGYesN73ktiC1qzkkit8=" => Some("testnet"),
        "SGO1GKSzyE7IEPItTxCByw9x8FmnrCDexi9/cOUJOiI=" => Some("mainnet"),
        "mFgazF+2uRS1tMiL9dsj01hJGySEmPN28B/TjjvpVW0=" => Some("betanet"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arc4_contract_still_parses_byte_identically() {
        // An ARC-4 document (no ARC-56 fields) parses with all extensions
        // defaulted to empty, and re-serializes byte-identically because the
        // extensions are skipped when empty.
        let json = r#"{"name":"Calculator","methods":[{"name":"add","args":[{"name":"a","type":"uint64"}],"returns":{"type":"uint64"}}]}"#;
        let contract: AbiContract = serde_json::from_str(json).unwrap();
        assert_eq!(contract.name, "Calculator");
        assert!(contract.arcs.is_empty());
        assert!(contract.structs.is_empty());
        assert!(contract.state.is_none());
        assert_eq!(contract.methods[0].get_signature(), "add(uint64)uint64");
        assert_eq!(serde_json::to_string(&contract).unwrap(), json);
    }

    #[test]
    fn genesis_to_network_maps_known_hashes() {
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
