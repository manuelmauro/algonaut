//! Pure ARC-4 ABI JSON data model.
//!
//! These structs mirror the on-the-wire ARC-4 ABI / app-spec JSON — contract,
//! interface, method, argument, return, and per-network deployment info — and
//! nothing more: no resolved [`AbiType`], no typed `AppId`, no method
//! selectors. They are the single source of truth for the JSON shape, shared
//! by two crates that must agree on it:
//!
//! - the runtime ([`algonaut_abi`]), whose richer `AbiContract`/`AbiMethod`/…
//!   add a lazily-parsed type cache and typed identifiers and (de)serialize by
//!   delegating here via `#[serde(from / into)]`;
//! - the compile-time `contract!` macro ([`algonaut_abi_macros`]), which reads
//!   these to generate a typed client.
//!
//! Keeping the model in a dependency-light leaf crate (serde-derive only, like
//! the sibling [`algonaut_abi_sig`] grammar crate) is what breaks the cycle:
//! `algonaut_abi` re-exports the macros, so `algonaut_abi_macros` cannot depend
//! on `algonaut_abi` — but both can depend on this.
//!
//! [`AbiType`]: https://docs.rs/algonaut_abi
//! [`algonaut_abi`]: https://docs.rs/algonaut_abi
//! [`algonaut_abi_macros`]: https://docs.rs/algonaut_abi_macros
//! [`algonaut_abi_sig`]: https://docs.rs/algonaut_abi_sig

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An ARC-4 contract: a concrete set of methods implemented by a single app.
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

/// A single ABI method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbiMethod {
    /// Method name.
    pub name: String,

    /// Optional human-readable description.
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,

    /// The method's arguments, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<AbiMethodArg>,

    /// The method's return type. Defaults to `void` when absent.
    #[serde(default)]
    pub returns: AbiReturn,
}

/// A single method argument.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbiMethodArg {
    /// Optional argument name (omitted by some ARC-4 producers).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// ABI type string (e.g. `"uint64"`, `"address"`, `"byte[]"`).
    #[serde(rename = "type")]
    pub type_: String,

    /// Optional human-readable description.
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
}

/// A method's return type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbiReturn {
    /// ABI type string for the return value (`"void"` for no return).
    #[serde(rename = "type")]
    pub type_: String,

    /// Optional human-readable description.
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
}

impl Default for AbiReturn {
    fn default() -> Self {
        Self {
            type_: "void".to_owned(),
            desc: None,
        }
    }
}

/// Per-network deployment info: which app ID hosts the contract on a network.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbiContractNetworkInfo {
    /// The application ID on this network.
    #[serde(rename = "appID")]
    pub app_id: u64,
}

impl AbiMethod {
    /// Reconstruct the ARC-4 method signature (e.g. `"add(uint64,uint64)uint64"`).
    pub fn get_signature(&self) -> String {
        let arg_types: Vec<&str> = self.args.iter().map(|a| a.type_.as_str()).collect();
        format!(
            "{}({}){}",
            self.name,
            arg_types.join(","),
            self.returns.type_
        )
    }
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
    fn get_signature_reconstructs_from_parts() {
        let method = AbiMethod {
            name: "add".to_owned(),
            desc: None,
            args: vec![
                AbiMethodArg {
                    name: Some("a".to_owned()),
                    type_: "uint64".to_owned(),
                    desc: None,
                },
                AbiMethodArg {
                    name: Some("b".to_owned()),
                    type_: "uint64".to_owned(),
                    desc: None,
                },
            ],
            returns: AbiReturn {
                type_: "uint64".to_owned(),
                desc: None,
            },
        };
        assert_eq!(method.get_signature(), "add(uint64,uint64)uint64");
    }

    #[test]
    fn returns_defaults_to_void() {
        assert_eq!(AbiReturn::default().type_, "void");
        // A method object without a `returns` field decodes to void.
        let json = r#"{"name":"noop","args":[]}"#;
        let method: AbiMethod = serde_json::from_str(json).unwrap();
        assert_eq!(method.get_signature(), "noop()void");
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
