//! ARC-3 — the off-chain JSON metadata file.
//!
//! Based on the ERC-1155 Metadata URI JSON schema, with ARC-3's additions: the
//! per-URI `*_integrity` (subresource-integrity) and `*_mimetype` fields, the
//! `extra_metadata` hashing hook, and relative-URI support. Every field is
//! optional, so a valid ERC-1155 document is a valid ARC-3 document.
//!
//! This is the model only; the `am` hash lives in [`crate::metadata::integrity`]
//! and the ASA-parameter conventions (the `#arc3` URL fragment, the pure/
//! fractional shape) live in [`crate::asa`].

use serde::{Deserialize, Serialize};

/// An ARC-3 JSON metadata document.
///
/// Round-trips byte-faithfully: unset fields are skipped on serialisation, and
/// any keys ARC-3 does not name are preserved under [`Metadata::properties`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    /// Identifies the asset this token represents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The number of decimal places. If present it MUST equal the ASA's `dt`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decimals: Option<u32>,

    /// Describes the asset this token represents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A URI to a `image/*` resource representing the asset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// The SHA-256 subresource-integrity string for [`Metadata::image`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_integrity: Option<String>,

    /// The MIME type of [`Metadata::image`] (e.g. `image/png`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_mimetype: Option<String>,

    /// Background color, a six-character hex string without a leading `#`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,

    /// A URI to an external application or website.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_url: Option<String>,

    /// Subresource-integrity for [`Metadata::external_url`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_url_integrity: Option<String>,

    /// MIME type of [`Metadata::external_url`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_url_mimetype: Option<String>,

    /// A URI to a multimedia attachment for the asset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation_url: Option<String>,

    /// Subresource-integrity for [`Metadata::animation_url`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation_url_integrity: Option<String>,

    /// MIME type of [`Metadata::animation_url`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation_url_mimetype: Option<String>,

    /// Arbitrary properties / attributes. This is where ARC-16 `traits` and
    /// ARC-36 `filters` live; keep them as nested objects under this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<serde_json::Value>,

    /// Extra base64-encoded metadata. Its presence changes how the ARC-3 `am`
    /// hash is computed (see [`crate::metadata::integrity`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_metadata: Option<String>,

    /// Localization directives for translating the above fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub localization: Option<Localization>,
}

/// ARC-3 localization block. `uri` SHOULD contain the `{locale}` placeholder.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Localization {
    /// URI template with a `{locale}` substitution point.
    pub uri: String,
    /// The locale of the top-level document (e.g. `en`).
    pub default: String,
    /// The list of locales available at `uri`.
    pub locales: Vec<String>,
    /// Optional per-locale subresource-integrity strings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<serde_json::Map<String, serde_json::Value>>,
}

impl Metadata {
    /// Parse from a JSON byte slice (e.g. the bytes fetched from `au`).
    pub fn from_json(bytes: &[u8]) -> Result<Self, crate::NftError> {
        Ok(serde_json::from_slice(bytes)?)
    }

    /// Serialise to canonical JSON bytes.
    pub fn to_json(&self) -> Result<Vec<u8>, crate::NftError> {
        Ok(serde_json::to_vec(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_skips_empty() {
        let m = Metadata {
            name: Some("My NFT".into()),
            decimals: Some(0),
            image: Some("ipfs://Qm.../image.png".into()),
            image_integrity: Some("sha256-47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=".into()),
            ..Default::default()
        };
        let json = m.to_json().unwrap();
        let s = String::from_utf8(json.clone()).unwrap();
        // Unset fields are omitted entirely.
        assert!(!s.contains("description"));
        assert!(!s.contains("animation_url"));
        assert_eq!(Metadata::from_json(&json).unwrap(), m);
    }

    #[test]
    fn accepts_erc1155_superset() {
        // An ERC-1155-style document with only name/description/image parses.
        let doc = br#"{"name":"x","description":"y","image":"ipfs://z"}"#;
        let m = Metadata::from_json(doc).unwrap();
        assert_eq!(m.name.as_deref(), Some("x"));
    }
}
