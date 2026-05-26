//! ABI methods and their arguments, returns, default values, and resource
//! recommendations.

use crate::{Actions, Event};
use serde::{Deserialize, Serialize};

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

    // --- ARC-56 extensions ------------------------------------------------
    /// OnComplete actions valid when invoking this method.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actions: Option<Actions>,

    /// Whether the method writes no state (ARC-22); safe to call via simulate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub readonly: Option<bool>,

    /// ARC-28 events this method may emit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Event>,

    /// Hints for clients on resources this method needs (fees, refs, boxes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommendations: Option<Recommendations>,
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

/// A single method argument.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbiMethodArg {
    /// Optional argument name (omitted by some ARC-4 producers).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// ABI type string (e.g. `"uint64"`, `"address"`, `"byte[]"`). For a
    /// struct argument this is still the canonical ABI tuple type; `struct`
    /// names it.
    #[serde(rename = "type")]
    pub type_: String,

    /// Optional human-readable description.
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,

    // --- ARC-56 extensions ------------------------------------------------
    /// Name of the `AbiContract::structs` entry that names this argument's
    /// tuple type, if any.
    #[serde(rename = "struct", default, skip_serializing_if = "Option::is_none")]
    pub struct_: Option<String>,

    /// A default value the caller may omit, sourced from storage, a literal,
    /// or another method.
    #[serde(
        rename = "defaultValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub default_value: Option<DefaultValue>,
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

    // --- ARC-56 extensions ------------------------------------------------
    /// Name of the `AbiContract::structs` entry that names the return's
    /// tuple type, if any.
    #[serde(rename = "struct", default, skip_serializing_if = "Option::is_none")]
    pub struct_: Option<String>,
}

impl Default for AbiReturn {
    fn default() -> Self {
        Self {
            type_: "void".to_owned(),
            desc: None,
            struct_: None,
        }
    }
}

/// A default value for an argument the caller may omit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefaultValue {
    /// Where the value comes from: `"box"`, `"global"`, `"local"`,
    /// `"literal"`, or `"method"`.
    pub source: String,

    /// The data: base64 bytes for storage/literal sources, or a method
    /// signature for the `"method"` source.
    pub data: String,

    /// Encoding of the data (ABI type or AVM type), when applicable.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
}

/// Client hints about the resources a method needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recommendations {
    /// Number of inner transactions the method issues.
    #[serde(
        rename = "innerTransactionCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub inner_transaction_count: Option<u64>,

    /// A box the method reads or writes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boxes: Option<BoxRecommendation>,

    /// Foreign accounts the method references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accounts: Vec<String>,

    /// Foreign apps the method references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub apps: Vec<u64>,

    /// Foreign assets the method references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assets: Vec<u64>,
}

/// A box-storage recommendation for a method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoxRecommendation {
    /// App ID owning the box, if not the called app.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app: Option<u64>,

    /// Box key, base64-encoded.
    pub key: String,

    /// Bytes read from the box.
    #[serde(rename = "readBytes")]
    pub read_bytes: u64,

    /// Bytes written to the box.
    #[serde(rename = "writeBytes")]
    pub write_bytes: u64,
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
                    struct_: None,
                    default_value: None,
                },
                AbiMethodArg {
                    name: Some("b".to_owned()),
                    type_: "uint64".to_owned(),
                    desc: None,
                    struct_: None,
                    default_value: None,
                },
            ],
            returns: AbiReturn {
                type_: "uint64".to_owned(),
                desc: None,
                struct_: None,
            },
            actions: None,
            readonly: None,
            events: Vec::new(),
            recommendations: None,
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
}
