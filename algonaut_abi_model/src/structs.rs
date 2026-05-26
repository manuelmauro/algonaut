//! Named-struct definitions: a struct's field list and each field's type.

use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
