use algonaut_encoding::{SignatureVisitor, U8_32Visitor, deserialize_bytes32};
use data_encoding::{BASE32_NOPAD, BASE64};
use ed25519_dalek::{Verifier, VerifyingKey};
use fmt::Debug;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

/// Support for turning 32 byte keys into human-readable mnemonics and back
pub mod mnemonic;

pub mod error;

#[derive(Copy, Clone, Eq, Debug, PartialEq, Serialize, Deserialize)]
pub enum HashType {
    Sha512_256,
    Sumhash,
    Sha256,
}

/// A SHA512_256 hash
#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub struct HashDigest(pub [u8; 32]);

impl FromStr for HashDigest {
    type Err = String;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut decoded = [0; 32];
        decoded.copy_from_slice(
            &BASE64
                .decode(string.as_bytes())
                .map_err(|e| e.to_string())?,
        );
        Ok(HashDigest(decoded))
    }
}

impl Display for HashDigest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", BASE64.encode(&self.0))
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Ed25519PublicKey(pub [u8; 32]);

impl Ed25519PublicKey {
    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        let Ok(verifying_key) = VerifyingKey::from_bytes(&self.0) else {
            return false;
        };
        verifying_key
            .verify(message, &ed25519_dalek::Signature::from_bytes(&signature.0))
            .is_ok()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MasterDerivationKey(pub [u8; 32]);

/// An Ed25519 Signature
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Signature(pub [u8; 64]);

impl Debug for Signature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &BASE64.encode(&self.0))
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        // JSON: base64; msgpack: raw bytes. See ADR
        // domain-types-serialize-for-both-json-and-msgpack.
        if serializer.is_human_readable() {
            serializer.serialize_str(&BASE64.encode(&self.0))
        } else {
            serializer.serialize_bytes(&self.0[..])
        }
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            let bytes = BASE64
                .decode(s.as_bytes())
                .map_err(serde::de::Error::custom)?;
            let arr: [u8; 64] = bytes.try_into().map_err(|v: Vec<u8>| {
                serde::de::Error::custom(format!("expected 64-byte signature, got {}", v.len()))
            })?;
            Ok(Signature(arr))
        } else {
            Ok(Signature(deserializer.deserialize_bytes(SignatureVisitor)?))
        }
    }
}

impl Serialize for HashDigest {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&BASE64.encode(&self.0))
        } else {
            serializer.serialize_bytes(&self.0[..])
        }
    }
}

impl<'de> Deserialize<'de> for HashDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            let bytes = BASE64
                .decode(s.as_bytes())
                .map_err(serde::de::Error::custom)?;
            let arr: [u8; 32] = bytes.try_into().map_err(|v: Vec<u8>| {
                serde::de::Error::custom(format!("expected 32-byte hash, got {}", v.len()))
            })?;
            Ok(HashDigest(arr))
        } else {
            Ok(HashDigest(deserializer.deserialize_bytes(U8_32Visitor)?))
        }
    }
}

impl Debug for HashDigest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", BASE32_NOPAD.encode(&self.0))
    }
}

impl AsRef<[u8]> for HashDigest {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Serialize for Ed25519PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&BASE64.encode(&self.0))
        } else {
            serializer.serialize_bytes(&self.0[..])
        }
    }
}

impl<'de> Deserialize<'de> for Ed25519PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            let bytes = BASE64
                .decode(s.as_bytes())
                .map_err(serde::de::Error::custom)?;
            let arr: [u8; 32] = bytes.try_into().map_err(|v: Vec<u8>| {
                serde::de::Error::custom(format!("expected 32-byte ed25519 pk, got {}", v.len()))
            })?;
            Ok(Ed25519PublicKey(arr))
        } else {
            Ok(Ed25519PublicKey(
                deserializer.deserialize_bytes(U8_32Visitor)?,
            ))
        }
    }
}

impl Debug for Ed25519PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", BASE32_NOPAD.encode(&self.0))
    }
}

impl Debug for MasterDerivationKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", BASE32_NOPAD.encode(&self.0))
    }
}

pub fn deserialize_hash<'de, D>(deserializer: D) -> Result<HashDigest, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(HashDigest(deserialize_bytes32(deserializer)?))
}

pub fn deserialize_opt_hash<'de, D>(deserializer: D) -> Result<Option<HashDigest>, D::Error>
where
    D: Deserializer<'de>,
{
    // TODO needs testing
    let hash = deserialize_bytes32(deserializer)?;
    if hash == [0; 32] {
        Ok(None)
    } else {
        Ok(Some(HashDigest(hash)))
    }
}

pub fn deserialize_mdk<'de, D>(deserializer: D) -> Result<MasterDerivationKey, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(MasterDerivationKey(deserialize_bytes32(deserializer)?))
}

pub fn deserialize_public_keys<'de, D>(deserializer: D) -> Result<Vec<Ed25519PublicKey>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    <Vec<String>>::deserialize(deserializer)?
        .iter()
        .map(|string| {
            let mut decoded = [0; 32];
            let bytes = BASE64.decode(string.as_bytes()).map_err(D::Error::custom)?;
            decoded.copy_from_slice(&bytes);
            Ok(Ed25519PublicKey(decoded))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_json_round_trip_is_base64() {
        let sig = Signature([3; 64]);
        let json = serde_json::to_string(&sig).unwrap();
        assert_eq!(json, format!("\"{}\"", BASE64.encode(&sig.0)));
        let parsed: Signature = serde_json::from_str(&json).unwrap();
        assert_eq!(sig, parsed);
    }

    #[test]
    fn signature_msgpack_is_raw_bytes() {
        let sig = Signature([3; 64]);
        let bytes = rmp_serde::to_vec(&sig).unwrap();
        assert_eq!(bytes[0], 0xc4); // bin8
        assert_eq!(bytes[1], 0x40); // 64 bytes
        assert_eq!(&bytes[2..], &[3u8; 64]);
        let parsed: Signature = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(sig, parsed);
    }

    #[test]
    fn hash_digest_json_round_trip_is_base64() {
        let h = HashDigest([11; 32]);
        let json = serde_json::to_string(&h).unwrap();
        assert_eq!(json, format!("\"{}\"", BASE64.encode(&h.0)));
        let parsed: HashDigest = serde_json::from_str(&json).unwrap();
        assert_eq!(h, parsed);
    }

    #[test]
    fn hash_digest_msgpack_is_raw_bytes() {
        let h = HashDigest([11; 32]);
        let bytes = rmp_serde::to_vec(&h).unwrap();
        assert_eq!(bytes[0], 0xc4);
        assert_eq!(bytes[1], 0x20);
        assert_eq!(&bytes[2..], &[11u8; 32]);
        let parsed: HashDigest = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(h, parsed);
    }

    #[test]
    fn ed25519_public_key_json_round_trip_is_base64() {
        let pk = Ed25519PublicKey([1; 32]);
        let json = serde_json::to_string(&pk).unwrap();
        assert_eq!(json, format!("\"{}\"", BASE64.encode(&pk.0)));
        let parsed: Ed25519PublicKey = serde_json::from_str(&json).unwrap();
        assert_eq!(pk, parsed);
    }

    #[test]
    fn ed25519_public_key_msgpack_is_raw_bytes() {
        let pk = Ed25519PublicKey([1; 32]);
        let bytes = rmp_serde::to_vec(&pk).unwrap();
        assert_eq!(bytes[0], 0xc4);
        assert_eq!(bytes[1], 0x20);
        assert_eq!(&bytes[2..], &[1u8; 32]);
        let parsed: Ed25519PublicKey = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(pk, parsed);
    }
}
