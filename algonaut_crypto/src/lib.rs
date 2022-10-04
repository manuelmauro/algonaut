use algonaut_encoding::{deserialize_bytes32, SignatureVisitor, U8_32Visitor};
use data_encoding::{BASE32_NOPAD, BASE64};
use fmt::Debug;
use ring::signature::UnparsedPublicKey;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

/// Support for turning 32 byte keys into human-readable mnemonics and back
pub mod mnemonic;

///
pub mod error;

pub enum HashType {
    Sha512_256,
	Sumhash,
	Sha256,
}

pub struct HashFactory {
    pub hash_type: HashType 
}

/// A SHA512_256 hash
#[derive(Copy, Clone, Eq, PartialEq)]
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
        let peer_public_key = UnparsedPublicKey::new(&ring::signature::ED25519, self.0);
        match peer_public_key.verify(message, signature.0.as_ref()) {
            Ok(()) => true,
            Err(_e) => {
                println!("Signature verification failed");
                false
            }
        }
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
