//! Pure ARC-56 / ARC-4 app-spec JSON data model.
//!
//! These structs mirror the on-the-wire app-spec JSON and nothing more: no
//! resolved [`AbiType`], no typed `AppId`, no method selectors. They are the
//! single source of truth for the JSON shape, shared by two crates that must
//! agree on it:
//!
//! - the runtime ([`algonaut_abi`]), whose richer `AbiContract`/`AbiMethod`/…
//!   add a lazily-parsed type cache and typed identifiers and (de)serialize by
//!   delegating here via `#[serde(from / into)]`;
//! - the compile-time `contract!` macro ([`algonaut_abi_macros`]), which reads
//!   these to generate a typed client.
//!
//! # ARC-4 and ARC-56
//!
//! ARC-56 ("Extended App Description") is a strict superset of the ARC-4
//! contract description: same `name`, `methods`, and `networks`, plus named
//! `structs`, declared `state`, ARC-28 `events`, per-method `actions` /
//! `readonly`, per-argument `struct` / `defaultValue`, and deploy metadata
//! (`source` / `byteCode` / `compilerInfo`). Every ARC-56-only field here is
//! optional with a serde default, so an ARC-4 document parses unchanged and
//! re-serializes byte-identically (the new fields are skipped when empty). The
//! method-argument `type` stays the canonical ARC-4 ABI string even in ARC-56;
//! `struct` is only a naming overlay, so [`AbiMethod::get_signature`] is
//! unaffected.
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
    /// Name of the [`AbiContract::structs`] entry that names this argument's
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
    /// Name of the [`AbiContract::structs`] entry that names the return's
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

/// Per-network deployment info: which app ID hosts the contract on a network.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbiContractNetworkInfo {
    /// The application ID on this network.
    #[serde(rename = "appID")]
    pub app_id: u64,
}

// ---------------------------------------------------------------------------
// ARC-56 supporting types
// ---------------------------------------------------------------------------

/// One field of a named struct. `type` is either an ABI type / struct name
/// (a string) or an inline nested struct (an array of fields).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructField {
    /// Field name.
    pub name: String,

    /// Field type: an ABI type / struct-name string, or a nested field list.
    #[serde(rename = "type")]
    pub type_: StructFieldType,
}

/// A [`StructField`]'s type: either a leaf type string or an inline struct.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StructFieldType {
    /// An ABI type string or the name of another struct.
    Type(String),

    /// An inline, anonymous nested struct.
    Nested(Vec<StructField>),
}

/// Declared application state: creation schema, fixed keys, and dynamic maps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractState {
    /// Global/local allocation requested at creation.
    pub schema: StateSchema,

    /// Fixed, named storage locations.
    #[serde(default)]
    pub keys: StateKeys,

    /// Dynamic, prefixed key/value collections.
    #[serde(default)]
    pub maps: StateMaps,
}

/// Global and local creation schema (counts of int and byte-slice slots).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateSchema {
    /// Global state allocation.
    pub global: SchemaCounts,

    /// Local (per-account) state allocation.
    pub local: SchemaCounts,
}

/// Number of integer and byte-slice slots in a state schema.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaCounts {
    /// Number of `uint64` slots.
    pub ints: u64,

    /// Number of byte-slice slots.
    pub bytes: u64,
}

/// Fixed, named storage keys grouped by storage class.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateKeys {
    /// Keys in global state.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub global: HashMap<String, StorageKey>,

    /// Keys in local (per-account) state.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub local: HashMap<String, StorageKey>,

    /// Keys in box storage.
    #[serde(rename = "box", default, skip_serializing_if = "HashMap::is_empty")]
    pub box_: HashMap<String, StorageKey>,
}

/// Dynamic, prefixed storage maps grouped by storage class.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateMaps {
    /// Maps in global state.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub global: HashMap<String, StorageMap>,

    /// Maps in local (per-account) state.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub local: HashMap<String, StorageMap>,

    /// Maps in box storage.
    #[serde(rename = "box", default, skip_serializing_if = "HashMap::is_empty")]
    pub box_: HashMap<String, StorageMap>,
}

/// A single fixed storage key and the encodings of its key and value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageKey {
    /// Optional human-readable description.
    #[serde(rename = "desc", default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,

    /// Encoding of the key (ABI type, AVM type, or struct name).
    #[serde(rename = "keyType")]
    pub key_type: String,

    /// Encoding of the value (ABI type, AVM type, or struct name).
    #[serde(rename = "valueType")]
    pub value_type: String,

    /// The literal key bytes, base64-encoded.
    pub key: String,
}

/// A dynamic storage map: a key/value encoding plus an optional key prefix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageMap {
    /// Optional human-readable description.
    #[serde(rename = "desc", default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,

    /// Encoding of map keys (ABI type, AVM type, or struct name).
    #[serde(rename = "keyType")]
    pub key_type: String,

    /// Encoding of map values (ABI type, AVM type, or struct name).
    #[serde(rename = "valueType")]
    pub value_type: String,

    /// Optional key prefix, base64-encoded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
}

/// A set of OnComplete actions, split into those valid on create vs. on call.
///
/// Action strings are the AVM OnComplete names (`"NoOp"`, `"OptIn"`,
/// `"CloseOut"`, `"UpdateApplication"`, `"DeleteApplication"`); kept as strings
/// so the model stays forward-compatible and dependency-free.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actions {
    /// Actions valid when creating the app.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub create: Vec<String>,

    /// Actions valid when calling an existing app.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub call: Vec<String>,
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

/// An ARC-28 event definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    /// Event name (its signature seeds the 4-byte selector).
    pub name: String,

    /// Optional human-readable description.
    #[serde(rename = "desc", default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,

    /// The event's fields, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<EventArg>,
}

/// A single field of an [`Event`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventArg {
    /// ABI type string for the field.
    #[serde(rename = "type")]
    pub type_: String,

    /// Optional field name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Optional human-readable description.
    #[serde(rename = "desc", default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,

    /// Name of the [`AbiContract::structs`] entry naming this field's tuple
    /// type, if any.
    #[serde(rename = "struct", default, skip_serializing_if = "Option::is_none")]
    pub struct_: Option<String>,
}

/// Source-map info for the approval and clear programs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgramSourceInfoPair {
    /// Approval-program source info.
    pub approval: ProgramSourceInfo,

    /// Clear-state-program source info.
    pub clear: ProgramSourceInfo,
}

/// Per-program source info: a list of source entries and the PC-offset method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgramSourceInfo {
    /// Per-PC source entries.
    #[serde(rename = "sourceInfo", default, skip_serializing_if = "Vec::is_empty")]
    pub source_info: Vec<SourceInfo>,

    /// How program-counter offsets are encoded: `"none"` or `"cblocks"`.
    #[serde(rename = "pcOffsetMethod")]
    pub pc_offset_method: String,
}

/// A single source-map entry tying program counters to source/teal/error info.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceInfo {
    /// Program-counter values this entry applies to.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pc: Vec<u64>,

    /// Error message associated with these PCs, if any.
    #[serde(
        rename = "errorMessage",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub error_message: Option<String>,

    /// 1-based line number in the compiled TEAL, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teal: Option<u64>,

    /// 1-based line number in the original source, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// A base64-encoded approval/clear program pair (used by `source` and
/// `byteCode`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgramPair {
    /// Approval program, base64-encoded.
    pub approval: String,

    /// Clear-state program, base64-encoded.
    pub clear: String,
}

/// Which compiler produced the contract, and its version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerInfo {
    /// Compiler name, e.g. `"algod"` or `"puya"`.
    pub compiler: String,

    /// Compiler version.
    #[serde(rename = "compilerVersion")]
    pub compiler_version: CompilerVersion,
}

/// A semantic compiler version, with an optional commit hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerVersion {
    /// Major version.
    pub major: u64,

    /// Minor version.
    pub minor: u64,

    /// Patch version.
    pub patch: u64,

    /// Optional source commit hash.
    #[serde(
        rename = "commitHash",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub commit_hash: Option<String>,
}

/// A TEAL template variable: its encoding and an optional value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Encoding of the variable (ABI type, AVM type, or struct name).
    #[serde(rename = "type")]
    pub type_: String,

    /// The value, base64-encoded, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// A scratch-slot assignment: which slot, and what it holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScratchVariable {
    /// Scratch slot number.
    pub slot: u64,

    /// Encoding of the slot's value (ABI type, AVM type, or struct name).
    #[serde(rename = "type")]
    pub type_: String,
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
    fn struct_field_type_is_string_or_nested() {
        let leaf: StructField = serde_json::from_str(r#"{"name":"a","type":"uint64"}"#).unwrap();
        assert_eq!(leaf.type_, StructFieldType::Type("uint64".to_owned()));

        let nested: StructField =
            serde_json::from_str(r#"{"name":"p","type":[{"name":"x","type":"uint64"}]}"#).unwrap();
        match nested.type_ {
            StructFieldType::Nested(fields) => assert_eq!(fields.len(), 1),
            other => panic!("expected nested struct, got {other:?}"),
        }
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
