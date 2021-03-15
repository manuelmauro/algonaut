//! This file mostly just hides away various trait implementations that would clutter up and distract from the more important code elsewhere
use crate::error::{AlgorandError, BuilderError};
use crate::models::{HashDigest, MasterDerivationKey};
use data_encoding::BASE64;
use derive_more::Display;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serializer};

pub(crate) struct SignatureVisitor;

impl<'de> Visitor<'de> for SignatureVisitor {
    type Value = [u8; 64];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a 64 byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() == 64 {
            let mut bytes = [0; 64];
            bytes.copy_from_slice(v);
            Ok(bytes)
        } else {
            Err(E::custom(format!("Invalid signature length: {}", v.len())))
        }
    }
}

pub(crate) struct U8_32Visitor;

impl<'de> Visitor<'de> for U8_32Visitor {
    type Value = [u8; 32];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a 32 byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() == 32 {
            let mut bytes = [0; 32];
            bytes.copy_from_slice(v);
            Ok(bytes)
        } else {
            Err(E::custom(format!("Invalid byte array length: {}", v.len())))
        }
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

pub fn deserialize_bytes32<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
where
    D: Deserializer<'de>,
{
    let s = <&str>::deserialize(deserializer)?;
    let mut decoded = [0; 32];
    decoded.copy_from_slice(&BASE64.decode(s.as_bytes()).unwrap());
    Ok(decoded)
}

pub fn deserialize_bytes64<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let s = <&str>::deserialize(deserializer)?;
    let mut decoded = [0; 64];
    let bytes = BASE64.decode(s.as_bytes()).map_err(D::Error::custom)?;
    decoded.copy_from_slice(&bytes);
    Ok(decoded)
}

pub fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <&str>::deserialize(deserializer)?;
    Ok(BASE64.decode(s.as_bytes()).unwrap())
}

pub fn serialize_bytes<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&BASE64.encode(bytes))
}

/// An API token.
#[derive(Display)]
#[display(fmt = "{}", token)]
pub struct ApiToken {
    token: String,
}

const TOKEN_LENGTH: usize = 64;

impl ApiToken {
    /// Parses a string slice representing an API token.
    pub fn parse(token: &str) -> Result<Self, AlgorandError> {
        if token.len() != TOKEN_LENGTH {
            return Err(BuilderError::BadToken.into());
        }

        Ok(ApiToken {
            token: token.to_string(),
        })
    }
}
