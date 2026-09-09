//! ARC-53 — collection-level metadata declarations.
//!
//! A project-level (not per-asset) declaration, typically uploaded to IPFS and
//! linked from a smart contract under a `project` key. It groups an NFT
//! collection that may span wallets or lack consistent unit-name prefixes, and
//! lets "blank slate" collections (e.g. ARC-19 mints) re-expose their trait
//! `properties` off-chain. `version` is the only required top-level field.

use serde::{Deserialize, Serialize};

/// An ARC-53 metadata declaration document.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Declaration {
    /// The declaration schema version (the only required field).
    pub version: String,

    /// Wallets associated with the project and their roles.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub associates: Vec<Associate>,

    /// The NFT collections this project declares.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub collections: Vec<Collection>,

    /// Individual tokens (by asset id) for image sourcing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<Token>,

    /// Frequently asked questions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub faq: Vec<Faq>,

    /// Arbitrary extra key/value pairs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Map<String, serde_json::Value>>,
}

/// A wallet associated with the project.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Associate {
    /// The associate's Algorand address.
    pub address: String,
    /// The associate's role (e.g. `creator`, `artist`).
    pub role: String,
}

/// A declared NFT collection.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Collection {
    /// The collection name.
    pub name: String,
    /// The network (defaults to `algorand`; `multichain` is special).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    /// Unit-name prefixes used to scope membership.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prefixes: Vec<String>,
    /// Minting wallets that scope membership.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub addresses: Vec<String>,
    /// Asset ids directly included.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assets: Vec<u64>,
    /// Asset ids directly excluded.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_assets: Vec<u64>,
    /// Royalty in basis points (0–10000).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub royalty_percentage: Option<u64>,
    /// Whether the collection is flagged explicit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explicit: Option<bool>,
    /// Off-chain trait declarations (name → possible values).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<CollectionProperty>,
    /// Arbitrary extra key/value pairs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Map<String, serde_json::Value>>,
}

/// A named property of a collection and its possible values.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CollectionProperty {
    /// The property name (e.g. `background`).
    pub name: String,
    /// The possible values for this property.
    pub values: Vec<CollectionPropertyValue>,
}

/// A single possible value of a [`CollectionProperty`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CollectionPropertyValue {
    /// The value name (e.g. `red`).
    pub name: String,
    /// Optional image URI for this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// SHA-256 subresource-integrity for `image`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_integrity: Option<String>,
    /// MIME type of `image`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_mimetype: Option<String>,
}

/// A token entry referenced by the declaration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Token {
    /// The token's asset id.
    pub asset_id: u64,
    /// Optional image URI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// SHA-256 subresource-integrity for `image`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_integrity: Option<String>,
    /// MIME type of `image`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_mimetype: Option<String>,
}

/// A frequently-asked-question entry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Faq {
    /// The question.
    pub q: String,
    /// The answer.
    pub a: String,
}

impl Declaration {
    /// Parse from JSON bytes.
    pub fn from_json(bytes: &[u8]) -> Result<Self, crate::NftError> {
        Ok(serde_json::from_slice(bytes)?)
    }

    /// Serialise to JSON bytes.
    pub fn to_json(&self) -> Result<Vec<u8>, crate::NftError> {
        Ok(serde_json::to_vec(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_declaration_round_trips() {
        let doc = br#"{"version":"1.0","collections":[{"name":"Cubs","royalty_percentage":500}]}"#;
        let d = Declaration::from_json(doc).unwrap();
        assert_eq!(d.version, "1.0");
        assert_eq!(d.collections[0].royalty_percentage, Some(500));
        // version survives, empty vecs are omitted on re-serialisation.
        let s = String::from_utf8(d.to_json().unwrap()).unwrap();
        assert!(s.contains("\"version\":\"1.0\""));
        assert!(!s.contains("associates"));
    }
}
