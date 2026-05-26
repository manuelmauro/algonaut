//! Declared application state: creation schema, fixed keys, and dynamic maps.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
