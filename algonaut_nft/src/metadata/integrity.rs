//! ARC-3 integrity: the Asset Metadata Hash (`am`) and the SRI `*_integrity`
//! strings.
//!
//! Both schemes are centralised here so no caller re-derives them.
//!
//! - The **`am`** hash is plain SHA-256 of the JSON file when the document has no
//!   `extra_metadata`, and a domain-separated SHA-512/256 construction when it
//!   does (matching Algorand's `SHA-512/256` convention).
//! - The **`*_integrity`** fields are W3C subresource-integrity strings,
//!   `sha256-<base64>`, computed over the *referenced* bytes (image, animation,
//!   …) — which the caller supplies, since this crate fetches nothing by default.

use data_encoding::BASE64;
use sha2::{Digest, Sha256, Sha512_256};

const AM_PREFIX: &[u8] = b"arc0003/am";
const AMJ_PREFIX: &[u8] = b"arc0003/amj";

/// Compute the ARC-3 Asset Metadata Hash (`am`) for a JSON metadata file.
///
/// Pass the *exact* bytes of the JSON file. If the document carries an
/// `extra_metadata` field, pass its decoded bytes as `extra_metadata`; this
/// switches to the domain-separated SHA-512/256 form mandated by ARC-3.
pub fn metadata_hash(json_file: &[u8], extra_metadata: Option<&[u8]>) -> [u8; 32] {
    match extra_metadata {
        None => Sha256::digest(json_file).into(),
        Some(e) => {
            // h = SHA-512/256("arc0003/amj" || json)
            let mut amj = Sha512_256::new();
            amj.update(AMJ_PREFIX);
            amj.update(json_file);
            let h = amj.finalize();

            // am = SHA-512/256("arc0003/am" || h || e)
            let mut am = Sha512_256::new();
            am.update(AM_PREFIX);
            am.update(h);
            am.update(e);
            am.finalize().into()
        }
    }
}

/// Verify a JSON file against an expected on-chain `am` value (32 bytes).
pub fn verify_metadata_hash(
    json_file: &[u8],
    extra_metadata: Option<&[u8]>,
    expected_am: &[u8],
) -> Result<(), crate::NftError> {
    if metadata_hash(json_file, extra_metadata) == expected_am {
        Ok(())
    } else {
        Err(crate::NftError::MetadataHashMismatch)
    }
}

/// Compute the SHA-256 subresource-integrity string (`sha256-<base64>`) for a
/// referenced resource's bytes.
pub fn sri_sha256(resource: &[u8]) -> String {
    format!("sha256-{}", BASE64.encode(&Sha256::digest(resource)))
}

/// Verify a resource's bytes against an SRI string from a metadata field.
///
/// Only `sha256-` is supported (ARC-3's choice). `field` names the metadata
/// field for error reporting (e.g. `image`).
pub fn verify_sri(resource: &[u8], integrity: &str, field: &str) -> Result<(), crate::NftError> {
    let mismatch = || crate::NftError::IntegrityMismatch {
        field: field.to_string(),
    };
    let b64 = integrity.strip_prefix("sha256-").ok_or_else(mismatch)?;
    let expected = BASE64.decode(b64.as_bytes()).map_err(|_| mismatch())?;
    if Sha256::digest(resource).as_slice() == expected.as_slice() {
        Ok(())
    } else {
        Err(mismatch())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn am_without_extra_is_plain_sha256() {
        let json = br#"{"name":"x"}"#;
        let am = metadata_hash(json, None);
        let expected: [u8; 32] = Sha256::digest(json).into();
        assert_eq!(am, expected);
        verify_metadata_hash(json, None, &expected).unwrap();
    }

    #[test]
    fn am_with_extra_is_domain_separated() {
        let json = br#"{"name":"x"}"#;
        let extra = b"\x00\x01\x02";
        let with = metadata_hash(json, Some(extra));
        let without = metadata_hash(json, None);
        // The two constructions must differ.
        assert_ne!(with, without);
        // And recompute deterministically.
        assert_eq!(with, metadata_hash(json, Some(extra)));
    }

    #[test]
    fn sri_round_trips_and_detects_tampering() {
        let bytes = b"the image bytes";
        let sri = sri_sha256(bytes);
        assert!(sri.starts_with("sha256-"));
        verify_sri(bytes, &sri, "image").unwrap();
        assert!(verify_sri(b"tampered", &sri, "image").is_err());
        assert!(verify_sri(bytes, "md5-zzzz", "image").is_err());
    }
}
