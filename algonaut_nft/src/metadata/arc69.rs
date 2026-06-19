//! ARC-69 — on-chain metadata stored in the asset-config `note`.
//!
//! Unlike ARC-3, ARC-69 stores its JSON in the `note` field of the most recent
//! asset-config (`acfg`) transaction, so it can be applied to assets that already
//! exist and updated by the manager. The single required field is `standard`,
//! which MUST equal `arc69`.

use serde::{Deserialize, Serialize};

/// The required value of the ARC-69 `standard` field.
pub const STANDARD: &str = "arc69";

/// The media type signalled by the trailing `#x` fragment on the asset URL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaType {
    /// `#i` — image (the default when no fragment is present).
    Image,
    /// `#v` — video.
    Video,
    /// `#a` — audio.
    Audio,
    /// `#p` — PDF.
    Pdf,
    /// `#h` — HTML / interactive.
    Html,
}

impl MediaType {
    /// Derive the media type from an ARC-69 asset URL's trailing fragment.
    /// Per the spec, a URL with no recognised fragment defaults to [`MediaType::Image`].
    pub fn from_url(url: &str) -> Self {
        match url.rsplit_once('#').map(|(_, frag)| frag) {
            Some("v") => MediaType::Video,
            Some("a") => MediaType::Audio,
            Some("p") => MediaType::Pdf,
            Some("h") => MediaType::Html,
            _ => MediaType::Image,
        }
    }

    /// The fragment character (`i`/`v`/`a`/`p`/`h`).
    pub fn fragment(self) -> char {
        match self {
            MediaType::Image => 'i',
            MediaType::Video => 'v',
            MediaType::Audio => 'a',
            MediaType::Pdf => 'p',
            MediaType::Html => 'h',
        }
    }
}

/// A single OpenSea-style attribute (the deprecated ARC-69 `attributes` form).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    /// The attribute name.
    pub trait_type: String,
    /// The attribute value (string or number).
    pub value: serde_json::Value,
}

/// An ARC-69 metadata document, as stored in the `acfg` note.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    /// MUST be `arc69`. Defaults to [`STANDARD`] so a hand-built value is valid.
    #[serde(default = "default_standard")]
    pub standard: String,

    /// A human-readable description of the asset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A URI to an external application or website.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_url: Option<String>,

    /// A URI to the high-resolution media file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_url: Option<String>,

    /// The MIME type of the asset URL (e.g. `video/mp4`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// Preferred EIP-1155 "simple properties" object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<serde_json::Value>,

    /// Deprecated OpenSea-style attributes array; supported for reading.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<Attribute>>,
}

fn default_standard() -> String {
    STANDARD.to_string()
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            standard: default_standard(),
            description: None,
            external_url: None,
            media_url: None,
            mime_type: None,
            properties: None,
            attributes: None,
        }
    }
}

impl Metadata {
    /// Parse from the raw bytes of an `acfg` transaction note.
    ///
    /// Returns [`NftError::InvalidArc69Note`](crate::NftError::InvalidArc69Note)
    /// if the bytes are not JSON or the `standard` field is not `arc69`.
    pub fn from_note(note: &[u8]) -> Result<Self, crate::NftError> {
        let m: Metadata = serde_json::from_slice(note)
            .map_err(|e| crate::NftError::InvalidArc69Note(e.to_string()))?;
        if m.standard != STANDARD {
            return Err(crate::NftError::InvalidArc69Note(format!(
                "standard is {:?}, expected {STANDARD:?}",
                m.standard
            )));
        }
        Ok(m)
    }

    /// Serialise to the bytes that go into the `acfg` note.
    pub fn to_note(&self) -> Result<Vec<u8>, crate::NftError> {
        Ok(serde_json::to_vec(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_type_from_fragment() {
        assert_eq!(MediaType::from_url("ipfs://x#v"), MediaType::Video);
        assert_eq!(MediaType::from_url("ipfs://x#h"), MediaType::Html);
        // No fragment, or an unknown one, defaults to Image.
        assert_eq!(MediaType::from_url("ipfs://x"), MediaType::Image);
        assert_eq!(MediaType::from_url("ipfs://x#zzz"), MediaType::Image);
    }

    #[test]
    fn note_requires_arc69_standard() {
        let good = br#"{"standard":"arc69","description":"hi"}"#;
        assert_eq!(
            Metadata::from_note(good).unwrap().description.as_deref(),
            Some("hi")
        );

        let bad = br#"{"standard":"arc3"}"#;
        assert!(matches!(
            Metadata::from_note(bad),
            Err(crate::NftError::InvalidArc69Note(_))
        ));
    }

    #[test]
    fn default_fills_standard() {
        let m = Metadata::default();
        let s = String::from_utf8(m.to_note().unwrap()).unwrap();
        assert!(s.contains("\"standard\":\"arc69\""));
    }
}
