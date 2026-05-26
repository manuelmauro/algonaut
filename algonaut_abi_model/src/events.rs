//! ARC-28 event definitions.

use serde::{Deserialize, Serialize};

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

    /// Name of the `AbiContract::structs` entry naming this field's tuple
    /// type, if any.
    #[serde(rename = "struct", default, skip_serializing_if = "Option::is_none")]
    pub struct_: Option<String>,
}
