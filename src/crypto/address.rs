use crate::encoding::SignatureVisitor;
use crate::encoding::U8_32Visitor;
use data_encoding::BASE32_NOPAD;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::Digest;
use std::fmt::{Debug, Formatter};

type ChecksumAlg = sha2::Sha512Trunc256;

const CHECKSUM_LEN: usize = 4;
const HASH_LEN: usize = 32;

/// Public key address
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Address(pub [u8; HASH_LEN]);

impl Address {
    pub fn new(bytes: [u8; HASH_LEN]) -> Address {
        Address(bytes)
    }

    /// Decode address from base64 string with checksum
    pub fn from_string(string: &str) -> Result<Address, String> {
        let checksum_address = match BASE32_NOPAD.decode(string.as_bytes()) {
            Ok(decoded) => decoded,
            Err(err) => return Err(format!("Error decoding base32: {:?}", err)),
        };
        if checksum_address.len() != (HASH_LEN + CHECKSUM_LEN) {
            return Err("Input string is an invalid address. Wrong length".to_string());
        }
        let (address, checksum) = checksum_address.split_at(HASH_LEN);
        let hashed = ChecksumAlg::digest(&address);
        if &hashed[(HASH_LEN - CHECKSUM_LEN)..] == checksum {
            let mut bytes = [0; HASH_LEN];
            bytes.copy_from_slice(address);
            Ok(Address::new(bytes))
        } else {
            Err("Input checksum did not validate".to_string())
        }
    }

    /// Encode address to base64 string with checksum
    pub fn encode_string(&self) -> String {
        let hashed = ChecksumAlg::digest(&self.0);
        let checksum = &hashed[(HASH_LEN - CHECKSUM_LEN)..];
        let checksum_address = [&self.0, checksum].concat();
        BASE32_NOPAD.encode(&checksum_address)
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Address(deserializer.deserialize_bytes(U8_32Visitor)?))
    }
}

/// Convenience struct for handling multisig public identities
#[derive(Debug, Clone)]
pub struct MultisigAddress {
    /// the version of this multisig
    pub version: u8,
    /// how many signatures are needed to fully sign as this address
    pub threshold: u8,
    /// ordered list of public keys that could potentially sign a message
    pub public_keys: Vec<Ed25519PublicKey>,
}

impl MultisigAddress {
    pub fn new(
        version: u8,
        threshold: u8,
        addresses: &[Address],
    ) -> Result<MultisigAddress, String> {
        if version != 1 {
            Err("Unknown msig version".to_string())
        } else if threshold == 0 || addresses.is_empty() || threshold > addresses.len() as u8 {
            Err("Invalid threshold".to_string())
        } else {
            Ok(MultisigAddress {
                version,
                threshold,
                public_keys: addresses
                    .iter()
                    .map(|address| Ed25519PublicKey(address.0))
                    .collect(),
            })
        }
    }

    /// Generates a checksum from the contained public keys usable as an address
    pub fn address(&self) -> Address {
        let mut buf = b"MultisigAddr".to_vec();
        buf.push(self.version);
        buf.push(self.threshold);
        for key in &self.public_keys {
            buf.extend_from_slice(&key.0);
        }
        let hashed = ChecksumAlg::digest(&buf);
        let mut bytes = [0; HASH_LEN];
        bytes.copy_from_slice(&hashed);
        Address::new(bytes)
    }
}

/// An Ed25519 Signature
#[derive(Copy, Clone)]
pub struct Signature(pub [u8; 64]);

impl Debug for Signature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Signature").field(&self.0.to_vec()).finish()
    }
}

impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..64 {
            if self.0[i] != other.0[i] {
                return false;
            }
        }
        true
    }
}

impl Eq for Signature {}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Signature(deserializer.deserialize_bytes(SignatureVisitor)?))
    }
}

#[derive(Default, Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct MultisigSignature {
    #[serde(rename = "subsig")]
    pub subsigs: Vec<MultisigSubsig>,
    #[serde(rename = "thr")]
    pub threshold: u8,
    #[serde(rename = "v")]
    pub version: u8,
}

impl Serialize for MultisigSignature {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        // For some reason SerializeStruct ends up serializing as an array, so this explicitly serializes as a map
        use serde::ser::SerializeMap;
        let mut state = serializer.serialize_map(Some(3))?;
        state.serialize_entry("subsig", &self.subsigs)?;
        state.serialize_entry("thr", &self.threshold)?;
        state.serialize_entry("v", &self.version)?;
        state.end()
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct MultisigSubsig {
    #[serde(rename = "pk")]
    pub key: Ed25519PublicKey,
    #[serde(rename = "s")]
    pub sig: Option<Signature>,
}

impl Serialize for MultisigSubsig {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        let len = if self.sig.is_some() { 2 } else { 1 };
        let mut state = serializer.serialize_map(Some(len))?;
        state.serialize_entry("pk", &self.key)?;
        if let Some(sig) = &self.sig {
            state.serialize_entry("s", sig)?;
        }
        state.end()
    }
}

/// A SHA512_256 hash
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct HashDigest(pub [u8; 32]);

impl Serialize for HashDigest {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl<'de> Deserialize<'de> for HashDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(HashDigest(deserializer.deserialize_bytes(U8_32Visitor)?))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Ed25519PublicKey(pub [u8; 32]);

impl Serialize for Ed25519PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl<'de> Deserialize<'de> for Ed25519PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Ed25519PublicKey(
            deserializer.deserialize_bytes(U8_32Visitor)?,
        ))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MasterDerivationKey(pub [u8; 32]);

#[cfg(test)]
mod tests {
    use super::*;

    /// Trying to decode a valid base32 address should succeed.
    #[test]
    fn decode() {
        let s = "737777777777777777777777777777777777777777777777777UFEJ2CI";

        let addr = Address::from_string(s).expect("failed to decode address from string");
        assert_eq!(s, addr.encode_string());
    }

    /// Tryng to decode a base32 address with an invalid checksum must fail.
    #[test]
    fn decode_invalid_checksum() {
        let invalid_csum = "737777777777777777777777777777777777777777777777777UFEJ2CJ";

        assert!(Address::from_string(invalid_csum).is_err());
    }
}
